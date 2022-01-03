/*
 This file defines executer of this app, named `Rapt`.
 `Rapt` holds context information and executes subcommands.
*/

use super::{error::RaptError, subcommand::*};
use crate::{context::Context, util::emoji::EMOJI_CROSS};

use console::style;

pub struct Rapt {
  context: Context,
  command: SubCommand,
}

impl Rapt {
  pub fn new(context: Context, command: SubCommand) -> Self {
    Self { context, command }
  }

  pub fn execute(&self) {
    let result = match &self.command {
      SubCommand::UPDATE { args } => update::execute(&self.context, args),
      SubCommand::LIST { args } => list::execute(&self.context, args),
      SubCommand::DEP { args } => dep::execute(&self.context, args),
      SubCommand::INSTALL { args } => install::execute(&self.context, args),
      SubCommand::UPGRADE { args } => upgrade::execute(&self.context, args),
      SubCommand::CLEAN { args } => clean::execute(&self.context, args),
      SubCommand::REMOVE { args } => remove::execute(&self.context, args),
      SubCommand::AUTOREMOVE { args } => autoremove::execute(&self.context, args),
      _ => Err(RaptError::UnknownCommand {
        command: self.command.clone(),
      }),
    };

    if let Err(err) = result {
      println!(
        "{} {}: rapt2 aborted an operation due to below error:",
        EMOJI_CROSS,
        style("Error").red().bold()
      );
      println!("{}", err.to_string());
      std::process::exit(1);
    }
  }
}
