/*
 This file implements misc helper functions.
*/

use chrono::{DateTime, NaiveDateTime, Utc};
use fs2::FileExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use users::get_current_uid;

pub mod emoji {
  use console::Emoji;

  pub static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");
  pub static EMOJI_BOOKMARK: Emoji<'_, '_> = Emoji("🔖", "");
  pub static EMOJI_GLASS: Emoji<'_, '_> = Emoji("🔍", "");
  pub static EMOJI_SPARKLES: Emoji<'_, '_> = Emoji("✨", "");
  pub static EMOJI_EXC: Emoji<'_, '_> = Emoji("❗", "");
  pub static EMOJI_LOCK: Emoji<'_, '_> = Emoji("🔐", "");
  pub static EMOJI_CROSS: Emoji<'_, '_> = Emoji("❌", "");
  pub static EMOJI_DOWN: Emoji<'_, '_> = Emoji("⬇️", "");
  pub static EMOJI_TARGET: Emoji<'_, '_> = Emoji("🎯", "");
  pub static EMOJI_EARTH: Emoji<'_, '_> = Emoji("🌎", "");
  pub static EMOJI_INFORMATION: Emoji<'_, '_> = Emoji("ℹ️", "");
  pub static EMOJI_COMPUTER: Emoji<'_, '_> = Emoji("💻", "");
}

pub fn split_by_empty_line(s: &str) -> Vec<Vec<String>> {
  let mut result = vec![];
  let mut acc = vec![];

  for line in s.trim().split('\n') {
    if line.trim().is_empty() {
      if !acc.is_empty() {
        result.push(acc.clone());
      }
      acc.clear();
    } else {
      acc.push(line.into());
    }
  }

  if !acc.is_empty() {
    result.push(acc.clone());
  }

  result
}

pub fn first_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for (ix, c) in s_bytes.iter().enumerate().take(s.len()) {
    if (*c as char).is_numeric() {
      return Some(ix);
    }
  }

  None
}

pub fn first_non_numeric(s: &str) -> Option<usize> {
  let s_bytes = s.as_bytes();
  for (ix, c) in s_bytes.iter().enumerate().take(s.len()) {
    if !(*c as char).is_numeric() {
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

// convert SystemTime into `If-Modified-Since` format string.
pub fn timestamp2ims(t: SystemTime) -> String {
  let secs = t.duration_since(UNIX_EPOCH).unwrap().as_secs();
  let naive = NaiveDateTime::from_timestamp(secs as i64, 0);
  let utc: DateTime<Utc> = DateTime::from_utc(naive, Utc);
  utc.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

pub fn default_progbar(len: u64) -> ProgressBar {
  let prog_style = ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .progress_chars("##-");
  let progress = ProgressBar::new(len);
  progress.set_style(prog_style);

  progress
}
