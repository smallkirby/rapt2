/*
 This file implements misc helper functions.
*/

use fs2::FileExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use users::get_current_uid;

pub fn split_by_empty_line(s: &str) -> Vec<Vec<String>> {
  let mut result = vec![];
  let mut acc = vec![];

  for line in s.trim().split("\n") {
    if line.trim().len() == 0 {
      if acc.len() != 0 {
        result.push(acc.clone());
      }
      acc.clear();
    } else {
      acc.push(line.into());
    }
  }

  if acc.len() != 0 {
    result.push(acc.clone());
  }

  result
}

pub fn first_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for ix in 0..s.len() {
    if (s_bytes[ix] as char).is_numeric() {
      return Some(ix);
    }
  }

  None
}

pub fn first_non_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for ix in 0..s.len() {
    if !(s_bytes[ix] as char).is_numeric() {
      return Some(ix);
    }
  }

  None
}

#[derive(Error, Debug)]
pub enum FileLockError {
  #[error("file not exists: {filename:?}")]
  FileNotExist { filename: String },

  #[error("file operation failed: {operation:?}")]
  FileOperationError { operation: String },

  #[error("couldn't acquire lock.")]
  LockAcquireFailed,
}

pub fn try_lock_file(path: PathBuf, force_create: bool) -> Result<fs::File, FileLockError> {
  let path = path.as_path();
  if path.exists() && path.is_dir() {
    return Err(FileLockError::FileNotExist {
      filename: path.to_string_lossy().to_string(),
    });
  }
  if !path.exists() {
    if !force_create {
      return Err(FileLockError::FileNotExist {
        filename: path.to_string_lossy().to_string(),
      });
    }
    if fs::File::create(path).is_err() {
      return Err(FileLockError::FileOperationError {
        operation: "create".into(),
      });
    }
  }

  match fs::File::open(path) {
    Ok(file) => match file.try_lock_exclusive() {
      Ok(()) => Ok(file),
      Err(_) => Err(FileLockError::LockAcquireFailed),
    },
    Err(_) => Err(FileLockError::FileOperationError {
      operation: "open".into(),
    }),
  }
}

pub fn create_long_spinner(msg: String) -> ProgressBar {
  let pb = ProgressBar::new_spinner();

  pb.set_message(msg);

  pb.enable_steady_tick(120);
  pb.set_style(
    ProgressStyle::default_spinner()
      .tick_strings(&[
        "▹▹▹▹▹",
        "▸▹▹▹▹",
        "▹▸▹▹▹",
        "▹▹▸▹▹",
        "▹▹▹▸▹",
        "▹▹▹▹▸",
        "▪▪▪▪▪",
      ])
      .template("{spinner:.blue} {msg}"),
  );

  pb
}

pub fn ami_root() -> bool {
  get_current_uid() == 0
}
