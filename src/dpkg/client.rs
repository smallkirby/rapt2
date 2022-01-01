/*
 This file defines operations relating to dpkg.
*/

use super::status::*;
use crate::apt::extended_states;
use crate::package::{error::PackageError, package::*, parser, version};

use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// Dpkg IO client.
// It ensures that dpkg status file is read only once for each `DpkgClient` by using `OnceCell`.
pub struct DpkgClient {
  dpkg_dir: PathBuf,
  dpkg_package_cache: OnceCell<HashSet<Package>>,
}

impl DpkgClient {
  pub fn new(dpkg_dir: PathBuf) -> Self {
    Self {
      dpkg_dir,
      dpkg_package_cache: OnceCell::new(),
    }
  }

  fn is_cache_initiated(&self) -> bool {
    self.dpkg_package_cache.get().is_some()
  }

  // Initiate dpkg package cache
  fn harvest_dpkg_package_cache(&mut self) -> Result<(), PackageError> {
    if self.is_cache_initiated() {
      Ok(())
    } else {
      let dpkg_status_pathbuf = self.dpkg_dir.join("status");
      let dpkg_status_path = dpkg_status_pathbuf.as_path();
      if dpkg_status_path.is_file() {
        let packages = parser::parse_entries_as_status(&fs::read_to_string(dpkg_status_path)?)?;
        self.dpkg_package_cache.set(packages).unwrap();
      } else {
        return Err(PackageError::FileNotFound {
          target: dpkg_status_path.to_string_lossy().to_string(),
        });
      }

      Ok(())
    }
  }

  pub fn get_installed_packages(&mut self) -> Result<HashSet<Package>, PackageError> {
    if !self.is_cache_initiated() {
      self.harvest_dpkg_package_cache()?;
    }
    Ok(self.dpkg_package_cache.get().unwrap().clone())
  }

  // Get packages which are:
  //    - installed but but have older version
  //    - not installed
  // Returned `package` is old one.
  pub fn get_obsolute_packages(
    &mut self,
    packages: &HashSet<Package>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let installed_packages = self.get_installed_packages()?;
    let extended_info_client = extended_states::AptExtendedStates::new();
    let extended_info = extended_info_client.get()?;
    self.get_obsolute_packages_internal(packages, installed_packages, extended_info)
  }

  fn get_obsolute_packages_internal(
    &self,
    news: &HashSet<Package>,
    installeds: HashSet<Package>,
    extended_info: Vec<extended_states::AptExtendedPackageInfo>,
  ) -> Result<Vec<PackageStatus>, PackageError> {
    let mut results = vec![];

    // get only installed state packages.
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

    // check its status by `/var/lib/dpkg/status`.
    for package in &installeds {
      let candidate_new = match news.get(package) {
        Some(candidate) => candidate,
        None => continue, // XXX installed, but no information in Packages files
      };
      if candidate_new.version > package.version {
        results.push(PackageStatus {
          package: package.clone(),
          status: StatusComp::OLD(package.version.clone()),
          new_version: Some(candidate_new.version.clone()),
        });
      }
    }

    // get only not automatically installed packages
    let results: Vec<PackageStatus> = results
      .into_iter()
      .filter(|package_status| {
        match extended_info
          .iter()
          .find(|ex| ex.name == package_status.package.name)
        {
          Some(info) => !info.automatic_installed,
          None => true,
        }
      })
      .collect();

    Ok(results)
  }

  pub fn check_installed_status(&mut self, target: &Package) -> Result<StatusComp, PackageError> {
    let installeds = self.get_installed_packages()?;
    match installeds
      .iter()
      .find(|installed| installed.name == target.name)
    {
      Some(package) => {
        // check only installed packages
        if let Some(status) = &package.status {
          if status.status != DpkgStatusStatus::Installed {
            return Ok(StatusComp::NOTINSTALLED);
          }
        }
        // compare version
        if package.version < target.version {
          Ok(StatusComp::OLD(package.version.clone()))
        } else {
          Ok(StatusComp::UPTODATE)
        }
      }
      None => Ok(StatusComp::NOTINSTALLED),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StatusComp {
  NOTINSTALLED,
  OLD(version::Version),
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
  use crate::package::{client::PackageClient, version::Version};

  #[test]
  fn test_dpkg_status_is_readable() {
    let mut client = DpkgClient::new(PathBuf::from("/var/lib/dpkg"));
    client.get_installed_packages().unwrap();
  }

  //#[test]
  #[allow(dead_code)]
  fn test_dpkg_get_obsolute_packages() {
    let package_client = PackageClient::new(PathBuf::from("./tests/resources/lists")).unwrap();
    let packages = package_client.read_single_file("test2_Packages").unwrap();
    let mut dpkg_client = DpkgClient::new(PathBuf::from("./tests/resources/dpkg"));
    let obsolute_packages = dpkg_client.get_obsolute_packages(&packages).unwrap();
    assert_eq!(obsolute_packages.len(), 1);
    assert_eq!(
      obsolute_packages[0].status,
      StatusComp::OLD(Version::from("2:8.1.2269-1ubuntu5").unwrap())
    );
    assert_eq!(obsolute_packages[0].package.name, "vim");
  }
}
