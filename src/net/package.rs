use super::error::DownloadError;
use crate::source::source::*;

use flate2::read::GzDecoder;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct PackageDownloader {
  source: Source,
  client: reqwest::blocking::Client,
  cache_dir: PathBuf,
}

impl PackageDownloader {
  pub fn new(source: Source, cache_dir: PathBuf) -> Result<Self, DownloadError> {
    // check existence of cache dir
    if !cache_dir.as_path().is_dir() && std::fs::create_dir(&cache_dir).is_err() {
      return Err(DownloadError::FileNotFound {
        name: cache_dir.to_string_lossy().to_string(),
      });
    }

    // construct HTTP client
    let client = reqwest::blocking::Client::builder()
      .gzip(false)
      .build()
      .unwrap();
    Ok(Self {
      source,
      client,
      cache_dir,
    })
  }

  pub fn get(&self) -> Result<String, DownloadError> {
    let url = self.source.packages_url();
    let builder = self.client.get(url);
    match builder.send() {
      Ok(res) => {
        let bytes: Vec<u8> = res.bytes().unwrap().into_iter().collect::<Vec<u8>>();
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut body = String::new();
        decoder.read_to_string(&mut body)?;

        // save cache
        self.save_cache(&body)?;
        Ok(body)
      }
      Err(err) => Err(DownloadError::RequestFailed(err)),
    }
  }

  fn save_cache(&self, content: &str) -> Result<(), DownloadError> {
    fs::write(
      self.cache_dir.join(self.source.cache_filename()).as_path(),
      content,
    )?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  //#[test]
  #[allow(dead_code)]
  fn test_package_get_row() {
    let source = Source::from(
      ArchivedType::DEB,
      "http://jp.archive.ubuntu.com/ubuntu",
      "focal",
      vec![Component::MAIN],
    )
    .into_iter()
    .next()
    .unwrap();

    let client = PackageDownloader::new(source, PathBuf::from("./rapt2/lists/cache")).unwrap();
    assert_eq!(client.get().is_ok(), true);
  }
}
