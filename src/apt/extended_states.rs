/*
 This file defines client for IO of `/var/lib/apt/extended_states`.
 Refer to `/docs/extended_states.md` for the details.
*/

use std::fs;
use std::path::PathBuf;

pub struct AptExtendedStates {
  path: PathBuf,
}

impl Default for AptExtendedStates {
  fn default() -> Self {
    Self::new()
  }
}

impl AptExtendedStates {
  pub fn new() -> Self {
    Self {
      path: PathBuf::from("/var/lib/apt/extended_states"),
    }
  }

  pub fn get(&self) -> Result<Vec<AptExtendedPackageInfo>, std::io::Error> {
    let content = fs::read_to_string(self.path.as_path())?;
    let mut lines = vec![];
    let mut result = vec![];
    for line in content.split('\n') {
      if line.trim().is_empty() {
        if let Some(entry) = AptExtendedPackageInfo::from(&lines.join("\n")) {
          result.push(entry);
        }
        lines.clear();
        continue;
      }
      lines.push(line);
    }

    Ok(result)
  }
}

#[derive(Debug)]
pub struct AptExtendedPackageInfo {
  pub name: String,
  pub arch: String,
  pub automatic_installed: bool,
}

impl AptExtendedPackageInfo {
  pub fn from(s: &str) -> Option<Self> {
    let lines: Vec<&str> = s.split('\n').collect();
    if lines.len() != 3 {
      return None;
    }

    let package_line = lines[0];
    let arch_line = lines[1];
    let auto_installed_line = lines[2];

    let name = package_line.split(": ").collect::<Vec<&str>>()[1].into();
    let arch = arch_line.split(": ").collect::<Vec<&str>>()[1].into();
    let automatic_installed = matches!(
      auto_installed_line.split(": ").collect::<Vec<&str>>()[1],
      "1"
    );

    Some(Self {
      name,
      arch,
      automatic_installed,
    })
  }
}
