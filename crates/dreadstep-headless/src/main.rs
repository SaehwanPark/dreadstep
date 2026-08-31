//! Process entry point for the deterministic headless developer CLI.

use std::{env, process::ExitCode};

fn main() -> ExitCode {
  let args: Vec<_> = env::args().skip(1).collect();
  if args
    .first()
    .is_some_and(|argument| argument == "--verify-replay")
  {
    if args.len() != 2 {
      eprintln!("error: --verify-replay requires exactly one path");
      return ExitCode::from(2);
    }
    return match dreadstep_headless::verify_replay_file(&args[1]) {
      Ok(export) => {
        print!(
          "{}",
          dreadstep_headless::render_replay_verification(&export)
        );
        ExitCode::SUCCESS
      }
      Err(error) => {
        eprintln!("error: {error}");
        ExitCode::from(2)
      }
    };
  }
  match dreadstep_headless::execute(args) {
    Ok(output) => {
      print!("{}", output.render());
      ExitCode::SUCCESS
    }
    Err(error) => {
      eprintln!("error: {error}");
      ExitCode::from(2)
    }
  }
}
