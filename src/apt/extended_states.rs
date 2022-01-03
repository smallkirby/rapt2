/*
 This file defines client for IO of `/var/lib/apt/extended_states`.
 Refer to `/docs/extended_states.md` for the details.
*/

use crate::package::error::PackageError;
use crate::package::package::Package;

use std::fs;
use std::path::PathBuf;

pub struct AptExtendedStateClient {
  path: PathBuf,
}

impl Default for AptExtendedStateClient {
  fn default() -> Self {
    Self::new(&PathBuf::from("/var/lib/apt/extended_states"))
  }
}

impl AptExtendedStateClient {
  pub fn new(extended_state: &PathBuf) -> Self {
    Self {
      path: extended_state.clone(),
    }
  }

  pub fn read(&self) -> Result<Vec<AptExtendedPackageInfo>, PackageError> {
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

  // Update entry of apt extended_states.
  // If an entry for `package_name` exists, update its value or remove the entry.
  // NOTE: if `auto_installed` is false, it just removes the entry.
  pub fn update(&self, package_name: &str, auto_installed: bool) -> Result<(), PackageError> {
    let mut extended_infos = self.read()?;
    let target_info_ix = extended_infos
      .iter()
      .position(|info| info.name == package_name);
    let new_extended_file_str = match target_info_ix {
      Some(target_info_ix) => {
        if auto_installed {
          return Ok(());
        } else {
          extended_infos.remove(target_info_ix);
          extended_states_to_string(&extended_infos)
        }
      }
      None => {
        if auto_installed {
          let target_info = AptExtendedPackageInfo {
            name: package_name.into(),
            arch: "amd64".into(),
            automatic_installed: auto_installed,
          };
          extended_infos.push(target_info);
          extended_states_to_string(&extended_infos)
        } else {
          return Ok(());
        }
      }
    };

    std::fs::write(self.path.clone(), new_extended_file_str)?;

    Ok(())
  }
}

#[derive(Debug, Clone)]
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

  pub fn to_target_string(&self) -> String {
    format!(
      "Package: {}\nArchitecture: {}\nAuto-Installed: {}",
      self.name,
      self.arch,
      if self.automatic_installed { "1" } else { "0" }
    )
  }

  pub fn to_empty_package(&self) -> Package {
    Package {
      name: self.name.to_string(),
      arch: self.arch.to_string(),
      ..Default::default()
    }
  }
}

// Convert extended_state entries into the format of extended_state file.
// If entry's `automatic_installed` is false, just ignore the entry.
fn extended_states_to_string(infos: &Vec<AptExtendedPackageInfo>) -> String {
  let targets: Vec<&AptExtendedPackageInfo> = infos
    .into_iter()
    .filter(|info| info.automatic_installed)
    .collect();
  targets
    .into_iter()
    .map(|info| info.to_target_string())
    .collect::<Vec<String>>()
    .join("\n\n")
    + "\n"
}
