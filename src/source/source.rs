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
  PARTNER,    // Canonical partners
  CONTRIB,    // DFSG-compliant, but have deps not in main.
  STABLE,
  NULL, // component is not given (refer to /docs/Source.md)
}

impl std::fmt::Display for Component {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MAIN => write!(f, "main"),
      Self::RESTRICTED => write!(f, "restricted"),
      Self::UNIVERSE => write!(f, "universe"),
      Self::MULTIVERSE => write!(f, "multiverse"),
      Self::PARTNER => write!(f, "partner"),
      Self::CONTRIB => write!(f, "contrib"),
      Self::STABLE => write!(f, "stable"),
      Self::NULL => write!(f, "(empty)"),
    }
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
      "partner" => Ok(Self::PARTNER),
      "contrib" => Ok(Self::CONTRIB),
      "stable" => Ok(Self::STABLE),
      _ => Ok(Self::NULL),
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

  pub fn inrelease_url(&self) -> String {
    let mut url = self.url.as_str();
    let mut distro = self.distro.as_str();

    if url.ends_with('/') {
      let mut tmp = url.chars();
      tmp.next_back().unwrap();
      url = tmp.as_str();
    }
    if distro.ends_with('/') {
      let mut tmp = distro.chars();
      tmp.next_back().unwrap();
      distro = tmp.as_str();
    }

    format!("{}/dists/{}/InRelease", url, distro)
  }

  pub fn packages_url(&self) -> String {
    let mut url = self.url.as_str();
    let mut distro = self.distro.as_str();

    if url.ends_with('/') {
      let mut tmp = url.chars();
      tmp.next_back().unwrap();
      url = tmp.as_str();
    }
    if distro.ends_with('/') {
      let mut tmp = distro.chars();
      tmp.next_back().unwrap();
      distro = tmp.as_str();
    }
    let type_str = match self.archive_type {
      ArchivedType::DEB => "binary-amd64",
      ArchivedType::DEBSRC => "source",
    };
    let filename = match self.archive_type {
      ArchivedType::DEB => "Packages",
      ArchivedType::DEBSRC => "Sources",
    };
    if self.component == Component::NULL {
      format!("{}/{}/{}.gz", url, distro, filename,)
    } else {
      format!(
        "{}/dists/{}/{}/{}/{}.gz",
        url,
        distro,
        self.component.to_string(),
        type_str,
        filename,
      )
    }
  }

  pub fn cache_filename(&self) -> String {
    let text = String::from(self.packages_url().split("://").collect::<Vec<&str>>()[1]);
    text.replace("/", "_")[..text.len() - 3].into()
  }

  pub fn inrelease_filename(&self) -> String {
    let text = String::from(self.inrelease_url().split("://").collect::<Vec<&str>>()[1]);
    text.replace("/", "_").into()
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
