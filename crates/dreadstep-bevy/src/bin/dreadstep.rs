//! Cargo-runnable Dreadstep desktop showcase entry point.

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
  dreadstep_bevy::desktop::run(env::args_os().skip(1))
}
