/*
 This file implements `update` subcommand.
*/

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};

use super::super::error::RaptError;
use crate::{
  context::Context,
  net::package::PackageDownloader,
  package::{client::*, dpkg, package::*},
  source::{
    client::SourceClient,
    source::{ArchivedType, Source},
  },
};

use std::collections::HashSet;

static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");
static EMOJI_BOOKMARK: Emoji<'_, '_> = Emoji("🔖", "");
static EMOJI_GLASS: Emoji<'_, '_> = Emoji("🔍", "");
static EMOJI_SPARKLES: Emoji<'_, '_> = Emoji("✨", "");
static EMOJI_EXC: Emoji<'_, '_> = Emoji("❗", "");

pub fn execute(context: &Context) -> Result<(), RaptError> {
  // get list of sources
  println!(
    "{} {} Reading source lists...",
    style("[1/3]").bold().dim(),
    EMOJI_BOOKS,
  );
  let source_client = SourceClient::new(context.source_dir.clone())?;
  let sources = source_client.read_all()?;
  // `update` would consider only binary packages?
  let target_sources: HashSet<Source> = sources
    .into_iter()
    .filter(|source| source.archive_type == ArchivedType::DEB)
    .collect();

  // fetch Packages and save its cache.
  println!(
    "{} {} Fetching package index...",
    style("[2/3]").bold().dim(),
    EMOJI_BOOKMARK,
  );
  let prog_style = ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .progress_chars("##-");
  let progress = ProgressBar::new(target_sources.len() as u64);
  progress.set_style(prog_style);

  let mut total_packages: HashSet<Package> = HashSet::new();
  for source in target_sources {
    progress.set_message(source.packages_url());
    let downloader = PackageDownloader::new(source, context.list_dir.clone())?;
    let package_content = downloader.get()?;
    let packages = to_packages(&package_content, EntryType::BINARY)?;
    total_packages.extend(packages);
    progress.inc(1);
  }
  progress.abandon_with_message("Complete");

  // check if there are upgradable pacakges
  println!(
    "{} {} Comparing with dpkg status...",
    style("[3/3]").bold().dim(),
    EMOJI_GLASS,
  );
  let dpkg_client = dpkg::DpkgClient::new(context.dpkg_dir.clone());
  let obsolute_packages = dpkg_client.get_obsolute_packages(&total_packages)?;

  // show result
  if obsolute_packages.len() == 0 {
    println!(
      "{} {}",
      EMOJI_SPARKLES,
      style("All packages are up-to-new!").cyan(),
    );
  } else {
    println!(
      "{} {} packages are upgradable:",
      EMOJI_EXC,
      style(obsolute_packages.len()).bold(),
    );
    for package in obsolute_packages {
      let new_version = if let Some(v) = package.new_version {
        v.to_string()
      } else {
        "".into()
      };
      println!(
        "\t- {} ({} -> {})",
        style(package.package.name).dim(),
        style(package.package.version.to_string()).dim(),
        style(new_version).dim(),
      );
    }
  }

  Ok(())
}
