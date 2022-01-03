/*
 This file implements `dep` subcommand.
*/

use super::{super::error::RaptError, DepArgs};
use crate::{
  context::Context,
  package::{client::PackageClient, error::PackageError},
  source::client::SourceClient,
  util::emoji::*,
};

use console::style;
use std::path::PathBuf;

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

  // show dependencies
  println!("{}  Target: {}", EMOJI_TARGET, style(&keyword).yellow());
  println!("{}  Dependencies({}):", EMOJI_DOWN, deps.len() - 1);
  for dep in deps {
    if dep.package.name == keyword {
      // don't show target package itself
      continue;
    }
    let package = dep.package;
    let source = dep.source;
    println!(
      "\t{} / {} {} {}",
      style(package.name).cyan(),
      style(source.distro).dim(),
      style(package.version).dim(),
      style(package.arch).dim(),
    );
  }

  Ok(())
}
