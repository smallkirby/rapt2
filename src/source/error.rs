/*
 This file defines Error type for `source` module.
*/

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
  #[error("file/dir not found: {target:?}")]
  FileNotFound { target: String },

  #[error("error in file IO")]
  FileIoError(#[from] io::Error),

  #[error("invalid sources.list format: {msg:?}")]
  InvalidFormat { msg: String },

  #[error("invalid field in sources.list: {field:?} = {value:?}")]
  InvalidField { field: String, value: String },
}
