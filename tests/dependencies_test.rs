extern crate rapt2;

mod helper;

use rapt2::{
  algorithm::dag::{sort_depends, split_layers},
  package::client::PackageClient,
  source::source::*,
};

use std::path::PathBuf;

#[test]
fn test_resolve_deps() {
  /*
     0────►1 ────┐
           ▲     │
           │     ▼
           │     2 ──────►4 ─────►5     9
           │     │        │       │     ▲ │
           │     │        │       │     │ │
           3 ◄───┘        ▼       ▼     │ ▼
                         6 ─────►7 ────►8
  */
  let source = Source {
    archive_type: ArchivedType::DEB,
    url: "http://test3".into(),
    distro: "/".into(),
    component: Component::NULL,
  };
  let client = PackageClient::new(PathBuf::from("tests/resources/lists")).unwrap();
  let deps = client
    .get_package_with_deps("0", &vec![source], true, None)
    .unwrap();

  // check if dependencies are correctly gathered from list files.
  let package_names: Vec<String> = (0..=9).into_iter().map(|n| n.to_string()).collect();
  assert_eq!(package_names.len(), deps.len());
  for package_name in package_names {
    if deps
      .iter()
      .find(|pws| pws.package.name == package_name)
      .is_none()
    {
      panic!("Not found: {}", package_name);
    }
  }

  // check if deps are correctly resolved using topological sort
  let sorted_deps = sort_depends(deps, "0").unwrap();
  assert_eq!(10, sorted_deps.len());

  // order of nodes in the same group is undefined.
  // so check all possibility
  assert_eq!(sorted_deps[0].package.name, "0");
  let group1: Vec<String> = sorted_deps[1..=3]
    .iter()
    .map(|node| node.package.name.to_string())
    .collect();
  let group1_answers: Vec<String> = (1..=3).into_iter().map(|n| n.to_string()).collect();
  assert_eq!(true, group1.iter().all(|g| group1_answers.contains(g)));
  assert_eq!(sorted_deps[4].package.name, "4");
  let group45: Vec<String> = sorted_deps[5..=6]
    .iter()
    .map(|node| node.package.name.to_string())
    .collect();
  let group45_answers: Vec<String> = (5..=6).into_iter().map(|n| n.to_string()).collect();
  assert_eq!(true, group45.iter().all(|g| group45_answers.contains(g)));
  assert_eq!(sorted_deps[7].package.name, "7");
  let group6: Vec<String> = sorted_deps[8..=9]
    .iter()
    .map(|node| node.package.name.to_string())
    .collect();
  let group6_answers: Vec<String> = (8..=9).into_iter().map(|n| n.to_string()).collect();
  assert_eq!(true, group6.iter().all(|g| group6_answers.contains(g)));
}

#[test]
fn test_predeps() {
  /*
           strong
   ┌─────────────────────┐
   ├─────────────────────┤
   │                     │
   │                     │
   │                     ▼
   1────────►2          3 ────────► 4
  */

  let source = Source {
    archive_type: ArchivedType::DEB,
    url: "http://test4".into(),
    distro: "/".into(),
    component: Component::NULL,
  };
  let client = PackageClient::new(PathBuf::from("tests/resources/lists")).unwrap();
  let deps = client
    .get_package_with_deps("1", &vec![source], true, None)
    .unwrap();

  let sorted_deps = sort_depends(deps.clone(), "1").unwrap();
  let layers = split_layers(&sorted_deps);
  assert_eq!(layers.len(), 2);
  if layers[0].len() == 1 {
    assert_eq!(layers[0][0].package.name, "1");
    assert_eq!(layers[1][0].package.name, "3");
  } else if layers[0].len() == 2 {
    assert_eq!(layers[0][1].package.name, "2");
    assert_eq!(layers[1][0].package.name, "3");
  } else {
    panic!();
  }
}
