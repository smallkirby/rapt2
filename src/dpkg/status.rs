/*
 This file defines `Status` field of `/var/lib/dpkg/status`, called `Staus Area`.
*/

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DpkgStatusArea {
  pub want: DpkgStatusWant,
  pub flag: DpkgStatusFlag,
  pub status: DpkgStatusStatus,
}

impl DpkgStatusArea {
  pub fn from(s: &str) -> Self {
    let parts: Vec<&str> = s.split(' ').collect();
    if parts.len() != 3 {
      panic!("invalid status area: {}", s);
    }
    let want = DpkgStatusWant::from(parts[0]);
    let flag = DpkgStatusFlag::from(parts[1]);
    let status = DpkgStatusStatus::from(parts[2]);
    Self { want, flag, status }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DpkgStatusWant {
  INSTALL,
  HOLD,
  DEINSTALL,
  PURGE,
  UNKNOWN,
}

impl DpkgStatusWant {
  pub fn from(s: &str) -> Self {
    match s {
      "install" => Self::INSTALL,
      "hold" => Self::HOLD,
      "deinstall" => Self::DEINSTALL,
      "purge" => Self::PURGE,
      "unknown" => Self::UNKNOWN,
      _ => Self::UNKNOWN,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DpkgStatusFlag {
  ReinstReq,
  Hold,
  HoldReinstReq,
  Ok,
  Unknown,
}

impl DpkgStatusFlag {
  pub fn from(s: &str) -> Self {
    match s {
      "reinstreq" => Self::ReinstReq,
      "hold" => Self::Hold,
      "ok" => Self::Ok,
      "hold-reinstreq" => Self::HoldReinstReq,
      "unknown" => Self::Unknown,
      _ => Self::Unknown,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DpkgStatusStatus {
  Installed,
  NotInstalled,
  Unpacked,
  HalfConfigured,
  HalfInstalled,
  ConfigFiles,
  PostInstFailed,
  RemovalFailed,
  Removed,
  Unknown,
}

impl DpkgStatusStatus {
  pub fn from(s: &str) -> Self {
    match s {
      "installed" => Self::Installed,
      "not-installed" => Self::NotInstalled,
      "unpacked" => Self::Unpacked,
      "half-configured" => Self::HalfConfigured,
      "half-installed" => Self::HalfInstalled,
      "config-files" => Self::ConfigFiles,
      "post-inst-failed" => Self::PostInstFailed,
      "removal-failed" => Self::RemovalFailed,
      "removed" => Self::Removed,
      _ => Self::Unknown,
    }
  }
}
