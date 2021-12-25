/*
 This file defines Error type for `package` module.
*/

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackageError {
  #[error("file/dir not found: {target:?}")]
  FileNotFound { target: String },

  #[error("error in file IO")]
  FileIoError(#[from] io::Error),

  #[error("invalid Package format: {msg:?}")]
  InvalidFormat { msg: String },

  #[error("invalid field in Package entry: {field:?} = {value:?}")]
  InvalidField { field: String, value: String },

  #[error("Package entry lacks information for constructing package information: {msg:?}")]
  LackingField { msg: String },
}
