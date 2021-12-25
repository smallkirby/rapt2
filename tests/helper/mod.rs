extern crate rapt2;

use rapt2::{package::package::*, source::source::*};

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

// Package contents in `tests/resources/lists/cache/test1_InRelease.list`.
pub fn package_list_test1() -> Vec<Package> {
  vec![
    Package {
      name: "vim".into(),
      arch: "amd64".into(),
      version: "2:8.1.2269-1ubuntu5".into(),
      priority: Some(Priority::OPTIONAL),
      section: Some("editors".into()),
      maintainer: "Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>".into(),
      filename: "pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb".into(),
      size: 1237624,
      md5: "198ccbb07a9fc8ebe67a213eab6a3e96".into(),
      sha1: "796c962d044f99a81b187211e6ce9a0a44b8d5d1".into(),
      sha256: "1e38f267bf4c06e424b166e8d666ffd6ce25c657012892d099651bee18a2c834".into(),
      short_description: "Vi IMproved - enhanced vi editor".into(),
      ..Default::default()
    },
    Package {
      name: "gcc".into(),
      arch: "amd64".into(),
      version: "4:9.3.0-1ubuntu2".into(),
      priority: Some(Priority::OPTIONAL),
      section: Some("devel".into()),
      maintainer: "Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>".into(),
      filename: "pool/main/g/gcc-defaults/gcc_9.3.0-1ubuntu2_amd64.deb".into(),
      size: 5208,
      md5: "c8434d667d10beb0f15ae2e175ead386".into(),
      sha1: "ac589aa5799c3705383a16679fd9e968bcc6385e".into(),
      sha256: "78ab6a8841c68300ba39992e8e33190371e133b3592c601ed3052d54e2ba59ea".into(),
      short_description: "GNU C compiler".into(),
      ..Default::default()
    },
  ]
}
