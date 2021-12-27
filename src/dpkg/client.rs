/*
 This file defines operations relating to dpkg.
*/

use super::status::*;
use crate::package::{error::PackageError, package::*, parser, version};

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

  // Get packages which are:
  //    - installed but but have older version
  //    - not installed
  // Returned `package` is old one.
  pub fn get_obsolute_packages(
    &self,
    packages: &HashSet<Package>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let installed_packages = self.get_installed_packages()?;
    self.get_obsolute_packages_internal(packages, installed_packages)
  }

  fn get_obsolute_packages_internal(
    &self,
    news: &HashSet<Package>,
    installeds: HashSet<Package>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let mut results = vec![];
    let installeds: HashSet<Package> = installeds
      .into_iter()
      .filter(|package| {
        if let Some(status) = &package.status {
          status.status == DpkgStatusStatus::Installed
        } else {
          false
        }
      })
      .collect();

    for package in installeds {
      let candidate_new = match news.get(&package) {
        Some(candidate) => candidate,
        None => continue, // XXX installed, but no information in Packages files
      };
      if candidate_new.version > package.version {
        results.push(PackageStatus {
          package,
          status: StatusComp::OLD,
          new_version: Some(candidate_new.version.clone()),
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

// Result of comparision between installed/new packages.
// NOTE: this struct is not related to `Status Area` of dpkg status.
#[derive(Debug)]
pub struct PackageStatus {
  pub package: Package,
  pub status: StatusComp,
  pub new_version: Option<version::Version>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::package::client::PackageClient;

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
