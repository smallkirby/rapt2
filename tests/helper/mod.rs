extern crate rapt2;

use rapt2::source::source::*;

// Source contents in `tests/resources/sources.list`.
pub fn sources_list_sources() -> Vec<Source> {
  let s1 = Source {
    archive_type: ArchivedType::DEBSRC,
    url: "http://archive.ubuntu.com/ubuntu".into(),
    distro: "focal".into(),
    components: vec![Component::MAIN, Component::RESTRICTED],
  };
  let s2 = Source {
    archive_type: ArchivedType::DEB,
    url: "http://jp.archive.ubuntu.com/ubuntu/".into(),
    distro: "focal".into(),
    components: vec![Component::MAIN, Component::RESTRICTED],
  };
  let s3 = Source {
    archive_type: ArchivedType::DEBSRC,
    url: "http://jp.archive.ubuntu.com/ubuntu/".into(),
    distro: "focal".into(),
    components: vec![
      Component::MAIN,
      Component::RESTRICTED,
      Component::MULTIVERSE,
      Component::UNIVERSE,
    ],
  };

  vec![s1, s2, s3]
}

// Source contents in `tests/resources/sources.list.d/hoge.list`.
pub fn sources_list_hoge() -> Vec<Source> {
  vec![Source {
    archive_type: ArchivedType::DEB,
    url: "http://jp.archive.ubuntu.com/ubuntu/".into(),
    distro: "focal-updates".into(),
    components: vec![Component::MULTIVERSE],
  }]
}

// Source contents in `tests/resources/sources.list.d/fuga.list`.
pub fn sources_list_fuga() -> Vec<Source> {
  vec![Source {
    archive_type: ArchivedType::DEB,
    url: "http://jp.archive.ubuntu.com/ubuntu/".into(),
    distro: "focal-backports".into(),
    components: vec![Component::MAIN],
  }]
}
