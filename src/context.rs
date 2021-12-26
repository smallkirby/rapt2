/*
 This file defines global context information and app argument structure.
*/

use crate::command::subcommand::SubCommand as RaptSubCommand;
use clap::Parser;
use dirs::home_dir;

use std::path::PathBuf;

#[derive(Debug)]
pub struct Context {
  pub rapt_dir: PathBuf,   // base dir of rapt2
  pub list_dir: PathBuf,   // package list dir
  pub source_dir: PathBuf, // source list dir
  pub dpkg_dir: PathBuf,   // dpkg base dir
}

impl Default for Context {
  fn default() -> Self {
    let home = home_dir().unwrap();
    let rapt_dir = home.join("rapt2");
    let list_dir = rapt_dir.join("lists");
    let source_dir = rapt_dir.join("sources");

    let dpkg_dir = PathBuf::from("/var/lib/dpkg");

    Context {
      rapt_dir,
      list_dir,
      source_dir,
      dpkg_dir,
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
}

impl Args {
  pub fn to_context(&self) -> Context {
    let home = home_dir().unwrap();

    let rapt_dir = if self.rapt_dir.is_empty() {
      home.join(".rapt2")
    } else {
      PathBuf::from(&self.rapt_dir)
    };

    let dpkg_dir = if self.dpkg_dir.is_empty() {
      PathBuf::from("/var/lib/dpkg")
    } else {
      PathBuf::from(&self.dpkg_dir)
    };

    let list_dir = if self.list_dir.is_empty() {
      rapt_dir.join("lists")
    } else {
      PathBuf::from(&self.list_dir)
    };

    let source_dir = if self.source_dir.is_empty() {
      rapt_dir.join("sources")
    } else {
      PathBuf::from(&self.source_dir)
    };

    Context {
      rapt_dir,
      list_dir,
      source_dir,
      dpkg_dir,
    }
  }
}
