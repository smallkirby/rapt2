use super::{super::error::RaptError, UpgradeArgs};
use crate::{
  algorithm::dag::*,
  context::Context,
  dpkg::{client::DpkgClient, installer::DpkgInstaller},
  net::binary::BinaryDownloader,
  package::client::{PackageClient, PackageWithSource},
  source::{client::SourceClient, source::Source},
  util::{emoji::*, *},
};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

pub fn execute(context: &Context, _args: &UpgradeArgs) -> Result<(), RaptError> {
  // acquire lock
  let lock = acquire_lock_blocking_pretty(&context.lists_lock)?;

  // first, check upgradable packages
  println!(
    "{} {} Checking upgradable packages...",
    style("[1/3]").bold().dim(),
    EMOJI_BOOKS
  );

  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources: Vec<Source> = source_client.read_all()?.into_iter().collect();
  let mut package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let pwss = package_client.read_all_from_source_with_source(&sources)?;
  let mut dpkg_client = DpkgClient::new(
    PathBuf::from(&context.dpkg_dir),
    context.extended_state.clone(),
  );
  let obsolute_packages =
    dpkg_client.get_obsolute_packages(&pwss.iter().map(|pws| pws.package.clone()).collect())?;
  if obsolute_packages.is_empty() {
    println!(
      "{} {}",
      EMOJI_SPARKLES,
      style("All packages are up-to-new!").cyan(),
    );
    return Ok(());
  }

  // show upgradable packages
  println!(
    "{} {} packages are upgradable:",
    EMOJI_EXC,
    style(obsolute_packages.len()).cyan().bold(),
  );
  for package in &obsolute_packages {
    let new_version = if let Some(v) = &package.new_version {
      v.to_string()
    } else {
      "".into()
    };
    println!(
      "\t- {} ({} -> {})",
      style(package.package.name.clone()).yellow(),
      style(package.package.version.to_string()).dim(),
      style(new_version).dim(),
    );
  }

  // sort and make a layer for upgradable packages
  // NOTE: there mustn't be other dependencies other than these.
  let mut target_pwss: Vec<PackageWithSource> = obsolute_packages
    .into_iter()
    .map(|package| {
      pwss
        .iter()
        .find(|pws| pws.package.name == package.package.name)
        .unwrap()
        .clone()
    })
    .collect();
  let mut sorted_pwss = vec![];
  loop {
    if target_pwss.is_empty() {
      break;
    }
    // sort node reachable from current target
    let current_sorted_pwss: Vec<PackageWithSource> = sort_depends(
      target_pwss.clone().into_iter().collect(),
      &target_pwss[0].package.name,
    )?;
    // remove sorted targets
    let mut removable_ixs = vec![];
    for (ix, pws) in target_pwss.iter().enumerate() {
      if current_sorted_pwss.contains(pws) {
        removable_ixs.push(ix);
      }
    }
    for removable_ix in removable_ixs.into_iter().rev() {
      target_pwss.remove(removable_ix);
    }
    // push to targets (push to head)
    let tmp = sorted_pwss.clone();
    sorted_pwss = current_sorted_pwss;
    sorted_pwss.extend(tmp);
  }
  drop(target_pwss);
  let layers = split_layers(&sorted_pwss);

  if context.verbose {
    show_deps_verbose(&layers);
  }

  // ask users again to upgrade or not
  if !confirm_user_yesno("Do you really upgrade them?") {
    return Ok(());
  }

  // fetch all packages
  println!(
    "{} {} Fetching binary files...",
    style("[2/3]").bold().dim(),
    EMOJI_EARTH,
  );
  let progress = default_progbar(sorted_pwss.len() as u64);

  let binary_client = BinaryDownloader::new(
    sorted_pwss.clone().into_iter().collect(),
    PathBuf::from(&context.archive_dir),
  )?;
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

  // release lock
  drop(lock);

  // install them
  println!(
    "{} {} Installing packages...",
    style("[3/3]").bold().dim(),
    EMOJI_COMPUTER,
  );

  let prog_style = ProgressStyle::default_bar()
    .template(" install   {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .progress_chars("##-");
  let progress = ProgressBar::new(sorted_pwss.len() as u64 * 2);
  progress.set_style(prog_style);

  for layer in layers.into_iter().rev() {
    let dpkg_client = DpkgInstaller::new(
      PathBuf::from(&context.archive_dir),
      layer.into_iter().rev().collect(),
      vec![],
      context.extended_state.clone(),
    )?;
    for extracter in dpkg_client.extracters_iter() {
      progress.set_message(extracter.pws.package.name.clone());
      extracter.execute()?;
      progress.inc(1);
    }
    for configuer in dpkg_client.configuers_iter() {
      progress.set_message(configuer.pws.package.name.clone());
      configuer.execute()?;
      progress.inc(1);
    }
  }
  progress.abandon_with_message("Complete.");

  Ok(())
}
