//! Process entry point for the deterministic headless developer CLI.

use std::{env, process::ExitCode};

fn main() -> ExitCode {
  match dreadstep_headless::execute(env::args().skip(1)) {
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
