/*
 This file defines a downloader of Packages file in ubuntu repository.
 For caching strategy, refer to /docs/caching.md
*/

use super::error::DownloadError;
use crate::source::source::*;
use crate::util::*;

use flate2::read::GzDecoder;
use reqwest::header::IF_MODIFIED_SINCE;
use reqwest::StatusCode;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug)]
pub struct PackageDownloadClient {
  source_infos: Vec<SourceInfo>,
  cache_dir: PathBuf,
  cur_inrelease: usize, // index of SourceInfo which is checked for `InRelease`
  cur_packages: usize,  // index of SourceInfo which is checked for `Packages`
}

#[derive(Debug)]
struct SourceInfo {
  source: Source,
  old_package_hash: Option<String>,
  should_update: Option<bool>,
}

impl PackageDownloadClient {
  pub fn new(sources: Vec<Source>, cache_dir: PathBuf) -> Result<Self, DownloadError> {
    let mut source_infos: Vec<SourceInfo> = vec![];

    // check existence of cache dir
    if !cache_dir.as_path().is_dir() && std::fs::create_dir(&cache_dir).is_err() {
      return Err(DownloadError::FileNotFound {
        name: cache_dir.to_string_lossy().to_string(),
      });
    }

    // grouping sources based on its `url` and `distro`
    let mut source_groups: Vec<Vec<Source>> = vec![];
    for source in sources {
      let position = source_groups
        .iter()
        .position(|group| group[0].url == source.url && group[0].distro == source.distro);
      match position {
        Some(p) => source_groups[p].push(source),
        None => source_groups.push(vec![source]),
      }
    }

    // read `InRelease` caches and associate old MD5Sum hash with each source.
    for group in source_groups {
      let representative_source = &group[0];
      let inrelease = match read_inrelease_cache(&representative_source, &cache_dir) {
        Some(inrelease) => inrelease,
        None => {
          drop(representative_source);
          for source in group {
            source_infos.push(SourceInfo {
              source,
              old_package_hash: None,
              should_update: None,
            })
          }
          continue;
        }
      };
      drop(representative_source);
      for source in group {
        let search_string = format!("{}/binary-amd64/Packages.gz", source.component);
        let md5 = search_md5(&search_string, &inrelease);
        source_infos.push(SourceInfo {
          source,
          old_package_hash: md5,
          should_update: None,
        });
      }
    }

    Ok(Self {
      source_infos,
      cache_dir,
      cur_inrelease: 0,
      cur_packages: 0,
    })
  }

  pub fn get_done_packages_num(&mut self) -> usize {
    self.cur_packages
  }

  pub fn get_done_inrelease_num(&mut self) -> usize {
    self.increment_inrelease_counter();
    self.cur_inrelease
  }

  fn increment_inrelease_counter(&mut self) {
    while self.cur_inrelease < self.source_infos.len() {
      if self.source_infos[self.cur_inrelease]
        .should_update
        .is_none()
      {
        break;
      }
      self.cur_inrelease += 1;
    }
  }

  pub fn get_next_target_source_packages(&mut self) -> Option<Source> {
    if self.cur_packages >= self.source_infos.len() {
      None
    } else {
      Some(self.source_infos[self.cur_packages].source.clone())
    }
  }

  pub fn get_next_target_source_inrelease(&mut self) -> Option<Source> {
    // increment `self.cur_inrelease` til finding not-check Source.
    self.increment_inrelease_counter();

    if self.cur_inrelease >= self.source_infos.len() {
      None
    } else {
      Some(self.source_infos[self.cur_inrelease].source.clone())
    }
  }

  // Get `InRelease` and update each sources should download `Packages` files.
  // If all downloads are complete, returns Ok(None).
  pub fn get_package_ifneed(&mut self) -> Result<Option<String>, DownloadError> {
    if self.cur_packages >= self.source_infos.len() {
      return Ok(None);
    }

    // if there is no need to download `Packages`, read local DB cache.
    let source = &self.source_infos[self.cur_packages].source;
    let should_update = self.source_infos[self.cur_packages].should_update.unwrap();
    self.cur_packages += 1;
    if !should_update {
      let package_client = crate::package::client::PackageClient::new(self.cache_dir.clone())?;
      let packages_str = package_client.read_single_file_raw(&source.cache_filename())?;
      return Ok(Some(packages_str));
    }

    // Actual download of `Packages`.
    let url = source.packages_url();
    let client = reqwest::blocking::Client::builder()
      .gzip(false)
      .build()
      .unwrap();
    // XXX should add `If-Modified-Since` here also?
    let result = client.get(url).send();
    match result {
      Ok(res) => {
        let bytes: Vec<u8> = res.bytes().unwrap().into_iter().collect::<Vec<u8>>();
        let mut decoder = GzDecoder::new(&bytes[..]);
        let mut body = String::new();
        decoder.read_to_string(&mut body)?;

        // save cache
        self.save_cache_packages(&body, source)?;
        Ok(Some(body))
      }
      Err(err) => Err(DownloadError::RequestFailed(err)),
    }
  }

  // Get `InRelease` and update each sources should download `Packages` files.
  // If check is complete, returns Ok(false).
  // NOTE: Actually, I want to implement this as Iterator.
  //      But changing Iter itself is not allowed for elements of Iter in Rust.
  pub fn get_inrelease_ifneed(&mut self) -> Result<bool, DownloadError> {
    // increment `self.cur_inrelease` til finding not-check Source.
    self.increment_inrelease_counter();
    if self.cur_inrelease >= self.source_infos.len() {
      return Ok(false);
    }

    // fetch `InRelease` file
    // and check `InRelease` file and update whether each source should download `Packages`.
    let source = &self.source_infos[self.cur_inrelease].source.clone();
    match self.get_inrelease(source)? {
      Some(inrelease) => {
        self.update_should_download_packages(Some(&inrelease), source);
        self.save_cache_inrelease(&inrelease, source)?;
      }
      None => self.update_should_download_packages(None, source),
    };

    Ok(true)
  }

  // update `self.should_update` status of SourceInfo by comparing new and old MD5 of Packages.
  // if `inrelease` given is None, this func regards all associated sources are up-to-new.
  fn update_should_download_packages(&mut self, inrelease: Option<&str>, source: &Source) {
    let targets: Vec<&mut SourceInfo> = self
      .source_infos
      .iter_mut()
      .filter(|info| info.source.url == source.url && info.source.distro == source.distro)
      .collect();

    // first, check if list DB exists
    if !check_listdb_exists(&self.cache_dir, source) {
      for target in targets {
        target.should_update = Some(true);
      }
      return;
    }

    if let Some(inrelease) = inrelease {
      for target in targets {
        // check existing list DB's md5 hash
        let target_str = format!("{}/binary-amd64/Packages.gz", target.source.component);
        match search_md5(&target_str, inrelease) {
          Some(md5) => {
            if target.old_package_hash.is_some()
              && target.old_package_hash.as_ref().unwrap() == &md5
            {
              target.should_update = Some(false);
            } else {
              target.should_update = Some(true);
            }
          }
          None => target.should_update = Some(true),
        }
      }
    } else {
      // no need to download packages
      for target in targets {
        target.should_update = Some(false);
      }
    }
  }

  // Get `InRelease` file with using cache.
  // If `InRelease` is not modified, it returns `Ok(None)`
  fn get_inrelease(&self, source: &Source) -> Result<Option<String>, DownloadError> {
    let url = source.inrelease_url();
    let client = reqwest::blocking::Client::new();
    let result = if let Some(timestamp) = self.check_existing_timestamp(source) {
      client
        .get(url)
        .header(IF_MODIFIED_SINCE, timestamp2ims(timestamp))
        .send()
    } else {
      client.get(url).send()
    };
    match result {
      Ok(res) => match res.status() {
        StatusCode::NOT_MODIFIED => Ok(None),
        StatusCode::OK => Ok(Some(res.text().unwrap())),
        _ => Err(DownloadError::InvalidStatusCode {
          status: res.status(),
        }),
      },
      Err(err) => Err(DownloadError::RequestFailed(err)),
    }
  }

  // get timestamp of target source's `InRelease` file
  fn check_existing_timestamp(&self, source: &Source) -> Option<SystemTime> {
    let filepathbuf = self.cache_dir.join(source.inrelease_filename());
    let meta = match fs::metadata(filepathbuf.as_path()) {
      Ok(meta) => meta,
      Err(_) => return None,
    };
    match meta.modified() {
      Ok(t) => Some(t),
      Err(_) => None,
    }
  }

  fn save_cache_inrelease(&self, content: &str, source: &Source) -> Result<(), DownloadError> {
    fs::write(
      self.cache_dir.join(source.inrelease_filename()).as_path(),
      content,
    )?;
    Ok(())
  }

  fn save_cache_packages(&self, content: &str, source: &Source) -> Result<(), DownloadError> {
    fs::write(
      self.cache_dir.join(source.cache_filename()).as_path(),
      content,
    )?;
    Ok(())
  }
}

fn read_inrelease_cache(source: &Source, cache_dir: &PathBuf) -> Option<String> {
  let filepathbuf = cache_dir.join(source.inrelease_filename());
  match fs::read_to_string(filepathbuf.as_path()) {
    Ok(s) => Some(s),
    Err(_) => None,
  }
}

fn search_md5(target: &str, inrelease: &str) -> Option<String> {
  for line in inrelease.lines() {
    if line.contains(target) {
      let parts: Vec<&str> = line.split_whitespace().collect();
      return Some(parts.first().unwrap().trim().to_string());
    }
  }
  None
}

fn check_listdb_exists(package_cache_dir: &PathBuf, source: &Source) -> bool {
  let filepathbuf = package_cache_dir.join(source.cache_filename());
  filepathbuf.as_path().is_file()
}
