//! Cargo-runnable Dreadstep terminal showcase entry point.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
  dreadstep_tui::run(env::args().skip(1))
}
