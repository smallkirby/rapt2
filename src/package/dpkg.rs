/*
 This file defines operations relating to dpkg.
*/

use super::{error::PackageError, package::*, parser};

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct DpkgClient {
  dpkg_dir: PathBuf,
}

impl DpkgClient {
  pub fn new(dpkg_dir: PathBuf) -> Self {
    Self { dpkg_dir }
  }

  pub fn get_installed_packages(&self) -> Result<HashSet<Package>, PackageError> {
    let dpkg_status_pathbuf = self.dpkg_dir.join("status");
    let dpkg_status_path = dpkg_status_pathbuf.as_path();
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
    &self,
    packages: &HashSet<Package>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let installed_packages = self.get_installed_packages()?;
    self.get_absolute_packages_internal(packages, installed_packages)
  }

  fn get_absolute_packages_internal(
    &self,
    news: &HashSet<Package>,
    installeds: HashSet<Package>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let mut results = vec![];

    for package in installeds {
      let candidate_new = match news.get(&package) {
        Some(candidate) => candidate,
        None => continue, // XXX installed, but no information in Packages files
      };
      if candidate_new.version > package.version {
        results.push(PackageStatus {
          package,
          status: StatusComp::OLD,
        });
      }
    }

    Ok(results)
  }
}

#[derive(Debug, PartialEq)]
pub enum StatusComp {
  NOTINSTALLED,
  OLD,
  UPTODATE,
}

#[derive(Debug)]
pub struct PackageStatus {
  package: Package,
  status: StatusComp,
}

#[cfg(test)]
mod tests {
  use super::super::client::PackageClient;
  use super::*;

  #[test]
  fn test_dpkg_status_is_readable() {
    let client = DpkgClient::new(PathBuf::from("/var/lib/dpkg"));
    client.get_installed_packages().unwrap();
  }

  #[test]
  fn test_dpkg_get_obsolute_packages() {
    let package_client = PackageClient::new(PathBuf::from("./tests/resources/lists")).unwrap();
    let packages = package_client
      .read_single_file("test2_InRelease.list")
      .unwrap();
    let dpkg_client = DpkgClient::new(PathBuf::from("./tests/resources/dpkg"));
    let obsolute_packages = dpkg_client.get_obsolute_packages(&packages).unwrap();
    assert_eq!(obsolute_packages.len(), 1);
    assert_eq!(obsolute_packages[0].status, StatusComp::OLD);
    assert_eq!(obsolute_packages[0].package.name, "vim");
  }
}
