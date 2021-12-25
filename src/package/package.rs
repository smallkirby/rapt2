use std::str::FromStr;

/*
 This file defines structure of Package file of a repository.
*/

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Package {
  pub name: String,
  pub version: String,
  pub priority: Option<Priority>,
  pub arch: String,
  pub section: Option<String>,
  pub maintainer: String,
  pub filename: String,
  pub size: u64,
  pub md5: String,
  pub sha1: String,
  pub sha256: String,
  pub short_description: String,
  pub long_description: Option<String>,
}

impl Package {
  pub fn valid(&self) -> bool {
    !self.name.is_empty() && self.size != 0 && !self.filename.is_empty()
  }
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
