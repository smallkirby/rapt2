/*
 This file implements `remove` subcommand.
*/

use super::{super::error::RaptError, RemoveArgs};
use crate::{
  context::Context,
  dpkg::{client::DpkgClient, status::DpkgStatusStatus},
  util::{emoji::*, *},
};

use console::style;

pub fn execute(context: &Context, args: &RemoveArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();
  // acquire lock
  let lock = acquire_lock_blocking_pretty(&context.dpkg_lock)?;

  // get dpkg status of packages
  println!("{} Reading packages lists...", EMOJI_SPARKLES);
  let mut dpkg_client = DpkgClient::new(context.dpkg_dir.clone(), context.extended_state.clone());
  let packages = dpkg_client.get_installed_packages()?;
  let target_package = packages.into_iter().find(|package| package.name == keyword);
  // XXX should distinguish between non-existing and non~installed.
  if target_package.is_none() {
    println!(
      "{} Package {} is not installed.",
      EMOJI_INFORMATION,
      style(keyword).cyan()
    );
    return Ok(());
  }

  // ask again
  if !confirm_user_yesno("Do you really remove?") {
    return Ok(());
  }

  // remove package
  // XXX should update extended-state?
  let target_package = target_package.unwrap();
  if let Some(status) = &target_package.status {
    if status.status != DpkgStatusStatus::Installed {
      println!(
        "{} Package {} is already removed. No need to remove again",
        EMOJI_INFORMATION,
        style(keyword).cyan()
      );
      return Ok(());
    }
  }
  dpkg_client.remove_package(&target_package, false)?;

  // release lock
  drop(lock);

  // show result
  println!(
    "{} Successfully removed {}.",
    EMOJI_FIRE,
    style(target_package.name).cyan(),
  );

  Ok(())
}
