/*
 This file implements an IO reader of Package files.
*/

use super::package::EntryType;
use super::{error::PackageError, package::Package, parser};
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
    let pathbuf = self.cache_dir.join(filename);
    let path = pathbuf.as_path();

    if !path.exists() || !path.is_file() {
      return Err(PackageError::FileNotFound {
        target: path.to_str().unwrap().into(),
      });
    }

    let content = fs::read_to_string(path)?;
    parser::parse_entries_as_binary(&content) // XXX
  }

  pub fn read_all_from_source(
    &self,
    sources: &Vec<Source>,
  ) -> Result<HashSet<Package>, PackageError> {
    let mut results = HashSet::new();
    let base = self.cache_dir.as_path();

    for source in sources {
      let filename = base.join(source.cache_filename());
      // ignore error cuz lists file contains unreadable files such as `lock`.
      if let Ok(packages) = self.read_single_file(&filename.to_string_lossy().to_string()) {
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

  // search packages from list DB by package name.
  // Returns found packages with Source info.
  pub fn search_by_name_with_source(
    &self,
    name: &str,
    sources: &Vec<Source>,
  ) -> Result<HashSet<PackageWithSource>, PackageError> {
    let pattern = match glob::Pattern::new(name) {
      Ok(pattern) => pattern,
      Err(_) => return Err(PackageError::InvalidPackageName { name: name.into() }),
    };
    let mut results: HashSet<PackageWithSource> = HashSet::new();
    let sources: Vec<&Source> = sources
      .iter()
      .filter(|source| source.archive_type == ArchivedType::DEB)
      .collect();

    for source in sources {
      let packages = match self.read_single_file(&source.cache_filename()) {
        Ok(packages) => packages,
        // XXX ignore unreadable files
        Err(_) => continue,
      };
      let target_packages: Vec<Package> = packages
        .into_iter()
        .filter(|package| pattern.matches(&package.name))
        .collect();
      for package in target_packages {
        let package_with_source = PackageWithSource {
          package,
          source: source.clone(),
        };
        if results.contains(&package_with_source) {
          // choose newer version
          let existing = results.get(&package_with_source).unwrap();
          if existing.package.version < package_with_source.package.version {
            results.insert(package_with_source);
          }
        } else {
          results.insert(package_with_source);
        }
      }
    }

    Ok(results)
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PackageWithSource {
  pub package: Package,
  pub source: Source,
}

// hash only by its package
impl std::hash::Hash for PackageWithSource {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.package.hash(state);
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
