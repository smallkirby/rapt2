use super::{super::error::RaptError, InstallArgs};
use crate::{
  algorithm::dag::sort_depends,
  context::Context,
  dpkg::{
    client::{DpkgClient, StatusComp},
    installer::DpkgInstaller,
  },
  net::binary::BinaryDownloader,
  package::client::{PackageClient, PackageWithSource},
  source::client::SourceClient,
};

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

static EMOJI_SPARKLES: Emoji<'_, '_> = Emoji("✨", "");
static EMOJI_BOOKS: Emoji<'_, '_> = Emoji("📚", "");
static EMOJI_EARTH: Emoji<'_, '_> = Emoji("🌎", "");
static EMOJI_COMPUTER: Emoji<'_, '_> = Emoji("💻", "");
static EMOJI_INFORMATION: Emoji<'_, '_> = Emoji("ℹ️", "");
static EMOJI_TARGET: Emoji<'_, '_> = Emoji("🎯", "");

pub fn execute(context: &Context, args: &InstallArgs) -> Result<(), RaptError> {
  let keyword = args.keyword.clone();

  // first, search dependencies
  println!(
    "{} {} Resolving dependencies...",
    style("[1/3]").bold().dim(),
    EMOJI_BOOKS
  );
  let source_client = SourceClient::new(PathBuf::from(&context.source_dir))?;
  let sources = source_client.read_all()?;
  let package_client = PackageClient::new(PathBuf::from(&context.list_dir))?;
  let mut dpkg_client = DpkgClient::new(PathBuf::from(&context.dpkg_dir));
  let deps = package_client.get_package_with_deps(
    &keyword,
    &sources.into_iter().collect(),
    false,
    Some(&mut dpkg_client),
  )?;
  if deps.is_empty() {
    println!(
      "{} package {} is already up-to-date.",
      EMOJI_SPARKLES,
      style(keyword).cyan()
    );
    return Ok(());
  }

  let sorted_deps: Vec<PackageWithSource> =
    sort_depends(deps, &keyword).into_iter().rev().collect();
  let package_num = sorted_deps.len();

  // show info of packages
  show_to_install_packages(&sorted_deps, &keyword);

  // if dry-run, return here
  if args.dry_run {
    println!(
      "{}  This is dry run, so actuall installation is not performed.",
      EMOJI_INFORMATION
    );
    // if verbose mode, show dependencies.
    if context.verbose {
      show_deps_verbose(&sorted_deps);
    }
    return Ok(());
  }

  // ask users again to install or not
  if !confirm_user_yesno() {
    return Ok(());
  }

  // fetch all packages
  println!(
    "{} {} Fetching binary files...",
    style("[2/3]").bold().dim(),
    EMOJI_EARTH,
  );
  let prog_style = ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .progress_chars("##-");
  let progress = ProgressBar::new(sorted_deps.len() as u64);
  progress.set_style(prog_style);

  let binary_client =
    BinaryDownloader::new(sorted_deps.clone(), PathBuf::from(&context.archive_dir))?;
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

  // install them
  println!(
    "{} {} Installing packages...",
    style("[3/3]").bold().dim(),
    EMOJI_COMPUTER,
  );
  let dpkg_client = DpkgInstaller::new(PathBuf::from(&context.archive_dir), sorted_deps)?;
  {
    // extract all packages
    let prog_style = ProgressStyle::default_bar()
      .template("   extract   {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
      .progress_chars("##-");
    let progress = ProgressBar::new(package_num as u64);
    progress.set_style(prog_style);

    for extracter in dpkg_client.extracters_iter() {
      progress.set_message(format!("{}", &extracter.pws.package.name));
      extracter.execute()?;
      progress.inc(1);
    }
    progress.abandon_with_message("Complete.");
  }
  {
    // configure all packages
    let prog_style = ProgressStyle::default_bar()
      .template("   configure {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
      .progress_chars("##-");
    let progress = ProgressBar::new(package_num as u64);
    progress.set_style(prog_style);

    for configuer in dpkg_client.configuers_iter() {
      progress.set_message(format!("{}", &configuer.pws.package.name));
      configuer.execute()?;
      progress.inc(1);
    }
    progress.abandon_with_message("Complete.");
  }

  Ok(())
}

fn show_deps_verbose(sorted_deps: &Vec<PackageWithSource>) {
  println!(
    "\n{}",
    style("Packages would be installed in below order:").dim()
  );
  for pws in sorted_deps {
    let package = &pws.package;
    print!("\t- {} -> ", style(&package.name).green().dim());
    for dep in &package.depends {
      let dep_str: Vec<String> = dep
        .depends
        .iter()
        .map(|dep| {
          if let Some(version) = &dep.version {
            format!("{}({})", dep.package, version)
          } else {
            format!("{}(any)", dep.package)
          }
        })
        .collect();
      print!("{}, ", style(dep_str.join(" | ")).dim());
    }
    println!("");
  }
}

fn show_to_install_packages(pwss: &Vec<PackageWithSource>, target: &str) {
  println!(
    "Below packages are to be installed({}):",
    style(pwss.len()).bold().cyan()
  );

  println!("  {} Target: {}", EMOJI_TARGET, style(target).bold());

  // show newly installed packages.
  let news: Vec<&PackageWithSource> = pwss
    .iter()
    .filter(|pws| pws.dpkg_status == Some(StatusComp::NOTINSTALLED))
    .collect();
  println!(
    "  {} New ({}):",
    EMOJI_SPARKLES,
    style(news.len()).bold().cyan()
  );
  for new in &news {
    let package = &new.package;
    println!(
      "\t - {} ({})",
      style(&package.name).yellow(),
      style(&package.version).dim()
    );
  }
  drop(news);

  // show upgraded packages.
  let upgrades: Vec<&PackageWithSource> = pwss
    .iter()
    .filter(|pws| {
      if let Some(status) = &pws.dpkg_status {
        match status {
          StatusComp::OLD(_) => true,
          _ => false,
        }
      } else {
        false
      }
    })
    .collect();
  println!(
    "  {} Upgraded({}):",
    EMOJI_SPARKLES,
    style(upgrades.len()).bold().cyan()
  );
  for upgrade in upgrades {
    let package = &upgrade.package;
    match upgrade.dpkg_status.clone().unwrap() {
      StatusComp::OLD(old_version) => println!(
        "\t - {} ({} -> {})",
        style(&package.name).yellow(),
        style(&old_version).dim(),
        style(&package.version).dim()
      ),
      _ => println!(
        "\t - {} (-> {})",
        style(&package.name).yellow(),
        style(&package.version).dim()
      ),
    }
  }
}

fn confirm_user_yesno() -> bool {
  let mut s = String::new();
  print!("Do you really install them? [yN] > ");
  stdout().flush().unwrap();
  stdin().read_line(&mut s).expect("Invalid input");

  if s.to_lowercase().starts_with("y") {
    true
  } else {
    false
  }
}
