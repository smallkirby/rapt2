/*
 This file implements `dep` subcommand.
*/

use super::{super::error::RaptError, DepArgs};
use crate::{
  context::Context,
  package::{
    client::{PackageClient, PackageWithSource},
    error::PackageError,
    package::Package,
  },
  source::{client::SourceClient, source::Source},
  util::emoji::*,
};

use console::style;
use std::{collections::HashMap, path::PathBuf};

pub fn execute(context: &Context, args: &DepArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();
  // get sources
  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources = source_client.read_all()?;

  // get dependencies
  println!(
    "{} Searching dependencies of {}...\n",
    EMOJI_BOOKS,
    style(&keyword).cyan(),
  );
  let mut package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let deps = match package_client.get_package_with_deps(
    &keyword,
    &sources.into_iter().collect(),
    true,
    None,
  ) {
    Ok(deps) => deps,
    Err(PackageError::PackageNotFound { package_name }) => {
      println!(
        "{} Package {} not found.",
        EMOJI_CROSS,
        style(package_name).cyan()
      );
      return Ok(());
    }
    Err(err) => return Err(err.into()),
  };

  // show dependencies recursively
  let deps: HashMap<String, PackageWithSource> = deps
    .into_iter()
    .map(|pws| (pws.package.name.clone(), pws))
    .collect();
  let root = deps.get(&keyword).unwrap();
  print!("{} Target:\n    ", EMOJI_TARGET);
  show_single_package(&root.package, &root.source);
  println!(
    "{}  Depends({}):",
    EMOJI_DOWN,
    style(deps.len() - 1).yellow(),
  );

  let mut shown_packages = vec![];
  for root_depany in &root.package.depends {
    let root_dep = &root_depany.depends[0];
    if root_dep.package == keyword {
      continue;
    }
    recursive_depends_show(&root_dep.package, &deps, 1, &mut shown_packages);
  }

  Ok(())
}

fn recursive_depends_show(
  name: &str,
  packages: &HashMap<String, PackageWithSource>,
  hierarchy: usize,
  acc: &mut Vec<String>,
) {
  let target = packages.get(name).unwrap();
  if acc.contains(&target.package.name) {
    return;
  }

  // show package and source
  print!("{}", "  ".repeat(hierarchy * 2));
  show_single_package(&target.package, &target.source);

  // more recursive printing
  acc.push(target.package.name.clone());
  for dep_anyof in &target.package.depends {
    recursive_depends_show(&dep_anyof.depends[0].package, packages, hierarchy + 1, acc);
  }
}

fn show_single_package(package: &Package, source: &Source) {
  print!(
    "{} / {} {} {} -> ",
    style(&package.name).cyan(),
    style(&source.distro).dim(),
    style(&package.version).dim(),
    style(&package.arch).dim(),
  );
  // show direct dependency
  let depends_str = package
    .depends
    .iter()
    .map(|anyof| {
      if let Some(version) = &anyof.depends[0].version {
        format!(
          "{} ({})",
          style(&anyof.depends[0].package).yellow().dim(),
          style(version.to_string()).dim()
        )
      } else {
        format!(
          "{} ({})",
          style(&anyof.depends[0].package).yellow().dim(),
          style("any").dim()
        )
      }
    })
    .collect::<Vec<String>>()
    .join(", ");
  println!("{}", depends_str);
}
