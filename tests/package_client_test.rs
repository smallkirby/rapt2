extern crate rapt2;

use rapt2::package::client::PackageClient;

use std::path::PathBuf;

mod helper;

#[test]
fn test_package_cache_read() {
  // read single file
  let answer = helper::package_list_test1();
  let mut client = PackageClient::new(PathBuf::from("./tests/resources/lists")).unwrap();
  let packages = client.read_single_file("test1_Packages").unwrap();
  assert_eq!(answer, packages);
}
