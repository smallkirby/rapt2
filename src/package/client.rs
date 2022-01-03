/*
 This file implements an IO reader of Package files.
*/

/*
 NOTE: All methods of `PackageClient` must NOT read each package list files more than once.
 XXX: Maybe, it should use lazy_static cache member to gurantee that one client reads
     package list files only onece, upon each methods.
*/

use super::package::EntryType;
use super::{error::PackageError, package::Package, parser};
use crate::dpkg::client::{DpkgClient, StatusComp};
use crate::source::source::{ArchivedType, Source};

use glob;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PackageClient {
  cache_dir: PathBuf, // package cache dir
}

impl PackageClient {
  pub fn new(cache_dir: PathBuf) -> Result<Self, PackageError> {
    let path = Path::new(&cache_dir);
    if !path.exists() || !path.is_dir() {
      Err(PackageError::FileNotFound {
        target: path.to_string_lossy().to_string(),
      })
    } else {
      Ok(Self { cache_dir })
    }
  }

  // read a single list file.
  // `filename` is relative filename to `self.cache_dir`.
  pub fn read_single_file(&self, filename: &str) -> Result<HashSet<Package>, PackageError> {
    let content = self.read_single_file_raw(filename)?;
    parser::parse_entries_as_binary(&content) // XXX
  }

  pub fn read_single_file_raw(&self, filename: &str) -> Result<String, PackageError> {
    let pathbuf = self.cache_dir.join(filename);
    let path = pathbuf.as_path();

    if !path.exists() || !path.is_file() {
      return Err(PackageError::FileNotFound {
        target: path.to_str().unwrap().into(),
      });
    }

    let content = fs::read_to_string(path)?;
    Ok(content)
  }

  pub fn read_all_from_source(&self, sources: &[Source]) -> Result<HashSet<Package>, PackageError> {
    let mut results = HashSet::new();

    for source in sources {
      // ignore error cuz lists file contains unreadable files such as `lock`.
      if let Ok(packages) = self.read_single_file(&source.cache_filename()) {
        results.extend(packages);
      }
    }

    Ok(results)
  }

  pub fn read_all(&self) -> Result<HashSet<Package>, PackageError> {
    let mut results = HashSet::new();
    let base = self.cache_dir.as_path();
    let files = fs::read_dir(base)?;
    for target in files.flatten() {
      if !target.file_type().unwrap().is_file() {
        continue;
      }
      let path = target.path();
      if !path
        .as_path()
        .to_string_lossy()
        .to_string()
        .ends_with("Packages")
      {
        continue;
      }

      let filename = match path.as_path().file_name() {
        Some(s) => s.to_string_lossy().to_string(),
        None => continue,
      };
      // ignore error cuz lists file contains unreadable files such as `lock`.
      if let Ok(packages) = self.read_single_file(&filename) {
        results.extend(packages);
      }
    }

    Ok(results)
  }

  // search packages from list DB by package name.
  // glob pattern is supported for search.
  // NOTE: if multiple packages with same name found, the first one is returned.
  pub fn search_by_name(&self, name: &str) -> Result<HashSet<Package>, PackageError> {
    let mut results = HashSet::new();
    let pattern = match glob::Pattern::new(name) {
      Ok(pattern) => pattern,
      Err(_) => return Err(PackageError::InvalidPackageName { name: name.into() }),
    };

    let packages = self.read_all()?;
    for package in packages {
      if pattern.matches(&package.name) {
        results.insert(package);
      }
    }

    Ok(results)
  }

  pub fn read_all_from_source_with_source(
    &self,
    sources: &[Source],
  ) -> Result<HashSet<PackageWithSource>, PackageError> {
    let mut results: HashSet<PackageWithSource> = HashSet::new();

    for source in sources {
      // ignore error cuz lists file contains unreadable files such as `lock`.
      if let Ok(packages) = self.read_single_file(&source.cache_filename()) {
        let packages_with_source: Vec<PackageWithSource> = packages
          .into_iter()
          .map(|package| PackageWithSource {
            package,
            source: source.clone(),
            dpkg_status: None,
          })
          .collect();
        for package_with_source in packages_with_source {
          if results.contains(&package_with_source) {
            let existing = results.get(&package_with_source).unwrap().clone();
            if existing.package.version < package_with_source.package.version {
              results.remove(&existing); // must remove first
              results.insert(package_with_source);
            }
          } else {
            results.insert(package_with_source);
          }
        }
      }
    }

    Ok(results)
  }

  // search packages from list DB by package name.
  // Returns found packages with Source info.
  pub fn search_by_name_with_source(
    &self,
    name: &str,
    sources: Vec<Source>,
  ) -> Result<HashSet<PackageWithSource>, PackageError> {
    let pattern = match glob::Pattern::new(name) {
      Ok(pattern) => pattern,
      Err(_) => return Err(PackageError::InvalidPackageName { name: name.into() }),
    };
    let sources: Vec<Source> = sources
      .into_iter()
      .filter(|source| source.archive_type == ArchivedType::DEB)
      .collect();

    let packages = self.read_all_from_source_with_source(&sources)?;
    Ok(
      packages
        .into_iter()
        .filter(|package_with_source| pattern.matches(&package_with_source.package.name))
        .collect(),
    )
  }

  // Get target packages and all of its dependencies with Source information.
  // Result is returned in flattened HashSet.
  pub fn get_package_with_deps(
    &self,
    name: &str,                                      // target package
    #[allow(clippy::ptr_arg)] sources: &Vec<Source>, // sources to search for packages
    ignore_installed: bool,                          // ignore already installed packages
    mut dpkg_client: Option<&mut DpkgClient>,        // needed if `ignore_installed` is true
  ) -> Result<HashSet<PackageWithSource>, PackageError> {
    let pattern = glob::Pattern::new(name).unwrap();
    let packages_with_source = self.read_all_from_source_with_source(sources)?;

    // first, find target package itself
    let mut target_package_ws = match packages_with_source
      .iter()
      .find(|package_ws| pattern.matches(&package_ws.package.name))
    {
      Some(target) => target.clone(),
      None => {
        return Err(PackageError::PackageNotFound {
          package_name: name.into(),
        })
      }
    };

    // check target itself is already installed
    let mut deps: HashSet<PackageWithSource> = HashSet::new();
    if !ignore_installed {
      match dpkg_client
        .as_mut()
        .unwrap()
        .check_installed_status(&target_package_ws.package)?
      {
        StatusComp::UPTODATE => {
          return Ok(HashSet::new());
        }
        status => target_package_ws.dpkg_status = Some(status),
      };
    }

    // next, find all its dependencies recursively
    deps.insert(target_package_ws.clone());
    self.get_dependency_recursive(
      &target_package_ws,
      &packages_with_source,
      &mut deps,
      ignore_installed,
      &mut dpkg_client,
    )?;

    Ok(deps)
  }

  fn get_dependency_recursive(
    &self,
    target: &PackageWithSource,
    all_packages_ws: &HashSet<PackageWithSource>,
    acc: &mut HashSet<PackageWithSource>,
    ignore_installed: bool, // ignore already installed packages
    dpkg_client: &mut Option<&mut DpkgClient>,
  ) -> Result<(), PackageError> {
    for dep in &target.package.depends {
      // XXX choose arbitrary dependencie
      let dep = &dep.depends[0];
      // XXX it now choose first found depended-on package with same name.
      // maybe, should choose latest version.
      if acc.iter().any(|pws| pws.package.name == dep.package) {
        continue;
      }

      let depended_on = match all_packages_ws
        .iter()
        .find(|pws| pws.package.name == dep.package)
      {
        Some(target) => {
          if !ignore_installed {
            match dpkg_client
              .as_mut()
              .unwrap()
              .check_installed_status(&target.package)?
            {
              StatusComp::UPTODATE => continue,
              status => PackageWithSource {
                package: target.package.clone(),
                source: target.source.clone(),
                dpkg_status: Some(status),
              },
            }
          } else {
            target.clone()
          }
        }
        None => {
          return Err(PackageError::PackageNotFound {
            package_name: dep.package.to_string(),
          })
        }
      };

      // append depended-on package
      acc.insert(depended_on.clone());

      // more search recursively
      self.get_dependency_recursive(
        &depended_on,
        all_packages_ws,
        acc,
        ignore_installed,
        dpkg_client,
      )?;
    }

    Ok(())
  }

  // Remove all files named "*.deb" in `archive_dir`.
  // returns the number of removed binary files.
  pub fn remove_deb_caches(&self, archive_dir: &Path) -> Result<i32, PackageError> {
    if !archive_dir.is_dir() {
      return Err(PackageError::FileNotFound {
        target: archive_dir.to_string_lossy().to_string(),
      });
    }

    let mut removed_count = 0;
    // enumerate *.deb files and remove it
    for result in archive_dir.read_dir()? {
      if let Ok(entry) = result {
        if !entry.file_type()?.is_file() {
          continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".deb") {
          fs::remove_file(entry.path())?;
          removed_count += 1;
        }
      }
    }

    Ok(removed_count)
  }
}

#[derive(Debug, Eq, Clone)]
pub struct PackageWithSource {
  pub package: Package,
  pub source: Source,
  pub dpkg_status: Option<StatusComp>,
}

// hash only by its package
impl std::hash::Hash for PackageWithSource {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.package.hash(state);
  }
}

impl PartialEq for PackageWithSource {
  fn eq(&self, other: &Self) -> bool {
    self.package == other.package
  }
}

pub fn to_packages(content: &str, entry_type: EntryType) -> Result<HashSet<Package>, PackageError> {
  match entry_type {
    EntryType::BINARY => parser::parse_entries_as_binary(content),
    EntryType::SOURCE => parser::parse_entries_as_source(content),
    EntryType::STATUS => parser::parse_entries_as_status(content),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_search_by_name() {
    let client = PackageClient::new(PathBuf::from("./tests/resources/lists")).unwrap();

    let result = client.search_by_name("vi*").unwrap();
    assert_eq!(result.len(), 1);
    let vim = result.into_iter().next().unwrap();
    assert_eq!(vim.name, "vim");

    let result = client.search_by_name("?cc").unwrap();
    assert_eq!(result.len(), 1);
    let gcc = result.into_iter().next().unwrap();
    assert_eq!(gcc.name, "gcc");
  }
}
