/*
 This file implements `clean` subcommand.
*/

use super::{super::error::RaptError, CleanArgs};
use crate::{
  context::Context,
  package::client::*,
  util::{emoji::*, *},
};

use console::style;

pub fn execute(context: &Context, _args: &CleanArgs) -> Result<(), RaptError> {
  // acquire lock
  let lock_path = context.archive_dir.join("lock");
  let lock = acquire_lock_blocking_pretty(&lock_path)?;

  // remove cache
  println!("{} Removing deb file cache...", EMOJI_FIRE);
  let package_client = PackageClient::new(context.list_dir.clone())?;
  let removed_count = package_client.remove_deb_caches(&context.archive_dir)?;

  // release lock
  drop(lock);

  // show result
  println!(
    "{} Removed {} files.",
    EMOJI_SPARKLES,
    style(removed_count).yellow().bold()
  );

  Ok(())
}
