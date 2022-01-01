/*
 This file defines Error type for entire app: rapt
*/

use crate::{net::error::DownloadError, package::error::PackageError, source::error::SourceError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaptError {
  #[error("Source related error")]
  RaptSourceError(#[from] SourceError),

  #[error("Download related error")]
  RaptDownloadError(#[from] DownloadError),

  #[error("Package related error")]
  RaptPackageError(#[from] PackageError),

  #[error("Permission related error.")]
  PermissionDenied,

  #[error("Invalid input: {msg:?}")]
  InvalidInput { msg: String },

  #[error("Error while resolving dependency.")]
  ImpossibleDependency(#[from] crate::algorithm::dag::DagError),
}
