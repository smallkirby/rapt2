/*
 This file defines `sources.list` entry.
*/

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
#[derive(Debug, Eq, Clone)]
pub struct Source {
  pub archive_type: ArchivedType,
  pub url: String,
  pub distro: String,
  pub components: Vec<Component>,
}

impl Hash for Source {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.archive_type.hash(state);
    self.url.hash(state);
    self.distro.hash(state);
    self.components.clone().sort().hash(state);
  }
}

impl PartialEq for Source {
  fn eq(&self, other: &Self) -> bool {
    if !(self.archive_type == other.archive_type
      && self.url == other.url
      && self.distro == other.distro)
    {
      return false;
    }
    if self.components.len() != other.components.len() {
      return false;
    }
    for component in &self.components {
      if !other.components.contains(component) {
        return false;
      }
    }

    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn source_partial_eq() {
    // check if PartialEq trait is correctly implemented.
    let source1 = Source {
      archive_type: ArchivedType::DEB,
      url: "https://hogehoge.com/".into(),
      distro: "focal".into(),
      components: vec![Component::MAIN, Component::MULTIVERSE],
    };
    let source2 = Source {
      archive_type: ArchivedType::DEB,
      url: "https://fugafuga.com/".into(),
      distro: "focal".into(),
      components: vec![Component::UNIVERSE, Component::MAIN],
    };
    let source3 = Source {
      archive_type: ArchivedType::DEB,
      url: "https://fugafuga.com/".into(),
      distro: "focal".into(),
      components: vec![Component::MAIN, Component::UNIVERSE],
    };
    assert_ne!(source1, source2);
    assert_eq!(source2, source3);
  }
}
