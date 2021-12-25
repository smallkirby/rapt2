/*
 This file implements an IO reader of Package files.
*/

use super::error::PackageError;

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
}
