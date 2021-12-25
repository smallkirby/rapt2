/*
 This file implements `update` subcommand.
*/

use super::super::error::RaptError;
use crate::{
  context::Context,
  net::package::PackageDownloader,
  package::{client::*, package::*},
  source::client::SourceClient,
};

use std::collections::HashSet;

pub fn execute(context: &Context) -> Result<(), RaptError> {
  // get list of sources
  let source_client = SourceClient::new(context.source_dir.clone())?;
  let sources = source_client.read_all()?;

  // fetch Packages and save its cache.
  let mut total_packages: HashSet<Package> = HashSet::new();
  for source in sources {
    let downloader = PackageDownloader::new(source, context.list_dir.clone())?;
    let package_content = downloader.get()?;
    let packages = to_packages(&package_content)?;
    total_packages.extend(packages);
  }

  // get already-installed packages information
  unimplemented!();

  // check if there are upgradable pacakges
  unimplemented!();

  Ok(())
}
