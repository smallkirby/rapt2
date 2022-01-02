/*
 This file defines dpkg client to request installation of packages.
*/

use crate::{
  apt::extended_states::AptExtendedStateClient,
  package::{client::PackageWithSource, error::PackageError},
};

use std::{path::PathBuf, process::Command};

pub struct DpkgInstaller {
  archive_dir: PathBuf,
  pub pwss: Vec<PackageWithSource>, // packages to be installed
  automatics: Vec<String>,          // names of automatically installed packages
  extended_state: PathBuf,          // apt extended_state path
}

pub struct DpkgExtracterIter {
  archive_dir: PathBuf,
  curr: usize,
  pub pwss: Vec<PackageWithSource>,
  automatics: Vec<String>, // names of automatically installed packages
  extended_state: PathBuf,
}

pub struct DpkgExtracter {
  archive_dir: PathBuf,
  pub pws: PackageWithSource,
  is_automatic: bool,
  extended_state: PathBuf,
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
    Some(Self::Item {
      pws: self.pwss[ix].clone(),
      archive_dir: self.archive_dir.clone(),
      is_automatic: self.automatics.contains(&self.pwss[ix].package.name),
      extended_state: self.extended_state.clone(),
    })
  }
}

impl DpkgExtracter {
  pub fn execute(&self) -> Result<(), PackageError> {
    let package = &self.pws.package;
    let archived_filename = package.filename.split('/').last().unwrap();
    let archived_path = self.archive_dir.join(archived_filename);
    let archived_fullname = archived_path.to_string_lossy().to_string();
    let extended_state_client = AptExtendedStateClient::new(&self.extended_state);

    if !archived_path.as_path().is_file() {
      return Err(PackageError::FileNotFound {
        target: archived_fullname,
      });
    }

    // XXX should parse `Break` field instead using `--auto-deconfigure`.
    let output = Command::new("dpkg")
      .args(&["--auto-deconfigure", "--unpack", &archived_fullname])
      .output()
      .unwrap();
    if output.status.success() {
      extended_state_client.update(&package.name, self.is_automatic)?;
      Ok(())
    } else {
      let errstr = String::from_utf8(output.stderr).unwrap();
      Err(PackageError::InstallFailed {
        package_name: package.name.to_string(),
        errstr,
      })
    }
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
    Some(Self::Item {
      pws: self.pwss[ix].clone(),
    })
  }
}

impl DpkgConfigurer {
  pub fn execute(&self) -> Result<(), PackageError> {
    let package = &self.pws.package;

    let output = Command::new("dpkg")
      .args(&["--configure", &package.name])
      .output()
      .unwrap();
    if output.status.success() {
      Ok(())
    } else {
      let errstr = String::from_utf8(output.stderr).unwrap();
      Err(PackageError::InstallFailed {
        package_name: package.name.to_string(),
        errstr,
      })
    }
  }
}

impl DpkgInstaller {
  pub fn new(
    archive_dir: PathBuf,
    pwss: Vec<PackageWithSource>,
    automatics: Vec<String>,
    extended_state: PathBuf,
  ) -> Result<Self, PackageError> {
    if !archive_dir.as_path().is_dir() {
      return Err(PackageError::FileNotFound {
        target: archive_dir.to_string_lossy().to_string(),
      });
    }

    Ok(Self {
      archive_dir,
      pwss,
      automatics,
      extended_state,
    })
  }

  pub fn extracters_iter(&self) -> DpkgExtracterIter {
    DpkgExtracterIter {
      archive_dir: self.archive_dir.clone(),
      pwss: self.pwss.clone(),
      curr: 0,
      automatics: self.automatics.clone(),
      extended_state: self.extended_state.clone(),
    }
  }

  pub fn configuers_iter(&self) -> DpkgConfigurerIter {
    DpkgConfigurerIter {
      pwss: self.pwss.clone(),
      curr: 0,
    }
  }
}
