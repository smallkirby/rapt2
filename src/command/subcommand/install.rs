use super::{super::error::RaptError, InstallArgs};
use crate::{
  algorithm::dag::sort_depends,
  context::Context,
  dpkg::client::DpkgClient,
  net::binary::BinaryDownloader,
  package::client::{PackageClient, PackageWithSource},
  source::client::SourceClient,
};

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

static EMOJI_SPARKLES: Emoji<'_, '_> = Emoji("✨", "");
static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");
static EMOJI_EARTH: Emoji<'_, '_> = Emoji("🌎", "");
static EMOJI_COMPUTER: Emoji<'_, '_> = Emoji("💻", "");

pub fn execute(context: &Context, args: &InstallArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();

  // first, search dependencies
  println!(
    "{} {} Resolving dependencies...",
    style("[1/3]").bold().dim(),
    EMOJI_BOOKS
  );
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
  if deps.is_empty() {
    println!(
      "{} package {} is already up-to-date.",
      EMOJI_SPARKLES,
      style(keyword).cyan()
    );
    return Ok(());
  }

  let sorted_deps: Vec<PackageWithSource> = sort_depends(deps).into_iter().rev().collect();

  // fetch all packages
  println!(
    "{} {} Fetching binary files...",
    style("[2/3]").bold().dim(),
    EMOJI_EARTH,
  );
  let prog_style = ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .progress_chars("##-");
  let progress = ProgressBar::new(sorted_deps.len() as u64);
  progress.set_style(prog_style);

  let binary_client =
    BinaryDownloader::new(sorted_deps.clone(), PathBuf::from(&context.archive_dir))?;
  for downloader in binary_client.into_iter() {
    progress.set_message(
      style(downloader.pws.package.name.clone())
        .cyan()
        .to_string(),
    );
    downloader.download()?;
    progress.inc(1);
  }
  progress.abandon_with_message("Complete.");

  // install them
  println!(
    "{} {} Installing packages...",
    style("[3/3]").bold().dim(),
    EMOJI_COMPUTER,
  );
  let dpkg_client = DpkgClient::new(PathBuf::from(&context.dpkg_dir));
  dpkg_client.install_packages(&sorted_deps, PathBuf::from(&context.archive_dir))?;

  Ok(())
}
