/*
 This file defines DAG (Directed Acyclic Graph) representation of package dependencies.

 In theory, packages can have cyclic dependencies.
 In that case, choose arbitrary edge and cut it.
 (`Pre-Depends` must not have cyclic dependencies.)

 Here, Graph uses index-managed node structures to avoid annoying lifetime managements.
*/

use crate::package::package::*;

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

struct Graph {
  nodes: Vec<PackageNode>,
  index: i64,     // total visited count
  group_num: i64, // current group total count
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
    unimplemented!()
  }

  #[allow(dead_code)]
  pub fn debug_print_dfs_order(&self) {
    for (ix, node) in self.nodes.iter().enumerate() {
      println!("{}: {}", ix, node.normal_index);
    }
  }
}

//fn construct_nodes<'a>(packages: Vec<Package>) -> (Arena<PackageNode<'a>>, Vec<&'a mut PackageNode<'a>>) {
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
  })
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
    let rev_order = vec![0, 1, 2, 3];

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
  }
}
