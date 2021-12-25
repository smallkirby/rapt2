use std::str::FromStr;

/*
 This file defines structure of Package file of a repository.
*/

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Package {
  // shared?
  pub name: String,
  pub version: String,
  pub priority: Option<Priority>,
  pub arch: String,
  pub section: Option<String>,
  pub maintainer: String,
  pub filename: String,
  pub size: u64,
  pub short_description: String,
  pub long_description: Option<String>,

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

#[derive(Debug, Default)]
pub struct PackageVersion {
  pub package: String,
  pub version: String,
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
