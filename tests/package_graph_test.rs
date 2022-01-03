extern crate rapt2;

mod helper;

use rapt2::{algorithm::graph::Graph, package::client::PackageClient};

use std::path::PathBuf;

#[allow(dead_code)]
//#[test]
fn test_construct_graph() {
  let mut package_client = PackageClient::new(PathBuf::from("/var/lib/apt/lists")).unwrap();
  let packages = package_client
    .read_single_file("jp.archive.ubuntu.com_ubuntu_dists_focal-updates_main_binary-amd64_Packages")
    .unwrap();

  let _ = Graph::construct_graph(packages.into_iter().collect());
}
