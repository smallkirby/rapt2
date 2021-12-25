/*
 This file defines Error type for `net` module.
*/

use flate2::DecompressError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DownloadError {
  #[error("error while sending HTTP request")]
  RequestFailed(#[from] reqwest::Error),

  #[error("error while extracting gziped content")]
  UnzipFailed(#[from] std::io::Error),
}
