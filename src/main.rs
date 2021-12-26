use rapt2::command::rapt::Rapt;
use rapt2::context::{Args, Context};

use clap::Parser;

fn main() {
  let args = Args::parse();
  let context = args.to_context();
  let rapt = Rapt::new(context, args.command);

  rapt.execute();
}
