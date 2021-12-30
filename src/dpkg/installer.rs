/*
 This file defines dpkg client to request installation of packages.
*/

use crate::package::{client::PackageWithSource, error::PackageError};

use std::{
  path::PathBuf,
  process::{Command, Stdio},
};

pub struct DpkgInstaller {
  archive_dir: PathBuf,
  pub pwss: Vec<PackageWithSource>,
}

pub struct DpkgExtracterIter {
  archive_dir: PathBuf,
  curr: usize,
  pub pwss: Vec<PackageWithSource>,
}

pub struct DpkgExtracter {
  archive_dir: PathBuf,
  pub pws: PackageWithSource,
}

pub struct DpkgConfigurerIter {
  curr: usize,
  pub pwss: Vec<PackageWithSource>,
}

pub struct DpkgConfigurer {
  pub pws: PackageWithSource,
}

impl Iterator for DpkgExtracterIter {
  type Item = DpkgExtracter;

  fn next(&mut self) -> Option<Self::Item> {
    if self.curr >= self.pwss.len() {
      return None;
    }
    let ix = self.curr;
    self.curr += 1;
    return Some(Self::Item {
      pws: self.pwss[ix].clone(),
      archive_dir: self.archive_dir.clone(),
    });
  }
}

impl DpkgExtracter {
  pub fn execute(&self) -> Result<(), PackageError> {
    let package = &self.pws.package;
    let archived_filename = package.filename.split("/").last().unwrap();
    let archived_path = self.archive_dir.join(archived_filename);
    let archived_fullname = archived_path.to_string_lossy().to_string();

    if !archived_path.as_path().is_file() {
      return Err(PackageError::FileNotFound {
        target: archived_fullname,
      });
    }

    let output = Command::new("dpkg")
      .args(&["--unpack", &archived_fullname])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap()
      .wait_with_output()
      .unwrap();
    let outstr = String::from_utf8(output.stdout).unwrap();
    println!("{}", outstr);
    let errstr = String::from_utf8(output.stderr).unwrap();
    println!("{}", errstr);

    Ok(())
  }
}

impl Iterator for DpkgConfigurerIter {
  type Item = DpkgConfigurer;

  fn next(&mut self) -> Option<Self::Item> {
    if self.curr >= self.pwss.len() {
      return None;
    }
    let ix = self.curr;
    self.curr += 1;
    return Some(Self::Item {
      pws: self.pwss[ix].clone(),
    });
  }
}

impl DpkgConfigurer {
  pub fn execute(&self) -> Result<(), PackageError> {
    let package = &self.pws.package;

    let output = Command::new("dpkg")
      .args(&["--configure", &package.name])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap()
      .wait_with_output()
      .unwrap();
    let outstr = String::from_utf8(output.stdout).unwrap();
    println!("{}", outstr);
    let errstr = String::from_utf8(output.stderr).unwrap();
    println!("{}", errstr);

    Ok(())
  }
}

impl DpkgInstaller {
  pub fn new(archive_dir: PathBuf, pwss: Vec<PackageWithSource>) -> Result<Self, PackageError> {
    if !archive_dir.as_path().is_dir() {
      return Err(PackageError::FileNotFound {
        target: archive_dir.to_string_lossy().to_string(),
      });
    }

    Ok(Self { archive_dir, pwss })
  }

  pub fn extracters_iter(&self) -> DpkgExtracterIter {
    DpkgExtracterIter {
      archive_dir: self.archive_dir.clone(),
      pwss: self.pwss.clone(),
      curr: 0,
    }
  }

  pub fn configuers_iter(&self) -> DpkgConfigurerIter {
    DpkgConfigurerIter {
      pwss: self.pwss.clone(),
      curr: 0,
    }
  }
}