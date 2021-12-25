extern crate rapt2;

use rapt2::source::{client::SourceClient, source::*};

use std::collections::HashSet;
use std::path::PathBuf;

mod helper;

#[test]
fn test_sourcelist_read() {
  // read single file
  let answer = helper::sources_list_sources();
  let client = SourceClient::new(PathBuf::from("./tests/resources/sources")).unwrap();
  let sources = client.read_single_file("sources.list").unwrap();
  assert_eq!(answer, sources);

  // read all files
  let answer: HashSet<Source> = vec![
    helper::sources_list_sources(),
    helper::sources_list_hoge(),
    helper::sources_list_fuga(),
  ]
  .into_iter()
  .flatten()
  .collect();
  let sources = client.read_all().unwrap();
  let sources_hashset: HashSet<Source> = sources.into_iter().collect();
  assert_eq!(sources_hashset.iter().all(|s| answer.contains(s)), true);
  assert_eq!(answer.iter().all(|s| sources_hashset.contains(s)), true);
}
