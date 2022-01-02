/*
 This file defines global context information and app argument structure.
*/

use crate::command::subcommand::SubCommand as RaptSubCommand;
use clap::Parser;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Context {
  pub list_dir: PathBuf,    // package list dir
  pub source_dir: PathBuf,  // source list dir
  pub dpkg_dir: PathBuf,    // dpkg base dir
  pub dpkg_lock: PathBuf,   // dpkg frontend lock
  pub lists_lock: PathBuf,  // list cache lock
  pub archive_dir: PathBuf, // binary deb file archive dir
  pub verbose: bool,        // verbose output flag
}

impl Default for Context {
  fn default() -> Self {
    let list_dir = PathBuf::from("/var/lib/apt/lists.rapt2");
    let source_dir = PathBuf::from("/etc/apt");
    let archive_dir = PathBuf::from("/var/cache/apt/archives");

    let dpkg_dir = PathBuf::from("/var/lib/dpkg");
    let dpkg_lock = dpkg_dir.join("lock-frontend");

    let lists_lock = PathBuf::from("/var/lib/apt/lists/lock"); // share with apt

    Context {
      list_dir,
      source_dir,
      dpkg_dir,
      lists_lock,
      archive_dir,
      dpkg_lock,
      verbose: false,
    }
  }
}

#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {
  #[clap(subcommand)]
  pub command: RaptSubCommand,

  #[clap(long, help = "dpkg base directory", default_value = "/var/lib/dpkg")]
  pub dpkg_dir: String,

  #[clap(long, help = "rapt2 base directory", default_value = "")]
  pub rapt_dir: String,

  #[clap(long, help = "sources list directory", default_value = "")]
  pub source_dir: String,

  #[clap(long, help = "package database directory", default_value = "")]
  pub list_dir: String,

  #[clap(long, help = ".deb archive cache directory", default_value = "")]
  pub archive_dir: String,

  #[clap(long, help = "dpkg frontend lock file", default_value = "")]
  pub dpkg_lock: String,

  #[clap(long, help = "verbose output.")]
  pub verbose: bool,
}

impl Args {
  pub fn to_context(&self) -> Context {
    let mut context = Context {
      ..Default::default()
    };

    if !self.dpkg_dir.is_empty() {
      context.dpkg_dir = PathBuf::from(&self.dpkg_dir)
    };

    if !self.list_dir.is_empty() {
      context.list_dir = PathBuf::from(&self.list_dir)
    };

    if !self.source_dir.is_empty() {
      context.source_dir = PathBuf::from(&self.source_dir)
    };

    if !self.archive_dir.is_empty() {
      context.archive_dir = PathBuf::from(&self.archive_dir)
    };

    if !self.dpkg_lock.is_empty() {
      context.dpkg_lock = PathBuf::from(&self.dpkg_lock)
    };

    if self.verbose {
      context.verbose = true;
    };

    context
  }
}
