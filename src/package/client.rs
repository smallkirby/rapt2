/*
 This file implements an IO reader of Package files.
*/

use super::package::EntryType;
use super::{error::PackageError, package::Package, parser};

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

  pub fn read_single_file(&self, filename: &str) -> Result<HashSet<Package>, PackageError> {
    let pathbuf = self.cache_dir.join(filename);
    let path = pathbuf.as_path();

    if !path.exists() || !path.is_file() {
      return Err(PackageError::FileNotFound {
        target: path.to_str().unwrap().into(),
      });
    }

    let content = fs::read_to_string(path)?;
    println!("{}", content);
    parser::parse_entries_as_binary(&content) // XXX
  }
}

pub fn to_packages(content: &str, entry_type: EntryType) -> Result<HashSet<Package>, PackageError> {
  match entry_type {
    EntryType::BINARY => parser::parse_entries_as_binary(content),
    EntryType::SOURCE => parser::parse_entries_as_source(content),
    EntryType::STATUS => parser::parse_entries_as_status(content),
  }
}
