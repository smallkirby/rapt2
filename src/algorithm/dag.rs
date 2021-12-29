/*
 This file defines DAG (Directed Acyclic Graph) representation of package dependencies.

 In theory, packages can have cyclic dependencies.
 In that case, choose arbitrary edge and cut it.
 (`Pre-Depends` must not have cyclic dependencies.)

 Here, Graph uses index-managed node structures to avoid annoying lifetime managements.
*/

/*
 XXX
 Now, this Graph supports only single depth cyclic depth.
 In short, below condition is unsupported:

   ┌─────────────┐
   ▼             │
   1 ───────► 2  │
   ▲          │  │
   │          │  │
   │          │  │
   │          │  │
   │          ▼  │
   4 ◄─────── 3 ─┘

 Now, this would be converted to:

   123 ◄──────────┐
       └──────────► 4
*/

use crate::package::{client::PackageWithSource, package::*};

use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DagError {
  #[error("Target package not existing.")]
  TargetNotExist { from: String, target: String },

  #[error("Function is called with invalid Graph state.")]
  InvalidStateError,
}

struct PackageNode {
  pub package: Package,
  pub to: Vec<usize>,
  pub revto: Vec<usize>,
  pub normal_index: i64,
  pub group: i64,
  pub visited: bool,
}

// Simple node used to re-construct graph based on the result of SCC.
struct SimpleNode {
  group: i64,
  visited: bool,
  to: Vec<i64>,
}

struct Graph {
  nodes: Vec<PackageNode>,
  index: i64,                  // total visited count
  group_num: i64,              // current group total count
  topological_order: Vec<i64>, // order of groups after topological sort
  topological_count: i64,
  simplified_nodes: Vec<SimpleNode>,
}

impl Graph {
  pub fn from(packages: Vec<Package>) -> Result<Self, DagError> {
    construct_nodes(packages)
  }

  // way-nome indexed DFS
  pub fn dfs_all(&mut self) {
    for ix in 0..self.nodes.len() {
      if self.nodes[ix].normal_index == -1 {
        self.dfs_internal(ix);
      }
    }
  }

  fn dfs_internal(&mut self, start: usize) {
    if self.nodes[start].visited {
      return;
    }
    self.nodes[start].visited = true;
    for jx in 0..self.nodes[start].to.len() {
      self.dfs_internal(self.nodes[start].to[jx]);
    }
    self.nodes[start].normal_index = self.index;
    self.index += 1;
  }

  // way-nome indexed reverse DFS
  fn rev_dfs_grouping(&mut self, start: usize) {
    if self.nodes[start].visited {
      return;
    }
    self.nodes[start].visited = true;
    for jx in 0..self.nodes[start].revto.len() {
      self.rev_dfs_grouping(self.nodes[start].revto[jx]);
    }
    self.nodes[start].group = self.group_num;
  }

  fn clear_visited(&mut self) {
    for node in &mut self.nodes {
      node.visited = false;
    }
  }

  // Decomposition of Strongly Connected Components, to create DAG.
  fn scc(&mut self) -> Result<(), DagError> {
    self.clear_visited();
    self.check_before_scc_valid()?;

    // first, DFS in home-way order
    self.dfs_all();

    // second, reverse DFS and make groups
    self.clear_visited();
    for ix in (0..self.nodes.len()).rev() {
      let node_ix = self
        .nodes
        .iter()
        .position(|node| node.normal_index == ix as i64)
        .unwrap();
      if self.nodes[node_ix].group == -1 {
        self.rev_dfs_grouping(node_ix);
        self.group_num += 1;
      }
    }

    Ok(())
  }

  fn check_before_scc_valid(&self) -> Result<(), DagError> {
    let graph_state_valid = self.index == 0 && self.group_num == 0;
    let packages_state_valid = self
      .nodes
      .iter()
      .all(|node| node.normal_index == -1 && node.group == -1 && node.visited == false);

    if graph_state_valid && packages_state_valid {
      Ok(())
    } else {
      Err(DagError::InvalidStateError)
    }
  }

  // Topological Sort of cyclic dependencies using SCC
  fn topological_sort(&mut self) {
    // initialize topological orders
    self.topological_order = (0..self.group_num).into_iter().map(|_| -1).collect();
    self.clear_visited();
    self.topological_count = 0;

    // construct simplified graph based on the result of SCC
    self.construct_simple_graph();

    // home-way indexed DFS
    for ix in 0..self.simplified_nodes.len() {
      if !self.simplified_nodes[ix].visited {
        self.topological_dfs(ix);
      }
    }
  }

  // home-way indexed DFS for topological sort.
  fn topological_dfs(&mut self, start: usize) {
    if self.simplified_nodes[start].visited {
      return;
    }
    self.simplified_nodes[start].visited = true;

    for jx in 0..self.simplified_nodes[start].to.len() {
      self.topological_dfs(jx);
    }

    self.topological_order[self.simplified_nodes[start].group as usize] = self.topological_count;
    self.topological_count += 1;
  }

  fn construct_simple_graph(&mut self) {
    let mut result = vec![];
    for group_id in 0..self.group_num {
      let nodes: Vec<&PackageNode> = self
        .nodes
        .iter()
        .filter(|node| node.group == group_id)
        .collect();
      let mut simple_node = SimpleNode {
        group: group_id,
        to: vec![],
        visited: false,
      };
      for node in nodes {
        for to in &node.to {
          let to_group = self.nodes[to.clone()].group;
          if to_group != group_id && !simple_node.to.contains(&to_group) {
            simple_node.to.push(to_group);
          }
        }
      }
      result.push(simple_node);
    }

    self.simplified_nodes = result;
  }

  #[allow(dead_code)]
  pub fn debug_print_dfs_order(&self) {
    for (ix, node) in self.nodes.iter().enumerate() {
      println!("{}: {}", ix, node.normal_index);
    }
  }
}

fn construct_nodes(packages: Vec<Package>) -> Result<Graph, DagError> {
  // initiate nodes
  let mut nodes: Vec<PackageNode> = packages
    .into_iter()
    .map(|package| PackageNode {
      package,
      to: vec![],
      revto: vec![],
      normal_index: -1,
      group: -1,
      visited: false,
    })
    .collect();

  // assign TOs and reverse TOs
  for ix in 0..nodes.len() {
    let depends: Vec<Depends> = nodes[ix]
      .package
      .depends
      .iter()
      .map(|dep_anys| {
        // XXX choose arbitrary dep. Should choose the most loose dependency?
        dep_anys.depends[0].clone()
      })
      .collect();
    for dep in depends {
      let cand = match nodes
        .iter()
        .position(|package_nodes| package_nodes.package.name == dep.package)
      {
        Some(i) => i,
        None => {
          return Err(DagError::TargetNotExist {
            target: dep.package.to_string(),
            from: nodes[ix].package.name.to_string(),
          })
        }
      };
      // assign normal tos
      nodes[ix].to.push(cand);
      // assingn reverse tos
      nodes[cand].revto.push(ix);
    }
  }

  Ok(Graph {
    nodes,
    index: 0,
    group_num: 0,
    topological_order: vec![],
    topological_count: 0,
    simplified_nodes: vec![],
  })
}

pub fn sort_depends(deps: HashSet<PackageWithSource>) -> Vec<PackageWithSource> {
  let packages: Vec<Package> = deps.iter().map(|pws| pws.package.clone()).collect();
  let mut graph = Graph::from(packages).unwrap();

  // do SCC to make a DAG and do topological sort
  graph.scc().unwrap();
  graph.topological_sort();

  let mut results = vec![];
  let group_orders = graph.topological_order;
  for group_order in 0..group_orders.len() {
    let group_id = group_orders
      .iter()
      .position(|i| i.clone() == group_order as i64)
      .unwrap();
    let nodes: Vec<&PackageNode> = graph
      .nodes
      .iter()
      .filter(|node| node.group == group_id as i64)
      .collect();

    // XXX In the same group, push nodes in arbitrary order
    for node in nodes {
      // combine with source information
      let pws = deps
        .iter()
        .find(|pws| pws.package == node.package)
        .unwrap()
        .clone();
      results.push(pws);
    }
  }

  results
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn vec2packages(tos: Vec<Vec<u32>>) -> Vec<Package> {
    let mut packages = vec![];
    for ix in 0..tos.len() {
      let current_tos = &tos[ix];
      let dep_strings: Vec<String> = current_tos.iter().map(|to| to.to_string()).collect();
      let package = Package {
        name: ix.to_string(),
        depends: DependsAnyOf::from(&dep_strings.join(", ")).unwrap(),
        ..Default::default()
      };
      packages.push(package);
    }

    packages
  }

  fn to_orders(graph: &Graph) -> Vec<i64> {
    graph.nodes.iter().map(|node| node.normal_index).collect()
  }

  fn to_groups(graph: &Graph) -> HashMap<i64, Vec<i64>> {
    let mut groups: HashMap<i64, Vec<i64>> = HashMap::new();

    for (ix, node) in graph.nodes.iter().enumerate() {
      let current = groups.entry(node.group).or_insert(vec![]);
      current.push(ix as i64);
    }

    groups
  }

  #[test]
  fn test_dfs_all() {
    /*
       3 ←--
       ↑   |
       0 → 1
       ↓
       2

       DFS(0):     (3, 1, 2, 0)
       rev DFS(0): (0, 1, 2, 3)
    */
    let packages = vec2packages(vec![vec![1, 2, 3], vec![3], vec![], vec![]]);
    let mut graph = Graph::from(packages).unwrap();
    let order = vec![3, 1, 2, 0];

    graph.dfs_all();
    assert_eq!(to_orders(&graph), order);

    assert_eq!(graph.scc().is_err(), true);
  }

  #[test]
  fn test_scc() {
    /*
     [ BEFORE SCC ]
       0────►1 ────┐
             ▲     │
             │     ▼
             │     2 ──────►4 ─────►5     9
             │     │        │       │     ▲ │
             │     │        │       │     │ │
             3 ◄───┘        ▼       ▼     │ ▼
                           6 ─────►7 ────►8
     [ AFTER SCC ]
       G0─────►G1─────► G2 ───►G4
                         │      │
                         │      │
                         ▼      ▼
                       G3────►G5 ───────►G6
    */
    let packages = vec2packages(vec![
      vec![1],
      vec![2],
      vec![3, 4],
      vec![1],
      vec![5, 6],
      vec![7],
      vec![7],
      vec![8],
      vec![9],
      vec![8],
    ]);
    let mut graph = Graph::from(packages).unwrap();
    let mut answer_groups: HashMap<i64, Vec<i64>> = HashMap::new();
    answer_groups.insert(0, vec![0]);
    answer_groups.insert(1, vec![1, 2, 3]);
    answer_groups.insert(2, vec![4]);
    answer_groups.insert(3, vec![6]);
    answer_groups.insert(4, vec![5]);
    answer_groups.insert(5, vec![7]);
    answer_groups.insert(6, vec![8, 9]);

    graph.scc().unwrap();
    let groups = to_groups(&graph);
    assert_eq!(groups, answer_groups);

    let topological_answer = vec![0, 1, 2, 3, 4, 5, 6];
    graph.topological_sort();
    assert_eq!(topological_answer, graph.topological_order);
  }
}
