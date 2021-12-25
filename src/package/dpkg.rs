/*
 This file defines operations relating to dpkg.
*/

use super::{error::PackageError, package::*, parser};

use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn get_installed_packages() -> Result<HashSet<Package>, PackageError> {
  let dpkg_status_path = Path::new("/var/lib/dpkg/status");
  if dpkg_status_path.is_file() {
    parser::parse_entries_as_status(&fs::read_to_string(dpkg_status_path)?)
  } else {
    Err(PackageError::FileNotFound {
      target: dpkg_status_path.to_string_lossy().to_string(),
    })
  }
}

// get packages which are:
//    - installed but but have older version
//    - not installed
pub fn get_obsolute_packages(
  packages: &HashSet<Package>,
) -> Result<HashSet<Package>, PackageError> {
  let installed_packages = get_installed_packages()?;

  unimplemented!()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dpkg_status_is_readable() {
    get_installed_packages().unwrap();
  }
}
