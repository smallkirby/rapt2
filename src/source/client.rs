/*
 This file defines client for IO of sources.list files.
*/

use super::{error::SourceError, parser, source::*};

use itertools::Itertools;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SourceClient {
  source_dir: PathBuf, // sources base dir
}

impl SourceClient {
  pub fn new(source_dir: PathBuf) -> Result<Self, SourceError> {
    let path = Path::new(&source_dir);
    if !path.exists() {
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

  pub fn read_single_file(&self, filename: &str) -> Result<Vec<Source>, SourceError> {
    let pathbuf = self.source_dir.join(filename);
    self.read_single_file_internal(pathbuf.as_path().to_owned())
  }

  fn read_single_file_internal(&self, path: PathBuf) -> Result<Vec<Source>, SourceError> {
    if !path.exists() || !path.is_file() {
      return Err(SourceError::FileNotFound {
        target: path.to_str().unwrap().into(),
      });
    }

    let content = fs::read_to_string(path)?;

    parser::parse_lines(&content)
  }

  // search source directory and read all files
  pub fn read_all(&self) -> Result<Vec<Source>, SourceError> {
    let mut sources = vec![];
    let list_pathes = self.find_candidates();
    for path in list_pathes {
      sources.push(self.read_single_file_internal(path)?);
    }

    Ok(sources.into_iter().unique().flatten().collect())
  }

  // find and return candidate list files.
  fn find_candidates(&self) -> Vec<PathBuf> {
    let mut target_pathes = vec![];

    // first, find `sources.list` in base dir.
    let sources_list_path = self.source_dir.join("sources.list");
    if sources_list_path.as_path().is_file() {
      target_pathes.push(sources_list_path);
    }

    // next, search `sources.list.d` if exists
    let sources_list_d_path = self.source_dir.join("sources.list.d");
    if sources_list_d_path.as_path().is_dir() {
      let candidates = fs::read_dir(sources_list_d_path).unwrap();
      for candidate in candidates {
        if let Ok(ent) = candidate {
          let path = ent.path();
          if path.is_file() && path.extension().is_some() && path.extension().unwrap() == "list" {
            target_pathes.push(path);
          }
        }
      }
    }

    target_pathes
  }
}
