/*
 This file implements an IO reader of Package files.
*/

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
    if !path.exists() {
      Err(PackageError::FileNotFound {
        target: path.to_string_lossy().to_string(),
      })
    } else if !path.is_dir() {
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
    parser::parse_entries(&content)
  }
}
