/*
 This file implements parse of Package file
*/

use std::str::FromStr;

use super::{error::PackageError, package::*};
use crate::util::*;

pub fn parse_entry(content: &str) -> Result<Package, PackageError> {
  if !content.starts_with("Package: ") {
    return Err(PackageError::InvalidFormat {
      msg: "given Package doesn't start with 'Package: ' field.".into(),
    });
  }

  let mut package = Package {
    ..Default::default()
  };

  for line in content.trim().lines() {
    let parts: Vec<&str> = line.split(": ").collect();
    if parts.len() != 2 {
      return Err(PackageError::InvalidFormat { msg: line.into() });
    }
    let section = parts[0].trim().to_string();
    let ent = parts[1].trim().to_string();

    match section.to_lowercase().as_str() {
      "package" => package.name = ent,
      "version" => package.version = ent,
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
      "description" => package.short_description = ent,
      _ => continue,
    }
  }

  if !package.valid() {
    Err(PackageError::LackingField { msg: "".into() })
  } else {
    Ok(package)
  }
}

pub fn parse_entries(entries: &str) -> Result<Vec<Package>, PackageError> {
  let blocks = split_by_empty_line(entries);
  let entries: Vec<String> = blocks.into_iter().map(|block| block.join("\n")).collect();
  let mut packages = vec![];

  for entry in &entries {
    packages.push(parse_entry(entry)?);
  }

  Ok(packages)
}

#[cfg(test)]
mod tests {
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
      Depends: vim-common (= 2:8.1.2269-1ubuntu5), vim-runtime (= 2:8.1.2269-1ubuntu5), libacl1 (>= 2.2.23), libc6 (>= 2.29), libcanberra0 (>= 0.2), libgpm2 (>= 1.20.7), libpython3.8 (>= 3.8.2), libselinux1 (>= 1.32), libtinfo6 (>= 6)
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
      version: "2:8.1.2269-1ubuntu5".into(),
      priority: Some(Priority::OPTIONAL),
      section: Some("editors".into()),
      maintainer: "Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>".into(),
      filename: "pool/main/v/vim/vim_8.1.2269-1ubuntu5_amd64.deb".into(),
      size: 1237624,
      md5: "198ccbb07a9fc8ebe67a213eab6a3e96".into(),
      sha1: "796c962d044f99a81b187211e6ce9a0a44b8d5d1".into(),
      sha256: "1e38f267bf4c06e424b166e8d666ffd6ce25c657012892d099651bee18a2c834".into(),
      short_description: "Vi IMproved - enhanced vi editor".into(),
      ..Default::default()
    };

    let package = parse_entry(entry_str.trim()).unwrap();
    assert_eq!(answer, package);
  }
}
