/*
 This file implements parse of Package file
*/

use std::collections::HashSet;
use std::str::FromStr;

use super::{error::PackageError, package::*, version::*};
use crate::util::*;

fn parse_entry(content: &str, entry_type: EntryType) -> Result<Package, PackageError> {
  if !content.starts_with("Package: ") {
    return Err(PackageError::InvalidFormat {
      msg: "given Package doesn't start with 'Package: ' field.".into(),
    });
  }

  let mut package = Package {
    ..Default::default()
  };

  let mut parsing_description = false;
  let mut parsing_conffile = false;
  let mut parsing_unknown = false; // XXX
  let mut long_description = String::new();
  let mut conffiles = vec![];
  for line in content.lines() {
    if parsing_unknown {
      if line.starts_with(" ") {
        continue;
      }
      parsing_unknown = false;
    }
    if parsing_description {
      if line.starts_with(" ") {
        long_description = long_description + line;
        continue;
      }
      package.long_description = if long_description.len() != 0 {
        Some(long_description.clone())
      } else {
        None
      };
      parsing_description = false;
    }
    if parsing_conffile {
      if line.starts_with(" ") {
        conffiles.push(line.trim().to_string());
        continue;
      }
      package.conffiles = conffiles.clone();
      parsing_conffile = false;
    }

    let parts: Vec<&str> = line.split(": ").collect();
    let section = parts[0].trim().to_string();
    let ent = if parts.len() >= 2 {
      parts[1..].join(": ")
    } else {
      "".into()
    };

    match section.to_lowercase().as_str() {
      "package" => package.name = ent,
      "version" => package.version = Version::from(&ent).unwrap(),
      "architecture" => package.arch = ent,
      "priority" => package.priority = Some(Priority::from_str(&ent).unwrap()),
      "section" => package.section = Some(ent),
      "maintainer" => package.maintainer = ent,
      "filename" => package.filename = ent,
      "size" => {
        package.size = match ent.parse::<u64>() {
          Ok(size) => size,
          Err(_) => {
            return Err(PackageError::InvalidField {
              field: section,
              value: ent,
            })
          }
        }
      }
      "md5sum" => package.md5 = ent,
      "sha1" => package.sha1 = ent,
      "sha256" => package.sha256 = ent,
      "description" => {
        package.short_description = ent;
        parsing_description = true;
      }
      "conffiles" => {
        parsing_conffile = true;
      }
      "depends" => package.depends = DependsAnyOf::from(&ent).unwrap(),
      "files" | "checksums-sha1" | "checksums-sha256" | "package-list" => parsing_unknown = true,
      _ => continue,
    }
  }

  let is_valid = match entry_type {
    EntryType::BINARY => package.valid(),
    EntryType::STATUS => package.valid_as_status(),
    EntryType::SOURCE => package.valid_as_source(),
  };

  if is_valid {
    Ok(package)
  } else {
    Err(PackageError::IncompleteEntry {
      msg: content.into(),
      typ: entry_type,
    })
  }
}

pub fn parse_entries_as_binary(entries: &str) -> Result<HashSet<Package>, PackageError> {
  do_parse_entries(entries, EntryType::BINARY)
}

pub fn parse_entries_as_source(entries: &str) -> Result<HashSet<Package>, PackageError> {
  do_parse_entries(entries, EntryType::SOURCE)
}

pub fn parse_entries_as_status(entries: &str) -> Result<HashSet<Package>, PackageError> {
  do_parse_entries(entries, EntryType::STATUS)
}

fn do_parse_entries(
  entries: &str,
  entry_type: EntryType,
) -> Result<HashSet<Package>, PackageError> {
  let blocks = split_by_empty_line(entries);
  let entries: Vec<String> = blocks.into_iter().map(|block| block.join("\n")).collect();
  let mut packages = HashSet::new();

  for entry in &entries {
    packages.insert(parse_entry(entry, entry_type.clone())?);
  }

  Ok(packages)
}

#[cfg(test)]
mod tests {
  use crate::package::version;

  use super::*;

  #[test]
  fn test_parse_entry() {
    let entry_str = "
      Package: vim
      Architecture: amd64
      Version: 2:8.1.2269-1ubuntu5
      Priority: optional
      Section: editors
      Origin: Ubuntu
      Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
      Original-Maintainer: Debian Vim Maintainers <pkg-vim-maintainers@lists.alioth.debian.org>
      Bugs: https://bugs.launchpad.net/ubuntu/+filebug
      Installed-Size: 3038
      Provides: editor
      Depends: vim-common (= 2:8.1.2269-1ubuntu5), vim-runtime (= 2:8.1.2269-1ubuntu5), libacl1 (>= 2.2.23)
      Suggests: ctags, vim-doc, vim-scripts
      Filename: pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb
      Size: 1237624
      MD5sum: 198ccbb07a9fc8ebe67a213eab6a3e96
      SHA1: 796c962d044f99a81b187211e6ce9a0a44b8d5d1
      SHA256: 1e38f267bf4c06e424b166e8d666ffd6ce25c657012892d099651bee18a2c834
      Homepage: https://www.vim.org/
      Description: Vi IMproved - enhanced vi editor
      Task: server, cloud-image, lubuntu-desktop
      Description-md5: 59e8b8f7757db8b53566d5d119872de8
    ";
    let answer = Package {
      name: "vim".into(),
      arch: "amd64".into(),
      version: Version::from("2:8.1.2269-1ubuntu5").unwrap(),
      priority: Some(Priority::OPTIONAL),
      section: Some("editors".into()),
      maintainer: "Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>".into(),
      filename: "pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb".into(),
      size: 1237624,
      md5: "198ccbb07a9fc8ebe67a213eab6a3e96".into(),
      sha1: "796c962d044f99a81b187211e6ce9a0a44b8d5d1".into(),
      sha256: "1e38f267bf4c06e424b166e8d666ffd6ce25c657012892d099651bee18a2c834".into(),
      short_description: "Vi IMproved - enhanced vi editor".into(),
      depends: vec![
        DependsAnyOf {
          depends: vec![Depends {
            package: "vim-common".into(),
            version: Some(VersionComp {
              version: Version::from("2:8.1.2269-1ubuntu5").unwrap(),
              operator: version::VersionCompOperator::EQ,
            }),
          }],
        },
        DependsAnyOf {
          depends: vec![Depends {
            package: "vim-runtime".into(),
            version: Some(VersionComp {
              version: Version::from("2:8.1.2269-1ubuntu5").unwrap(),
              operator: version::VersionCompOperator::EQ,
            }),
          }],
        },
        DependsAnyOf {
          depends: vec![Depends {
            package: "libacl1".into(),
            version: Some(VersionComp {
              version: Version::from("2.2.23").unwrap(),
              operator: version::VersionCompOperator::GE,
            }),
          }],
        },
      ],
      ..Default::default()
    };

    let package = parse_entry(entry_str.trim(), EntryType::BINARY).unwrap();
    assert_eq!(answer, package);
  }
}
