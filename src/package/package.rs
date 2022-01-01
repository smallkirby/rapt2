/*
 This file defines structure of Package file of a repository.
*/

use super::version::*;
use crate::dpkg::status::DpkgStatusArea;

use std::collections::HashSet;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Debug, Default, Eq, Clone)]
pub struct Package {
  // shared?
  pub name: String,
  pub version: Version,
  pub priority: Option<Priority>,
  pub arch: String,
  pub section: Option<String>,
  pub maintainer: String,
  pub filename: String,
  pub size: u64,
  pub short_description: String,
  pub long_description: Option<String>,
  pub depends: Vec<DependsAnyOf>,

  // package information only
  pub md5: String,
  pub sha1: String,
  pub sha256: String,

  // status information only
  pub conffiles: Vec<String>,
  pub status: Option<DpkgStatusArea>,
}

impl PartialEq for Package {
  // XXX
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}

// XXX use only `name` field for hashing
impl Hash for Package {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.name.hash(state);
  }
}

impl Package {
  pub fn valid(&self) -> bool {
    !self.name.is_empty()
      && self.size != 0
      && !self.maintainer.is_empty()
      && !self.filename.is_empty()
  }

  pub fn valid_as_source(&self) -> bool {
    !self.name.is_empty() && !self.maintainer.is_empty()
  }

  pub fn valid_as_status(&self) -> bool {
    !self.name.is_empty()
  }

  pub fn extend(a: &mut HashSet<Self>, b: HashSet<Self>) {
    let mut removal_targets = vec![];
    for a_ent in a.iter() {
      if let Some(b_ent) = b.get(a_ent) {
        if a_ent.version < b_ent.version {
          removal_targets.push(a_ent.clone());
        }
      }
    }

    for target in removal_targets {
      a.remove(&target);
    }
    a.extend(b);
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntryType {
  BINARY,
  SOURCE,
  STATUS,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum DepType {
  Depends,
  PreDepends,
}

impl Default for DepType {
  fn default() -> Self {
    Self::Depends
  }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct Depends {
  pub package: String,
  pub version: Option<VersionComp>,
  pub dep_type: DepType,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct DependsAnyOf {
  pub depends: Vec<Depends>,
}

impl DependsAnyOf {
  #[allow(clippy::result_unit_err)]
  pub fn from(s: &str, dep_type: DepType) -> Result<Vec<Self>, ()> {
    let mut results: Vec<Self> = vec![];
    let parts: Vec<&str> = s.trim().split(", ").collect();

    for part in parts {
      if part.is_empty() {
        continue;
      }
      let or_parts: Vec<&str> = part.split(" | ").collect();
      let mut any_of = vec![];

      for or_part in or_parts {
        match or_part.find('(') {
          // eg: "libc6 (> 2.14)"
          Some(ix) => {
            let package_name = &or_part[0..ix - 1]; // eg: "libc6"
            let version_str_tmp = &or_part[ix + 1..or_part.len() - 1]; // eg: "> 2.14""
            let depends = Depends {
              package: package_name.into(),
              version: Some(VersionComp::from(version_str_tmp).unwrap()),
              dep_type: dep_type.clone(),
            };
            any_of.push(depends);
          }
          None => {
            let depends = Depends {
              package: or_part.trim().into(),
              version: None,
              dep_type: dep_type.clone(),
            };
            any_of.push(depends);
          }
        };
      }

      results.push(Self { depends: any_of });
    }

    Ok(results)
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Priority {
  REQUIRED,
  IMPORTANT,
  STANDARD,
  OPTIONAL,
  EXTRA,
  UNKNOWN,
}

impl FromStr for Priority {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match s.to_lowercase().as_str() {
      "required" => Self::REQUIRED,
      "important" => Self::IMPORTANT,
      "standard" => Self::STANDARD,
      "optional" => Self::OPTIONAL,
      "extra" => Self::EXTRA,
      _ => Self::UNKNOWN,
    })
  }
}

impl Default for Priority {
  fn default() -> Self {
    Self::UNKNOWN
  }
}
