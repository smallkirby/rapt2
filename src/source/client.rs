/*
 This file defines client for IO of sources.list files.
*/

use super::{error::SourceError, parser, source::*};

use std::fs;
use std::path::{Path, PathBuf};

pub struct SourceClient {
  source_dir: PathBuf, // sources base dir
}

impl SourceClient {
  pub fn new(source_dir: PathBuf) -> Result<Self, SourceError> {
    let path = Path::new(&source_dir);
    if path.exists() {
      Err(SourceError::FileNotFound {
        target: path.to_string_lossy().to_string(),
      })
    } else if !path.is_dir() {
      Err(SourceError::FileNotFound {
        target: path.to_string_lossy().to_string(),
      })
    } else {
      Ok(Self { source_dir })
    }
  }

  fn read_single_file(&self, filename: &str) -> Result<Vec<Source>, SourceError> {
    let pathbuf = self.source_dir.join(filename);
    let path = Path::new(&pathbuf);
    if !path.exists() || !path.is_file() {
      return Err(SourceError::FileNotFound {
        target: path.to_str().unwrap().into(),
      });
    }

    let content = fs::read_to_string(path)?;

    parser::parse_lines(&content)
  }
}
