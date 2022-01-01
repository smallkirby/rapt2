/*
 This file defines a downloader of binary packages(.deb).
*/

use super::error::DownloadError;
use crate::package::client::PackageWithSource;

use reqwest::StatusCode;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

pub struct BinaryDownloader {
  packages: Vec<PackageWithSource>,
  cache_dir: PathBuf,
  curr: usize,
}

pub struct BinaryDownloaderExecuter {
  pub pws: PackageWithSource,
  cache_dir: PathBuf,
  client: reqwest::blocking::Client,
}

impl BinaryDownloader {
  pub fn new(packages: Vec<PackageWithSource>, cache_dir: PathBuf) -> Result<Self, DownloadError> {
    // check if `cache_dir` exists
    if !cache_dir.as_path().is_dir() && std::fs::create_dir(&cache_dir).is_err() {
      return Err(DownloadError::FileNotFound {
        name: cache_dir.to_string_lossy().to_string(),
      });
    }

    Ok(Self {
      packages,
      cache_dir,
      curr: 0,
    })
  }
}

impl BinaryDownloaderExecuter {
  pub fn download(&self) -> Result<(), DownloadError> {
    let package = &self.pws.package;
    let source = &self.pws.source;
    let url = format!("{}{}", source.url, package.filename);
    let builder = self.client.get(url);
    let filename = package.filename.split('/').last().unwrap();
    let filepath = self.cache_dir.join(filename);

    match builder.send() {
      Ok(res) => {
        if res.status() == StatusCode::OK {
          let bytes = res.bytes()?;
          let mut cache_file = fs::File::create(filepath)?;
          let mut content = Cursor::new(bytes);
          std::io::copy(&mut content, &mut cache_file)?;
        } else {
          return Err(DownloadError::InvalidStatusCode {
            status: res.status(),
          });
        }
      }
      Err(err) => return Err(DownloadError::RequestFailed(err)),
    }

    Ok(())
  }
}

impl Iterator for BinaryDownloader {
  type Item = BinaryDownloaderExecuter;

  fn next(&mut self) -> Option<Self::Item> {
    if self.curr >= self.packages.len() {
      return None;
    }
    let client = reqwest::blocking::Client::builder().build().unwrap();
    let ix = self.curr;
    self.curr += 1;
    Some(Self::Item {
      pws: self.packages[ix].clone(),
      client,
      cache_dir: self.cache_dir.clone(),
    })
  }
}
