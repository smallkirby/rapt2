use super::{super::error::RaptError, InstallArgs};
use crate::{
  algorithm::dag::sort_depends, context::Context, dpkg::client::DpkgClient,
  package::client::PackageClient, source::client::SourceClient,
};

use console::{style, Emoji};
use std::path::PathBuf;

static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");

pub fn execute(context: &Context, args: &InstallArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();

  // first, search dependencies
  println!("{} Resolving dependencies...", EMOJI_BOOKS);
  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources = source_client.read_all()?;
  let package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let mut dpkg_client = DpkgClient::new(PathBuf::from(&context.dpkg_dir));
  let deps = package_client.get_package_with_deps(
    &keyword,
    &sources.into_iter().collect(),
    false,
    Some(&mut dpkg_client),
  )?;

  let sorted_deps = sort_depends(deps); // XXX

  unimplemented!()
}
