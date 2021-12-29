use clap::{Args, Subcommand};

pub mod list;
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
  #[clap(about = "List packages by name.")]
  LIST {
    #[clap(flatten)]
    args: ListArgs,
  },
}

#[derive(Args, Debug)]
pub struct UpdateArgs {}

#[derive(Args, Debug)]
pub struct ListArgs {
  #[clap(long, help = "show only installed packages.")]
  pub installed: bool,

  #[clap(help = "Keyword to search for.")]
  pub keyword: String,
}
