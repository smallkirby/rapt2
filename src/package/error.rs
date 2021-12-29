/*
 This file defines Error type for `package` module.
*/

use super::package::EntryType;

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

  #[error(
    "Package entry lacks information for constructing package information: {msg:?} as {typ:?}"
  )]
  IncompleteEntry { msg: String, typ: EntryType },

  #[error("invalid package name is specified: {name:?}")]
  InvalidPackageName { name: String },

  #[error("Package not found: {package_name:?}")]
  PackageNotFound { package_name: String },
}
