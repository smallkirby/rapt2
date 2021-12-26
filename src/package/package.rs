/*
 This file defines structure of Package file of a repository.
*/

use super::version::*;

use std::str::FromStr;

#[derive(Debug, Default, PartialEq, Eq, Hash)]
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
}

impl Package {
  pub fn valid(&self) -> bool {
    !self.name.is_empty() && self.size != 0 && !self.filename.is_empty()
  }

  pub fn valid_as_status(&self) -> bool {
    !self.name.is_empty()
  }
}

#[derive(Clone)]
pub enum EntryType {
  FULL,
  STATUS,
}

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub struct Depends {
  pub package: String,
  pub version: Option<VersionComp>,
}

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub struct DependsAnyOf {
  pub depends: Vec<Depends>,
}

impl DependsAnyOf {
  pub fn from(s: &str) -> Result<Vec<Self>, ()> {
    let mut results: Vec<Self> = vec![];
    let parts: Vec<&str> = s.trim().split(", ").collect();

    for part in parts {
      let or_parts: Vec<&str> = part.split(" | ").collect();
      let mut any_of = vec![];

      for or_part in or_parts {
        match or_part.find("(") {
          // eg: "libc6 (> 2.14)"
          Some(ix) => {
            let package_name = &or_part[0..ix - 1]; // eg: "libc6"
            let version_str_tmp = &or_part[ix + 1..or_part.len() - 1]; // eg: "> 2.14""
            let depends = Depends {
              package: package_name.into(),
              version: Some(VersionComp::from(version_str_tmp).unwrap()),
            };
            any_of.push(depends);
          }
          None => {
            let depends = Depends {
              package: or_part.trim().into(),
              version: None,
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

#[derive(Debug, PartialEq, Eq, Hash)]
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
