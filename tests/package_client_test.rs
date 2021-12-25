extern crate rapt2;

use rapt2::package::{client::PackageClient, package::*};

use std::path::PathBuf;

mod helper;

#[test]
fn test_package_cache_read() {
  // read single file
  let answer = helper::package_list_test1();
  let client = PackageClient::new(PathBuf::from("./tests/resources/lists")).unwrap();
  let packages = client.read_single_file("test1_InRelease.list").unwrap();
  assert_eq!(answer, packages);
}
