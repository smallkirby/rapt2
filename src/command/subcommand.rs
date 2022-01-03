use clap::{Args, Subcommand};

pub mod autoremove;
pub mod clean;
pub mod dep;
pub mod install;
pub mod list;
pub mod remove;
pub mod update;
pub mod upgrade;

#[derive(Debug, Subcommand)]
pub enum SubCommand {
  #[clap(about = "Update package list database.")]
  UPDATE {
    #[clap(flatten)]
    args: UpdateArgs,
  },
  #[clap(about = "Install newer version of packages.")]
  UPGRADE {
    #[clap(flatten)]
    args: UpgradeArgs,
  },
  #[clap(about = "Install packages")]
  INSTALL {
    #[clap(flatten)]
    args: InstallArgs,
  },
  #[clap(about = "Remove packages.")]
  REMOVE {
    #[clap(flatten)]
    args: RemoveArgs,
  },
  #[clap(about = "Remove automatically installed, but unsed packages.")]
  AUTOREMOVE {
    #[clap(flatten)]
    args: AutoRemoveArgs,
  },
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
  #[clap(about = "Recursively search dependencies of a package.")]
  DEP {
    #[clap(flatten)]
    args: DepArgs,
  },
  #[clap(about = "Clean cached binary packages.")]
  CLEAN {
    #[clap(flatten)]
    args: CleanArgs,
  },
}

#[derive(Args, Debug)]
pub struct UpdateArgs {}

#[derive(Args, Debug)]
pub struct UpgradeArgs {}

#[derive(Args, Debug)]
pub struct ListArgs {
  #[clap(long, help = "show only installed packages.")]
  pub installed: bool,

  #[clap(help = "Keyword to search for.")]
  pub keyword: String,
}

#[derive(Args, Debug)]
pub struct DepArgs {
  #[clap(help = "Target package name.")]
  pub keyword: String,
}

#[derive(Args, Debug)]
pub struct InstallArgs {
  #[clap(help = "Target package name.")]
  pub keyword: String,

  #[clap(short = 'N', long, help = "Dry run.")]
  pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Args, Debug)]
pub struct RemoveArgs {
  #[clap(help = "Target package name.")]
  pub keyword: String,
}

#[derive(Args, Debug)]
pub struct AutoRemoveArgs {}
