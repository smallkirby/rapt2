extern crate rapt2;

mod helper;

use rapt2::{package::client::PackageClient, source::source::*};

use std::path::PathBuf;

#[test]
fn test_search_deps() {
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
  let deps = client.get_package_with_deps("0", &vec![source]).unwrap();

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
}
