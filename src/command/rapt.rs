/*
 This file defines executer of this app, named `Rapt`.
 `Rapt` holds context information and executes subcommands.
*/

use super::subcommand::*;
use crate::context::Context;

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
      _ => unimplemented!(),
    };

    if let Err(err) = result {
      unimplemented!("{:?}", err);
    }
  }
}
