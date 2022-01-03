/*
 This file implements `autoremove` subcommand.
*/

use super::{super::error::RaptError, AutoRemoveArgs};
use crate::{
  algorithm::graph::Graph,
  apt::extended_states::AptExtendedStateClient,
  context::Context,
  dpkg::client::DpkgClient,
  package::client::PackageClient,
  util::{emoji::*, *},
};

use console::style;

pub fn execute(context: &Context, _args: &AutoRemoveArgs) -> Result<(), RaptError> {
  // acquire lock
  let lock = acquire_lock_blocking_pretty(&context.dpkg_lock)?;

  // get auto-removable packages
  println!(
    "{} {} Checking auto removable packages...",
    EMOJI_BOOKS,
    style("[1/2]").bold().dim()
  );
  let mut dpkg_client = DpkgClient::new(context.dpkg_dir.clone(), context.extended_state.clone());
  let mut package_client = PackageClient::new(context.list_dir.clone())?;
  let mut packages = package_client.read_all()?;
  let installeds = dpkg_client.get_installed_packages()?;
  let extended_client = AptExtendedStateClient::new(&context.extended_state);
  let auto_installeds = extended_client.read()?;

  // add dpkg status to all installed packages
  for installed in &installeds {
    if let Some(package) = packages.get(installed) {
      let mut package = package.clone();
      package.status = installed.status.clone();
      packages.remove(&package);
      packages.insert(package);
    }
  }

  // construct dep tree and check if auto-installed packages are depended-on.
  let mut deptree = Graph::construct_graph(packages.clone().into_iter().collect());

  let mut auto_removables = vec![];
  for auto_installed in auto_installeds {
    if auto_installed.automatic_installed {
      if !deptree.is_depended_on(&auto_installed.name) {
        let package = installeds
          .iter()
          .find(|p| p.name == auto_installed.name)
          .unwrap()
          .clone();
        auto_removables.push(package);
      }
    }
  }

  // show result
  if auto_removables.is_empty() {
    println!("{} No packages are auto removable.", EMOJI_SPARKLES);
    return Ok(());
  }
  println!(
    "{} {} packages are auto removable:",
    EMOJI_GLASS,
    auto_removables.len()
  );
  for auto_removable in &auto_removables {
    println!("\t{}", style(&auto_removable.name).cyan());
  }

  // ask user again
  if !confirm_user_yesno("Do you really remove packages?") {
    return Ok(());
  }

  // remove them
  println!(
    "{} {} Uninstalling packages...",
    EMOJI_FIRE,
    style("[2/2]").bold().dim()
  );
  let progress = default_progbar(auto_removables.len() as u64);
  for auto_removable in auto_removables {
    progress.set_message(auto_removable.name.clone());
    dpkg_client.remove_package(&auto_removable, false)?;
    progress.inc(1);
  }
  progress.abandon_with_message("Complete.");

  // release lock
  drop(lock);

  Ok(())
}
