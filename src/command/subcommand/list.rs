/*
 This file implements `list` subcommand.
*/

use super::{super::error::RaptError, ListArgs};
use crate::{
  context::Context, package::client::PackageClient, source::client::SourceClient, util::emoji::*,
};

use console::style;
use std::path::PathBuf;

pub fn execute(context: &Context, args: &ListArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();
  // get sources
  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources = source_client.read_all()?;

  // get list of packages
  println!("{} Reading package lists...", EMOJI_BOOKS,);
  let package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let target_packages =
    package_client.search_by_name_with_source(&keyword, sources.into_iter().collect())?;

  // show result
  if target_packages.is_empty() {
    println!("{} Found no package...", EMOJI_CROSS,);
  } else {
    println!(
      "{} Found {} packages:",
      EMOJI_SPARKLES,
      style(target_packages.len()).yellow(),
    );

    for package_with_source in target_packages {
      let package = package_with_source.package;
      let source = package_with_source.source;
      println!(
        "\t{} / {} {} {}",
        style(package.name).cyan(),
        style(source.distro).dim(),
        style(package.version).dim(),
        style(package.arch).dim(),
      );
    }
  }

  Ok(())
}
