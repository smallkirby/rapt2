/*
 This file defines `sources.list` entry.
*/

use std::collections::HashSet;
use std::hash::Hash;
use std::str::FromStr;

// archive type of the source.
#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum ArchivedType {
  DEB,    // binary package
  DEBSRC, // source package
}

impl FromStr for ArchivedType {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "deb" => Ok(Self::DEB),
      "deb-src" => Ok(Self::DEBSRC),
      _ => Err(()),
    }
  }
}

#[derive(PartialEq, Debug, Eq, Hash, Clone, PartialOrd, Ord)]
pub enum Component {
  MAIN,       // free software, fully supported by Ubuntu
  RESTRICTED, // proprietary
  UNIVERSE,   // free, open, but not guranteed
  MULTIVERSE, // (might) not free
}

impl Component {
  pub fn to_string(&self) -> String {
    match self {
      Self::MAIN => "main",
      Self::RESTRICTED => "restricted",
      Self::UNIVERSE => "universe",
      Self::MULTIVERSE => "multiverse",
    }
    .into()
  }
}

impl FromStr for Component {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "main" => Ok(Self::MAIN),
      "restricted" => Ok(Self::RESTRICTED),
      "universe" => Ok(Self::UNIVERSE),
      "multiverse" => Ok(Self::MULTIVERSE),
      _ => Err(()),
    }
  }
}

// entry of `sources.list`.
#[derive(Debug, Eq, Clone, PartialEq, Hash)]
pub struct Source {
  pub archive_type: ArchivedType,
  pub url: String,
  pub distro: String,
  pub component: Component,
}

impl Source {
  pub fn from(
    archive_type: ArchivedType,
    url: &str,
    distro: &str,
    components: Vec<Component>,
  ) -> HashSet<Self> {
    components
      .iter()
      .map(|component| Self {
        archive_type: archive_type.clone(),
        url: url.into(),
        distro: distro.into(),
        component: component.clone(),
      })
      .collect()
  }

  pub fn packages_url(&self) -> String {
    let mut url = self.url.as_str();
    if url.ends_with("/") {
      let mut tmp = url.chars();
      tmp.next_back().unwrap();
      url = tmp.as_str();
    }
    let type_str = match self.archive_type {
      ArchivedType::DEB => "binary-amd64",
      ArchivedType::DEBSRC => "source",
    };
    let filename = match self.archive_type {
      ArchivedType::DEB => "Packages",
      ArchivedType::DEBSRC => "Sources",
    };
    format!(
      "{}/dists/{}/{}/{}/{}.gz",
      url,
      self.distro,
      self.component.to_string(),
      type_str,
      filename,
    )
  }

  pub fn cache_filename(&self) -> String {
    let text = String::from(self.packages_url().split("://").collect::<Vec<&str>>()[1]);
    text.replace("/", "_")[..text.len() - 3].into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn source_partial_eq() {
    // check if PartialEq trait is correctly implemented.
    let source1 = Source::from(
      ArchivedType::DEB,
      "https://hogehoge.com/",
      "focal",
      vec![Component::MAIN, Component::MULTIVERSE],
    );
    let source2 = Source::from(
      ArchivedType::DEB,
      "https://fugafuga.com/",
      "focal",
      vec![Component::UNIVERSE, Component::MAIN],
    );
    let source3 = Source::from(
      ArchivedType::DEB,
      "https://fugafuga.com/",
      "focal",
      vec![Component::MAIN, Component::UNIVERSE],
    );

    assert_ne!(source1, source2);
    assert_eq!(source2, source3);
  }
}
