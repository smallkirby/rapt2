/*
 This file implements `update` subcommand.
*/

use super::{super::error::RaptError, UpdateArgs};
use crate::{
  context::Context,
  dpkg,
  net::package::PackageDownloadClient,
  package::{client::*, package::*},
  source::{
    client::SourceClient,
    source::{ArchivedType, Source},
  },
  util::{emoji::*, *},
};

use console::style;
use std::collections::HashSet;

pub fn execute(context: &Context, _args: &UpdateArgs) -> Result<(), RaptError> {
  // acquire lock
  let lock = acquire_lock_blocking_pretty(&context.lists_lock)?;

  // get list of sources
  println!(
    "{} {} Reading source lists...",
    style("[1/4]").bold().dim(),
    EMOJI_BOOKS,
  );
  let source_client = SourceClient::new(context.source_dir.clone())?;
  let sources = source_client.read_all()?;
  // `update` would consider only binary packages?
  let target_sources: Vec<Source> = sources
    .into_iter()
    .filter(|source| source.archive_type == ArchivedType::DEB)
    .collect();

  let total_sources_num = target_sources.len();
  let mut total_packages: HashSet<Package> = HashSet::new();
  let mut downloader = PackageDownloadClient::new(target_sources, context.list_dir.clone())?;

  // fetch InRelease and save its cache.
  println!(
    "{} {} Fetching InRelease index...",
    style("[2/4]").bold().dim(),
    EMOJI_BOOKMARK,
  );
  let progress = default_progbar(total_sources_num as u64);
  loop {
    progress.set_position(downloader.get_done_inrelease_num() as u64);
    match downloader.get_next_target_source_inrelease() {
      Some(source) => {
        progress.set_message(source.inrelease_url());
        let _ = downloader.get_inrelease_ifneed().unwrap();
      }
      None => break,
    }
  }
  progress.abandon_with_message("Complete");

  // fetch all packages
  println!(
    "{} {} Fetching package index...",
    style("[3/4]").bold().dim(),
    EMOJI_BOOKMARK,
  );
  let progress = default_progbar(total_sources_num as u64);
  loop {
    progress.set_position(downloader.get_done_packages_num() as u64);
    match downloader.get_next_target_source_packages() {
      Some(source) => {
        progress.set_message(source.packages_url());
        let package_content = downloader.get_package_ifneed()?.unwrap();
        let packages = to_packages(&package_content, EntryType::BINARY)?;
        Package::extend(&mut total_packages, packages);
      }
      None => break,
    }
  }
  progress.abandon_with_message("Complete");

  // release lock
  drop(lock);

  // check if there are upgradable pacakges
  println!(
    "{} {} Comparing with dpkg status...",
    style("[4/4]").bold().dim(),
    EMOJI_GLASS,
  );
  let mut dpkg_client =
    dpkg::client::DpkgClient::new(context.dpkg_dir.clone(), context.extended_state.clone());
  let obsolute_packages = dpkg_client.get_obsolute_packages(&total_packages)?;

  // show result
  if obsolute_packages.is_empty() {
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
        style(package.package.name).yellow(),
        style(package.package.version.to_string()).dim(),
        style(new_version).dim(),
      );
    }
  }

  Ok(())
}
