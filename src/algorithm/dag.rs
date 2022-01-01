/*
 This file defines DAG (Directed Acyclic Graph) representation of package dependencies.

 In theory, packages can have cyclic dependencies.
 In that case, choose arbitrary edge and cut it.
 (`Pre-Depends` must not have cyclic dependencies.)

 Here, Graph uses index-managed node structures to avoid annoying lifetime managements.
*/

/*
 XXX
 Now, this Graph supports only single cyclic depth.
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
  #[error("Function is called with invalid Graph state.")]
  InvalidStateError,

  #[error("Failed to resolve pre-dependencies.")]
  FailedToResolve {
    pre_depended_on: String,
    pre_depending: String,
  },
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
  pub fn dfs_root(&mut self, root: &str) {
    // do DFS once starting from root node
    let root_ix = self
      .nodes
      .iter()
      .position(|node| node.package.name == root)
      .unwrap();
    self.dfs_internal(root_ix);
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
    // ignore already-visited nodes and unreachable nodes from root node
    if self.nodes[start].visited || self.nodes[start].normal_index == -1 {
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
  fn scc(&mut self, root: &str) -> Result<(), DagError> {
    self.clear_visited();
    self.check_before_scc_valid()?;

    // first, DFS in home-way order
    // NOTE: `node.normal_index` remains -1 if the node is unreachable from root node.
    self.dfs_root(root);

    // second, reverse DFS and make groups (just ignore unreachable node)
    self.clear_visited();
    for ix in (0..self.nodes.len()).rev() {
      if let Some(node_ix) = self
        .nodes
        .iter()
        .position(|node| node.normal_index == ix as i64)
      {
        if self.nodes[node_ix].group == -1 {
          self.rev_dfs_grouping(node_ix);
          self.group_num += 1;
        }
      }
    }

    Ok(())
  }

  fn check_before_scc_valid(&self) -> Result<(), DagError> {
    let graph_state_valid = self.index == 0 && self.group_num == 0;
    let packages_state_valid = self
      .nodes
      .iter()
      .all(|node| node.normal_index == -1 && node.group == -1 && !node.visited);

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
          let to_group = self.nodes[*to].group;
          if to_group != group_id && !simple_node.to.contains(&to_group) {
            simple_node.to.push(to_group);
          }
        }
      }
      result.push(simple_node);
    }

    self.simplified_nodes = result;
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
        None => continue, // just ignore when target is not found
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

// Check if pre-depended-on packages are placed after pre-depending packages.
#[allow(clippy::ptr_arg)]
fn sanity_check(deps: &Vec<PackageWithSource>) -> Result<(), DagError> {
  for (ix, package) in deps.iter().enumerate() {
    #[allow(clippy::ptr_arg)]
    let depended_ons: Vec<&DependsAnyOf> = package
      .package
      .depends
      .iter()
      .filter(|anyof| anyof.depends[0].dep_type == DepType::PreDepends)
      .collect();
    for depended_on in depended_ons {
      if let Some(depended_on_ix) = deps
        .iter()
        .position(|pws| pws.package.name == depended_on.depends[0].package)
      {
        if depended_on_ix < ix {
          return Err(DagError::FailedToResolve {
            pre_depended_on: depended_on.depends[0].package.to_string(),
            pre_depending: package.package.name.to_string(),
          });
        }
      }
    }
  }

  Ok(())
}

fn remove_normal_deps(packages: &mut Vec<Package>) {
  #[allow(clippy::needless_range_loop)]
  for ix in 0..packages.len() {
    let mut target_jx = vec![];
    for jx in 0..packages[ix].depends.len() {
      if packages[ix].depends[jx].depends[0].dep_type == DepType::Depends {
        target_jx.push(jx);
      }
    }
    for jx in target_jx.into_iter().rev() {
      packages[ix].depends.remove(jx);
    }
  }
}

// sort nodes in the same group respecting `Pre-Depends` dependency.
fn force_predepends_same_group(nodes: &mut Vec<&PackageNode>) {
  let mut orders = vec![];
  for (ix, node) in nodes.iter().enumerate() {
    if orders.contains(&ix) {
      continue;
    }
    for anyof in &node.package.depends {
      if anyof.depends[0].dep_type != DepType::PreDepends {
        continue;
      }
      // if the node has pre-depends in the same group, place pre-depended-on node after pre-depending node.
      if let Some(jx) = nodes
        .iter()
        .position(|node| node.package.name == anyof.depends[0].package)
      {
        orders.push(jx);
      }
    }

    let tmp = orders.clone();
    orders = vec![ix];
    orders.extend(tmp);
  }

  let mut results = vec![];
  for order in orders {
    #[allow(clippy::clone_double_ref)]
    results.push(nodes[order].clone());
  }

  nodes.clear();
  nodes.extend(results);
}

// Sort packages dependencies in topological way.
// NOTE: Caller must ensure that all necessary packages are included in `deps`.
//      If depended-on package is not found in `deps`, this function just ignores it.
//      (cuz it would happen when already-installed packages are removed from `deps`.)
// NOTE: result is returned in reversed-order of deps.
// NOTE: if there are unreachable nodes from `root` node, these nodes are omitted in returned vec.
fn sort_depends_internal(
  deps: HashSet<PackageWithSource>,
  root: &str,
  dep_type: DepType,
) -> Result<Vec<PackageWithSource>, DagError> {
  let mut packages: Vec<Package> = deps.iter().map(|pws| pws.package.clone()).collect();
  if dep_type == DepType::PreDepends {
    // remove all normal `Depends`.
    remove_normal_deps(&mut packages);
  }
  let mut graph = Graph::from(packages).unwrap();

  // do SCC to make a DAG and do topological sort
  graph.scc(root).unwrap();
  graph.topological_sort();

  let mut results = vec![];
  let group_orders = graph.topological_order;

  // NOTE: if target package itself has circular dependencies,
  //      at least the target package should be installed at the end.
  let target_group = graph
    .nodes
    .iter()
    .find(|node| node.package.name == root)
    .unwrap()
    .group;

  for group_order in 0..group_orders.len() {
    let group_id = group_orders
      .iter()
      .position(|i| *i == group_order as i64)
      .unwrap();
    if group_id as i64 == target_group {
      // ignore the group including target package
      continue;
    }
    let mut nodes: Vec<&PackageNode> = graph
      .nodes
      .iter()
      .filter(|node| node.group == group_id as i64)
      .collect();
    force_predepends_same_group(&mut nodes);

    // XXX In the same group, push nodes in arbitrary order
    for node in nodes {
      // remove unreachable nodes from root node
      if node.normal_index == -1 {
        continue;
      }
      // combine with source information
      let pws = deps
        .iter()
        .find(|pws| pws.package == node.package)
        .unwrap()
        .clone();
      results.push(pws);
    }
  }

  // push nodes in target group
  let mut target_results: Vec<PackageWithSource> = vec![];
  for node in graph.nodes.iter().filter(|node| node.group == target_group) {
    let pws = deps
      .iter()
      .find(|pws| pws.package == node.package)
      .unwrap()
      .clone();
    // if node is target package itself, add it at the end.
    if pws.package.name == root {
      let tmp = target_results.clone();
      target_results = vec![pws];
      target_results.extend(tmp);
    } else {
      target_results.push(pws);
    }
  }

  let sorted_deps = vec![target_results, results]
    .into_iter()
    .flatten()
    .collect();
  if dep_type == DepType::PreDepends {
    sanity_check(&sorted_deps)?;
  }
  Ok(sorted_deps)
}

pub fn sort_depends(
  deps: HashSet<PackageWithSource>,
  root: &str,
) -> Result<Vec<PackageWithSource>, DagError> {
  sort_depends_internal(deps, root, DepType::Depends)
}

// Returns layered packages.
// Packages in the same group should be extracted and configured in this order.
// NOTE: argument `pwss` must be in topological order, before reversed.
pub fn split_layers(pwss: &[PackageWithSource]) -> Vec<Vec<PackageWithSource>> {
  let mut layers = vec![];

  let mut depended_on_ixs = vec![];
  for pws in pwss.iter() {
    let package = &pws.package;
    for anyof in &package.depends {
      if anyof.depends[0].dep_type == DepType::PreDepends {
        if let Some(target) = pwss
          .iter()
          .position(|pws| pws.package.name == anyof.depends[0].package)
        {
          depended_on_ixs.push(target);
        }
      }
    }
  }

  let mut cur = vec![];
  for (ix, pws) in pwss.iter().enumerate() {
    if depended_on_ixs.contains(&ix) {
      layers.push(cur.clone());
      cur.clear();
      cur.push(pws.clone());
    } else {
      cur.push(pws.clone());
    }
  }
  if !cur.is_empty() {
    layers.push(cur);
  }

  layers
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
        depends: DependsAnyOf::from(&dep_strings.join(", "), DepType::Depends).unwrap(),
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

    graph.dfs_root("0");
    assert_eq!(to_orders(&graph), order);

    assert_eq!(graph.scc("0").is_err(), true);
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

    graph.scc("0").unwrap();
    let groups = to_groups(&graph);
    assert_eq!(groups, answer_groups);

    let topological_answer = vec![0, 1, 2, 3, 4, 5, 6];
    graph.topological_sort();
    assert_eq!(topological_answer, graph.topological_order);
  }
}
