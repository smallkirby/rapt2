use super::error::DownloadError;
use crate::source::source::*;

use flate2::read::GzDecoder;
use std::io::{self, prelude::*, BufRead, BufReader};

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

  fn get_row(&self) -> Result<Vec<(String, String)>, DownloadError> {
    let mut results = vec![];

    let urls = self.source.packages_url();
    for url in &urls {
      let builder = self.client.get(url);
      match builder.send() {
        Ok(res) => {
          let bytes: Vec<u8> = res.bytes().unwrap().into_iter().collect::<Vec<u8>>();
          let mut decoder = GzDecoder::new(&bytes[..]);
          let mut body = String::new();
          decoder.read_to_string(&mut body)?;
          results.push((url.clone(), body));
        }
        Err(err) => return Err(DownloadError::RequestFailed(err)),
      };
    }

    Ok(results)
  }
}
