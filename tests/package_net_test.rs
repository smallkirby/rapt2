extern crate rapt2;

use rapt2::{
  net::package::PackageDownloadClient,
  source::{
    client::SourceClient,
    source::{ArchivedType, Source},
  },
};

use std::path::PathBuf;

mod helper;

//#[test]
#[allow(dead_code)]
fn test_inrelease_net() {
  let source_client = SourceClient::new(PathBuf::from("./tests/resources/sources")).unwrap();
  let sources: Vec<Source> = source_client
    .read_single_file("sources.list")
    .unwrap()
    .into_iter()
    .collect();
  let sources = sources
    .into_iter()
    .filter(|source| source.archive_type == ArchivedType::DEB)
    .collect();

  let mut package_client =
    PackageDownloadClient::new(sources, PathBuf::from("./rapt2/lists")).unwrap();
  // update InRelease
  loop {
    if package_client.get_inrelease_ifneed().unwrap() == false {
      break;
    }
  }
  println!("{:?}", package_client);
}
