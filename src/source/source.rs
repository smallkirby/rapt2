/*
 This file defines `sources.list` entry.
*/

use std::str::FromStr;

// archive type of the source.
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
pub struct Source {
  pub archive_type: ArchivedType,
  pub url: String,
  pub distro: String,
  pub components: Vec<Component>,
}
