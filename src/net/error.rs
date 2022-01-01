/*
 This file defines Error type for `net` module.
*/

use thiserror::Error;

use reqwest::StatusCode;

#[derive(Error, Debug)]
pub enum DownloadError {
  #[error("error while sending HTTP request")]
  RequestFailed(#[from] reqwest::Error),

  #[error("invalid status code is returned: {status:?}")]
  InvalidStatusCode { status: StatusCode },

  #[error("file IO error")]
  FileIOError(#[from] std::io::Error),

  #[error("file/dir for caching not found: {name:?}")]
  FileNotFound { name: String },

  #[error("package IO error")]
  PackageIOError(#[from] crate::package::error::PackageError),
}
