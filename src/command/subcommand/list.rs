/*
 This file implements `list` subcommand.
*/

use super::{super::error::RaptError, ListArgs};
use crate::{
  context::Context,
  package::{client::PackageClient, package::Package},
  source::client::SourceClient,
};

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");
static EMOJI_SPARKLES: Emoji<'_, '_> = Emoji("✨", "");
static EMOJI_CROSS: Emoji<'_, '_> = Emoji("❌", "");

pub fn execute(context: &Context, args: &ListArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();
  let pattern = match glob::Pattern::new(&keyword) {
    Ok(pattern) => pattern,
    Err(_) => {
      return Err(RaptError::InvalidInput {
        msg: format!("invalid glob: {}", keyword),
      })
    }
  };

  // get sources
  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources = source_client.read_all()?;

  // get list of packages
  println!("{} Reading package lists...", EMOJI_BOOKS,);
  let package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let packages = package_client.read_all_from_source(&sources.into_iter().collect())?;
  let target_packages: Vec<&Package> = packages
    .iter()
    .filter(|package| pattern.matches(&package.name))
    .collect();

  // show result

  if target_packages.is_empty() {
    println!("{} Found no package...", EMOJI_CROSS,);
  } else {
    println!(
      "{} Found {} packages:",
      EMOJI_SPARKLES,
      style(target_packages.len()).cyan(),
    );

    for package in target_packages {
      println!("\t{}", package.name);
    }
  }

  Ok(())
}
