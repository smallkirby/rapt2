/*
 This file defines Error type for `net` module.
*/

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
  #[error("error while sending HTTP request")]
  RequestFailed(#[from] reqwest::Error),

  #[error("error while extracting gziped content")]
  UnzipFailed(#[from] std::io::Error),

  #[error("file/dir for caching not found: {name:?}")]
  FileNotFound { name: String },
}
