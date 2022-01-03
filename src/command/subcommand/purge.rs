/*
 This file implements `remove` subcommand.
*/

use super::{super::error::RaptError, PurgeArgs};
use crate::{
  context::Context,
  dpkg::{client::DpkgClient, status::DpkgStatusStatus},
  util::{emoji::*, *},
};

use console::style;

pub fn execute(context: &Context, args: &PurgeArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();
  // acquire lock
  let lock = acquire_lock_blocking_pretty(&context.dpkg_lock)?;

  // get installed packages
  println!("{} Reading packages lists...", EMOJI_SPARKLES);
  let mut dpkg_client = DpkgClient::new(context.dpkg_dir.clone(), context.extended_state.clone());
  let packages = dpkg_client.get_installed_packages()?;
  let target_package = packages.into_iter().find(|package| package.name == keyword);
  if target_package.is_none() {
    println!(
      "{} Package {} is not installed.",
      EMOJI_INFORMATION,
      style(keyword).cyan()
    );
    return Ok(());
  }

  // check package status
  let target_package = target_package.unwrap();
  let status = target_package.status.as_ref().unwrap().clone();
  if status.status != DpkgStatusStatus::ConfigFiles {
    return Err(RaptError::UnknownError {
      msg: format!(
        "Package {} is in invalid state: {:?}",
        target_package.name, status.status
      ),
    });
  }

  // ask user again
  if !confirm_user_yesno("Do you really purge the package?") {
    return Ok(());
  }

  // purge package
  // XXX should purge depending-on and not-depended-on packages
  dpkg_client.remove_package(&target_package, true)?;

  // release lock
  drop(lock);

  Ok(())
}
