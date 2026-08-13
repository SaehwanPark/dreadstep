//! Desktop process argument parsing.

use std::ffi::OsString;
use std::path::PathBuf;

pub(crate) const USAGE: &str = "Usage: dreadstep [--seed <u64>] [--procedural] [--depth <u32>] [--log-dir <path>] [--smoke] [--help]";

/// Parsed command-line options for the desktop process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopOptions {
  /// Seed passed to the deterministic starter-item scenario.
  pub seed: u64,
  /// Start the visible client from the seeded procedural floor instead of the item fixture.
  pub procedural: bool,
  /// One-based authored depth passed to the procedural floor generator.
  pub depth: u32,
  /// Directory in which this run's JSONL journal is created.
  pub log_dir: PathBuf,
  /// Run the display-free deterministic showcase sequence.
  pub smoke: bool,
}

impl Default for DesktopOptions {
  fn default() -> Self {
    Self {
      seed: 7,
      procedural: false,
      depth: 1,
      log_dir: PathBuf::from("dreadstep-logs"),
      smoke: false,
    }
  }
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ParseResult {
  Help,
  Options(DesktopOptions),
}

#[expect(
  clippy::too_many_lines,
  reason = "the desktop process keeps its small exhaustive CLI grammar in one parser"
)]
pub(crate) fn parse_options<I>(arguments: I) -> Result<ParseResult, String>
where
  I: IntoIterator<Item = OsString>,
{
  let mut options = DesktopOptions::default();
  let mut seed_seen = false;
  let mut procedural_seen = false;
  let mut depth_seen = false;
  let mut log_dir_seen = false;
  let mut smoke_seen = false;
  let mut help_seen = false;
  let mut iter = arguments.into_iter();
  while let Some(argument) = iter.next() {
    if argument == "--help" || argument == "-h" {
      if help_seen {
        return Err("duplicate --help".to_string());
      }
      help_seen = true;
      continue;
    }
    if argument == "--smoke" {
      if smoke_seen {
        return Err("duplicate --smoke".to_string());
      }
      smoke_seen = true;
      options.smoke = true;
      continue;
    }
    if argument == "--procedural" {
      if procedural_seen {
        return Err("duplicate --procedural".to_string());
      }
      procedural_seen = true;
      options.procedural = true;
      continue;
    }
    if argument == "--seed" {
      if seed_seen {
        return Err("duplicate --seed".to_string());
      }
      seed_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--seed requires a value".to_string())?;
      if matches!(
        value.to_str(),
        Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
      ) {
        return Err("--seed requires a value".to_string());
      }
      let value = value
        .into_string()
        .map_err(|_| "--seed must be an unsigned integer".to_string())?;
      options.seed = value
        .parse::<u64>()
        .map_err(|_| "--seed must be an unsigned integer".to_string())?;
      continue;
    }
    if argument == "--depth" {
      if depth_seen {
        return Err("duplicate --depth".to_string());
      }
      depth_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--depth requires a value".to_string())?;
      if matches!(
        value.to_str(),
        Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
      ) {
        return Err("--depth requires a value".to_string());
      }
      let value = value
        .into_string()
        .map_err(|_| "--depth must be an unsigned integer".to_string())?;
      options.depth = value
        .parse::<u32>()
        .map_err(|_| "--depth must be an unsigned integer".to_string())?;
      continue;
    }
    if argument == "--log-dir" {
      if log_dir_seen {
        return Err("duplicate --log-dir".to_string());
      }
      log_dir_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--log-dir requires a path".to_string())?;
      if value.is_empty()
        || matches!(
          value.to_str(),
          Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
        )
      {
        return Err("--log-dir requires a path".to_string());
      }
      options.log_dir = PathBuf::from(value);
      continue;
    }
    let display = argument.to_string_lossy();
    return Err(format!("unknown argument {display}"));
  }
  if help_seen {
    Ok(ParseResult::Help)
  } else {
    Ok(ParseResult::Options(options))
  }
}
