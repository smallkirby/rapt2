/*
 This file constructs dependency graph for all given packages.

 XXX should change filename.
*/

use crate::{dpkg::status::DpkgStatusStatus, package::package::Package};

use std::collections::HashMap;

#[derive(Debug)]
struct Node {
  pub package: Package,
  pub to: Vec<usize>,
  pub revto: Vec<usize>,
  visited: bool,
  depending_on: bool,
  index: usize,
  hierarchy: usize,
}

// NOTE: node MUST NOT be removed cuz each node is index-managed.
pub struct Graph {
  nodes: HashMap<usize, Node>,
  current_hierarchy: usize,
}

impl Graph {
  // each package of `packages` should have valid `status`.
  pub fn construct_graph(packages: Vec<Package>) -> Self {
    let mut nodes = HashMap::new();
    for (ix, package) in packages.iter().enumerate() {
      nodes.insert(
        package.name.clone(),
        Node {
          package: package.clone(),
          to: vec![],
          revto: vec![],
          visited: false,
          index: ix,
          depending_on: false,
          hierarchy: 0,
        },
      );
    }

    for ix in 0..packages.len() {
      let deps: Vec<String> = packages[ix]
        .depends
        .iter()
        .map(|anyof| anyof.depends[0].package.clone())
        .collect();
      let depending_ix = nodes.get(&packages[ix].name).unwrap().index;
      for dep in deps {
        if let Some(depended_node) = nodes.get_mut(&dep) {
          depended_node.revto.push(depending_ix);

          let depended_ix = depended_node.index;
          let node = nodes.get_mut(&packages[ix].name).unwrap();
          node.to.push(depended_ix);
        }
      }
    }

    Graph {
      nodes: nodes
        .into_iter()
        .map(|(_, node)| (node.index, node))
        .collect(),
      current_hierarchy: 0,
    }
  }

  fn clear_visited(&mut self) {
    for (_, node) in &mut self.nodes {
      node.visited = false;
      node.depending_on = false;
      node.hierarchy = 0;
    }
  }

  pub fn is_depended_on(&mut self, root: &str) -> bool {
    // clear visited status
    self.clear_visited();

    let root = self
      .nodes
      .iter()
      .find(|(_, node)| node.package.name == root);
    // if root itself is not in Graph, return false
    if root.is_none() {
      return false;
    }

    // do DFS and mark depending-on packages
    let (_, root_node) = root.unwrap();
    for revto in root_node.revto.clone() {
      self.rev_dfs(revto);
    }

    // check there are any depending-on packages installed
    self.nodes.iter().any(|(_, node)| node.depending_on)
  }

  fn rev_dfs(&mut self, start: usize) {
    let target = self.nodes.get_mut(&start).unwrap();
    if target.visited {
      return;
    }
    target.visited = true;
    if target.package.status.is_none()
      || target.package.status.as_ref().unwrap().status != DpkgStatusStatus::Installed
    {
      return;
    }
    target.depending_on = true;

    for revto in target.revto.clone() {
      self.rev_dfs(revto);
    }
  }

  // get dependencies of `root` package with hierarchical structure.
  pub fn get_hierarchical_deps(&mut self, root: &str) -> Vec<Vec<Package>> {
    // clear visited state
    self.clear_visited();

    let root = self
      .nodes
      .iter()
      .find(|(_, node)| node.package.name == root);
    // if root itself is not in Graph, return false
    if root.is_none() {
      return vec![];
    }

    // do DFS
    let (_, root_node) = root.unwrap();
    let root_package = root_node.package.clone();
    for to in root_node.to.clone() {
      self.dfs(to);
    }

    // collect visited packages
    let visiteds: Vec<&Node> = self.nodes.values().filter(|node| node.visited).collect();
    let mut results = vec![vec![root_package]];
    let max_hierarchy = visiteds
      .iter()
      .max_by_key(|node| node.hierarchy)
      .unwrap()
      .hierarchy;
    for hierarchy in 1..=max_hierarchy {
      let mut current_packages = vec![];
      let targets: Vec<&&Node> = visiteds
        .iter()
        .filter(|node| node.hierarchy == hierarchy)
        .collect();
      for target in targets {
        current_packages.push(target.package.clone());
      }
      results.push(current_packages);
    }

    results
  }

  fn dfs(&mut self, start: usize) {
    self.current_hierarchy += 1;
    let target = self.nodes.get_mut(&start).unwrap();
    if target.visited {
      self.current_hierarchy -= 1;
      return;
    }
    target.visited = true;
    target.hierarchy = self.current_hierarchy;

    for to in target.to.clone() {
      self.dfs(to);
    }
    self.current_hierarchy -= 1;
  }
}
