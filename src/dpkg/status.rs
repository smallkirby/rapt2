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
    let parts: Vec<&str> = s.split(" ").collect();
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
      "unknown" | _ => Self::UNKNOWN,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DpkgStatusFlag {
  REINSTREQ,
  HOLD,
  HOLD_REINSTREQ,
  OK,
  UNKNOWN,
}

impl DpkgStatusFlag {
  pub fn from(s: &str) -> Self {
    match s {
      "reinstreq" => Self::REINSTREQ,
      "hold" => Self::HOLD,
      "ok" => Self::OK,
      "hold-reinstreq" => Self::HOLD_REINSTREQ,
      "unknown" | _ => Self::UNKNOWN,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DpkgStatusStatus {
  INSTALLED,
  NOTINSTALLED,
  UNPACKED,
  HALFCONFIGURED,
  HALFINSTALLED,
  CONFIGFILES,
  POST_INST_FAILED,
  REMOVAL_FAILED,
  REMOVED,
  UNKNOWN,
}

impl DpkgStatusStatus {
  pub fn from(s: &str) -> Self {
    match s {
      "installed" => Self::INSTALLED,
      "not-installed" => Self::NOTINSTALLED,
      "unpacked" => Self::UNPACKED,
      "half-configured" => Self::HALFCONFIGURED,
      "half-installed" => Self::HALFINSTALLED,
      "config-files" => Self::CONFIGFILES,
      "post-inst-failed" => Self::POST_INST_FAILED,
      "removal-failed" => Self::REMOVAL_FAILED,
      "removed" => Self::REMOVED,
      _ => Self::UNKNOWN,
    }
  }
}
