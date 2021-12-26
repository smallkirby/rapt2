use clap::{Args, Subcommand};

pub mod update;

#[derive(Debug, Subcommand)]
pub enum SubCommand {
  #[clap(about = "Update package list database.")]
  UPDATE {
    #[clap(flatten)]
    args: UpdateArgs,
  },
  #[clap(about = "(not implemented)")]
  UPGRADE,
  #[clap(about = "(not implemented)")]
  INSTALL,
  #[clap(about = "(not implemented)")]
  REMOVE,
  #[clap(about = "(not implemented)")]
  PURGE,
  #[clap(about = "(not implemented)")]
  SEARCH,
  #[clap(about = "(not implemented)")]
  SHOW,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {}
