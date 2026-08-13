//! Subprocess smoke tests for the headless developer CLI.

use std::process::{Command, Output};

fn run_binary(commands: &str) -> Output {
  let binary = std::env::var("CARGO_BIN_EXE_dreadstep-headless")
    .expect("Cargo should provide the headless binary path");
  Command::new(binary)
    .args(["--seed", "7", "--commands", commands])
    .output()
    .expect("headless binary should launch")
}

#[test]
fn binary_smoke_test_runs_a_valid_scenario() {
  let output = run_binary("equip:1:103,wait:2,attack:1:2,wait:2,attack:1:2");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("CLI output should be UTF-8");
  assert!(stdout.contains("seed=7"));
  assert!(stdout.contains("digest="));
  assert!(stdout.contains("outcome=victory"));
}

#[test]
fn binary_smoke_test_runs_reload_command() {
  let output = run_binary("reload:1");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("CLI output should be UTF-8");
  assert!(stdout.contains("Reloaded"));
}

#[test]
fn binary_smoke_test_runs_drop_command() {
  let output = run_binary("drop:1:101");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("CLI output should be UTF-8");
  assert!(stdout.contains("ItemDropped"));
}

#[test]
fn binary_smoke_test_runs_frost_flask_throw() {
  let output = run_binary("throw:1:104:2");

  assert!(output.status.success());
  let stdout = String::from_utf8(output.stdout).expect("CLI output should be UTF-8");
  assert!(stdout.contains("ItemThrown"));
  assert!(stdout.contains("StatusApplied"));
}

#[test]
fn binary_maps_cast_chill_to_the_typed_core_rejection() {
  let output = run_binary("cast_chill:1:2");

  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  assert_eq!(
    String::from_utf8(output.stderr).expect("CLI errors should be UTF-8"),
    "error: command 0 rejected: actor 1 cannot cast Chill because only Frostcasters may cast it\n"
  );
}

#[test]
fn binary_reports_malformed_input_with_structured_process_error() {
  let output = run_binary("bad");

  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  assert_eq!(
    String::from_utf8(output.stderr).expect("CLI errors should be UTF-8"),
    "error: invalid command token bad\n"
  );
}

#[test]
fn binary_reports_core_rejection_with_structured_process_error() {
  let output = run_binary("move:2:east");

  assert_eq!(output.status.code(), Some(2));
  assert!(output.stdout.is_empty());
  assert_eq!(
    String::from_utf8(output.stderr).expect("CLI errors should be UTF-8"),
    "error: command 0 rejected: actor 2 is not scheduled; actor 1 must act next\n"
  );
}
