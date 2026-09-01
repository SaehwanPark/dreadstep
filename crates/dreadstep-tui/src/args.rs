//! Bounded command-line options for the terminal showcase.

use std::path::PathBuf;

/// Parsed launcher options.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
  /// Explicit run seed.
  pub seed: u64,
  /// When true, start a procedural floor instead of the item showcase.
  pub procedural: bool,
  /// Procedural depth; ignored for the item showcase.
  pub depth: u32,
  /// Directory for JSONL journals.
  pub log_dir: PathBuf,
  /// Display-free exhaustive smoke runner.
  pub smoke: bool,
  /// Print plain frames to stdout instead of using an alternate screen.
  pub print_frames: bool,
  /// Skip the wall-clock enemy delay.
  pub no_delay: bool,
  /// Optional directory that receives README screenshot goldens.
  pub capture_dir: Option<PathBuf>,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      seed: 7,
      procedural: false,
      depth: 1,
      log_dir: PathBuf::from("dreadstep-logs"),
      smoke: false,
      print_frames: false,
      no_delay: false,
      capture_dir: None,
    }
  }
}

/// CLI parse failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
  /// A flag was supplied without its value.
  MissingValue(&'static str),
  /// A numeric flag could not be parsed.
  InvalidNumber(&'static str, String),
  /// An unsupported flag.
  UnknownArgument(String),
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingValue(flag) => write!(formatter, "missing value for {flag}"),
      Self::InvalidNumber(flag, value) => write!(formatter, "invalid {flag} value {value}"),
      Self::UnknownArgument(argument) => write!(formatter, "unknown argument {argument}"),
    }
  }
}

impl std::error::Error for ParseError {}

/// Usage text printed for `--help`.
pub const USAGE: &str = "\
Usage: dreadstep-tui [options]

  --seed <u64>         run seed (default 7)
  --procedural         start a seeded procedural floor
  --depth <u32>        procedural depth (default 1)
  --log-dir <path>     JSONL journal directory (default dreadstep-logs)
  --smoke              display-free command/event coverage gate
  --print-frames       print plain frames to stdout (default when stdin is not a TTY)
  --no-delay           execute enemy turns immediately
  --capture <dir>      write item-showcase screenshots then exit (not with --procedural)
  --help               show this help
";

/// Parsed launcher result, including `--help`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseResult {
  /// `--help` was requested.
  Help,
  /// Parsed options.
  Options(Options),
}

/// Parses terminal-client arguments.
///
/// # Errors
///
/// Returns [`ParseError`] for unknown flags or invalid values.
pub fn parse_options<I, S>(args: I) -> Result<ParseResult, ParseError>
where
  I: IntoIterator<Item = S>,
  S: AsRef<str>,
{
  let mut options = Options::default();
  let mut args = args.into_iter();
  while let Some(argument) = args.next() {
    let argument = argument.as_ref();
    match argument {
      "--help" | "-h" => return Ok(ParseResult::Help),
      "--seed" => {
        options.seed = parse_number("--seed", next_value(&mut args, "--seed")?)?;
      }
      "--depth" => {
        options.depth = parse_number("--depth", next_value(&mut args, "--depth")?)?;
      }
      "--log-dir" => {
        options.log_dir = PathBuf::from(next_value(&mut args, "--log-dir")?);
      }
      "--capture" => {
        options.capture_dir = Some(PathBuf::from(next_value(&mut args, "--capture")?));
      }
      "--procedural" => options.procedural = true,
      "--smoke" => options.smoke = true,
      "--print-frames" => options.print_frames = true,
      "--no-delay" => options.no_delay = true,
      other => return Err(ParseError::UnknownArgument(other.to_string())),
    }
  }
  Ok(ParseResult::Options(options))
}

fn next_value<I, S>(args: &mut I, flag: &'static str) -> Result<String, ParseError>
where
  I: Iterator<Item = S>,
  S: AsRef<str>,
{
  args
    .next()
    .map(|value| value.as_ref().to_string())
    .ok_or(ParseError::MissingValue(flag))
}

fn parse_number<T: std::str::FromStr>(flag: &'static str, value: String) -> Result<T, ParseError> {
  value
    .parse()
    .map_err(|_| ParseError::InvalidNumber(flag, value))
}

#[cfg(test)]
mod tests {
  use super::{ParseResult, parse_options};

  #[test]
  fn parses_seed_and_print_frames() {
    let ParseResult::Options(options) =
      parse_options(["--seed", "11", "--print-frames"]).expect("parse")
    else {
      panic!("expected options");
    };
    assert_eq!(options.seed, 11);
    assert!(options.print_frames);
  }
}
