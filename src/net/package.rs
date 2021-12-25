use super::error::DownloadError;
use crate::source::source::*;

use flate2::read::GzDecoder;
use std::io::prelude::*;

#[derive(Debug)]
pub struct PackageDownloader {
  source: Source,
  client: reqwest::blocking::Client,
}

impl PackageDownloader {
  pub fn new(source: Source) -> Self {
    let client = reqwest::blocking::Client::builder()
      .gzip(false)
      .build()
      .unwrap();
    Self { source, client }
  }

  fn get_row(&self) -> Result<String, DownloadError> {
    let url = self.source.packages_url();
    let builder = self.client.get(url);
    match builder.send() {
      Ok(res) => {
        let bytes: Vec<u8> = res.bytes().unwrap().into_iter().collect::<Vec<u8>>();
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut body = String::new();
        decoder.read_to_string(&mut body)?;
        Ok(body)
      }
      Err(err) => Err(DownloadError::RequestFailed(err)),
    }
  }
}
