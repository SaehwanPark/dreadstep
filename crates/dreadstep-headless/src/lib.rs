//! Headless execution boundary for Dreadstep.
//!
//! This crate owns command-line parsing, fixed developer scenario setup, and stdout-facing
//! output while delegating every game decision to `dreadstep-core`.

#![forbid(unsafe_code)]

use std::{error::Error, fmt, fmt::Write as _};

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Direction, GridMap, HitPoints, Item,
  ItemDefinitionId, ItemId, Position, RunOutcome, StateDigest, Tile, WorldState,
};

/// Parsed command-line input for the fixed developer scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliInput {
  seed: u64,
  commands: Vec<Command>,
}

impl CliInput {
  /// Returns the explicit run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns semantic commands in their requested execution order.
  #[must_use]
  pub fn commands(&self) -> &[Command] {
    &self.commands
  }
}

/// Errors produced while parsing headless CLI arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliError {
  /// A required flag was not supplied.
  MissingArgument(&'static str),
  /// A flag was supplied more than once.
  DuplicateArgument(&'static str),
  /// A flag was supplied without its value.
  MissingValue(&'static str),
  /// An argument flag is not supported by this bounded CLI.
  UnknownArgument(String),
  /// The seed value is not an unsigned 64-bit integer.
  InvalidSeed(String),
  /// A command token is empty or malformed.
  InvalidCommand(String),
}

impl fmt::Display for CliError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MissingArgument(argument) => write!(formatter, "missing required argument {argument}"),
      Self::DuplicateArgument(argument) => write!(formatter, "duplicate argument {argument}"),
      Self::MissingValue(argument) => write!(formatter, "missing value for {argument}"),
      Self::UnknownArgument(argument) => write!(formatter, "unknown argument {argument}"),
      Self::InvalidSeed(seed) => write!(formatter, "invalid seed {seed}"),
      Self::InvalidCommand(command) => write!(formatter, "invalid command token {command}"),
    }
  }
}

impl Error for CliError {}

/// Errors produced while executing parsed commands through the core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunError {
  /// The fixed developer scenario could not be constructed.
  Scenario(String),
  /// A core command was rejected at its sequence index.
  Command {
    /// Zero-based index in the parsed command sequence.
    index: usize,
    /// The structured core rejection.
    source: CommandError,
  },
}

impl fmt::Display for RunError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Scenario(error) => write!(formatter, "scenario error: {error}"),
      Self::Command { index, source } => write!(formatter, "command {index} rejected: {source}"),
    }
  }
}

impl Error for RunError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Scenario(_) => None,
      Self::Command { source, .. } => Some(source),
    }
  }
}

/// Errors that can be returned by the complete CLI workflow.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppError {
  /// Argument parsing failed before a world was created.
  Cli(CliError),
  /// Core execution failed after parsing succeeded.
  Run(RunError),
}

impl fmt::Display for AppError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Cli(error) => error.fmt(formatter),
      Self::Run(error) => error.fmt(formatter),
    }
  }
}

impl Error for AppError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Cli(error) => Some(error),
      Self::Run(error) => Some(error),
    }
  }
}

/// Output evidence from one deterministic headless run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
  seed: u64,
  events: Vec<String>,
  digest: StateDigest,
  outcome: RunOutcome,
}

impl CliOutput {
  /// Returns the input seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns semantic event debug lines in command order.
  #[must_use]
  pub fn events(&self) -> &[String] {
    &self.events
  }

  /// Returns the final core state digest.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }

  /// Returns the canonical terminal outcome after the requested command sequence.
  #[must_use]
  pub const fn outcome(&self) -> RunOutcome {
    self.outcome
  }

  /// Renders stable line-oriented output for the process boundary.
  #[must_use]
  pub fn render(&self) -> String {
    let mut rendered = format!("seed={}\n", self.seed);
    for event in &self.events {
      rendered.push_str("event=");
      rendered.push_str(event);
      rendered.push('\n');
    }
    let outcome = match self.outcome {
      RunOutcome::InProgress => "in_progress",
      RunOutcome::Defeat => "defeat",
      RunOutcome::Victory => "victory",
    };
    let _ = writeln!(rendered, "outcome={outcome}");
    let _ = writeln!(rendered, "digest={}", self.digest.value());
    rendered
  }
}

/// Parses `--seed <u64> --commands <tokens>` arguments after the executable name.
///
/// # Errors
///
/// Returns [`CliError`] when a required argument is missing, an argument is repeated or
/// unsupported, or a seed/command token cannot be parsed.
pub fn parse_args<I>(args: I) -> Result<CliInput, CliError>
where
  I: IntoIterator<Item = String>,
{
  let mut seed = None;
  let mut commands = None;
  let mut arguments = args.into_iter();
  while let Some(argument) = arguments.next() {
    match argument.as_str() {
      "--seed" => {
        if seed.is_some() {
          return Err(CliError::DuplicateArgument("--seed"));
        }
        let value = arguments.next().ok_or(CliError::MissingValue("--seed"))?;
        let parsed = value
          .parse::<u64>()
          .map_err(|_| CliError::InvalidSeed(value))?;
        seed = Some(parsed);
      }
      "--commands" => {
        if commands.is_some() {
          return Err(CliError::DuplicateArgument("--commands"));
        }
        let value = arguments
          .next()
          .ok_or(CliError::MissingValue("--commands"))?;
        commands = Some(parse_commands(&value)?);
      }
      _ => return Err(CliError::UnknownArgument(argument)),
    }
  }
  Ok(CliInput {
    seed: seed.ok_or(CliError::MissingArgument("--seed"))?,
    commands: commands.ok_or(CliError::MissingArgument("--commands"))?,
  })
}

fn parse_commands(value: &str) -> Result<Vec<Command>, CliError> {
  if value.is_empty() {
    return Err(CliError::InvalidCommand(value.to_owned()));
  }
  value.split(',').map(parse_command).collect()
}

fn parse_command(token: &str) -> Result<Command, CliError> {
  let parts: Vec<_> = token.split(':').collect();
  let parse_actor = |value: Option<&&str>| {
    value
      .ok_or_else(|| CliError::InvalidCommand(token.to_owned()))?
      .parse::<u32>()
      .map(ActorId::new)
      .map_err(|_| CliError::InvalidCommand(token.to_owned()))
  };
  let parse_item = |value: Option<&&str>| {
    value
      .ok_or_else(|| CliError::InvalidCommand(token.to_owned()))?
      .parse::<u32>()
      .map(ItemId::new)
      .map_err(|_| CliError::InvalidCommand(token.to_owned()))
  };
  let parse_coordinate = |value: Option<&&str>| {
    value
      .ok_or_else(|| CliError::InvalidCommand(token.to_owned()))?
      .parse::<i32>()
      .map_err(|_| CliError::InvalidCommand(token.to_owned()))
  };
  match parts.as_slice() {
    ["move", actor, direction] => Ok(Command::Move {
      actor: parse_actor(Some(actor))?,
      direction: parse_direction(direction, token)?,
    }),
    ["wait", actor] => Ok(Command::Wait {
      actor: parse_actor(Some(actor))?,
    }),
    ["interact", actor, x, y] => Ok(Command::Interact {
      actor: parse_actor(Some(actor))?,
      position: Position::new(parse_coordinate(Some(x))?, parse_coordinate(Some(y))?),
    }),
    ["break", actor, x, y] => Ok(Command::Break {
      actor: parse_actor(Some(actor))?,
      position: Position::new(parse_coordinate(Some(x))?, parse_coordinate(Some(y))?),
    }),
    ["attack", actor, target] => Ok(Command::Attack {
      actor: parse_actor(Some(actor))?,
      target: parse_actor(Some(target))?,
    }),
    ["ranged_attack", actor, target] => Ok(Command::RangedAttack {
      actor: parse_actor(Some(actor))?,
      target: parse_actor(Some(target))?,
    }),
    ["chase", actor, target] => Ok(Command::Chase {
      actor: parse_actor(Some(actor))?,
      target: parse_actor(Some(target))?,
    }),
    ["reload", actor] => Ok(Command::Reload {
      actor: parse_actor(Some(actor))?,
    }),
    ["drop", actor, item] => Ok(Command::Drop {
      actor: parse_actor(Some(actor))?,
      item: parse_item(Some(item))?,
    }),
    _ => Err(CliError::InvalidCommand(token.to_owned())),
  }
}

fn parse_direction(value: &str, token: &str) -> Result<Direction, CliError> {
  match value {
    "north" => Ok(Direction::North),
    "south" => Ok(Direction::South),
    "west" => Ok(Direction::West),
    "east" => Ok(Direction::East),
    _ => Err(CliError::InvalidCommand(token.to_owned())),
  }
}

/// Executes parsed commands against the fixed developer scenario.
///
/// # Errors
///
/// Returns [`RunError::Scenario`] if the fixed scenario cannot be built, or
/// [`RunError::Command`] when the core rejects a command.
pub fn run(input: CliInput) -> Result<CliOutput, RunError> {
  let CliInput { seed, commands } = input;
  let mut world = fixed_scenario()?;
  let mut events = Vec::new();
  for (index, command) in commands.iter().copied().enumerate() {
    let outcome = world
      .execute(command)
      .map_err(|source| RunError::Command { index, source })?;
    events.extend(outcome.events().iter().map(|event| format!("{event:?}")));
  }
  Ok(CliOutput {
    seed,
    events,
    digest: world.digest(),
    outcome: world.outcome(),
  })
}

/// Parses and executes a complete CLI argument sequence.
///
/// # Errors
///
/// Returns [`AppError::Cli`] for malformed arguments and [`AppError::Run`] for scenario or
/// command execution failures.
pub fn execute<I>(args: I) -> Result<CliOutput, AppError>
where
  I: IntoIterator<Item = String>,
{
  let input = parse_args(args).map_err(AppError::Cli)?;
  run(input).map_err(AppError::Run)
}

fn fixed_scenario() -> Result<WorldState, RunError> {
  let map =
    GridMap::filled(3, 1, Tile::Floor).map_err(|error| RunError::Scenario(error.to_string()))?;
  let mut world = WorldState::new(
    map,
    vec![
      Actor::with_ranged_ammo(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
        2,
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .map_err(|error| RunError::Scenario(error.to_string()))?;
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(101), ItemDefinitionId::new(1)),
    )
    .map_err(|error| RunError::Scenario(error.to_string()))?;
  Ok(world)
}

#[cfg(test)]
mod tests {
  use dreadstep_core::{ActorId, Command, Direction};

  use super::*;

  #[test]
  fn parses_seed_and_ordered_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "attack:1:2,wait:2".to_owned(),
    ])
    .expect("valid CLI input should parse");

    assert_eq!(input.seed(), 7);
    assert_eq!(
      input.commands(),
      &[
        Command::Attack {
          actor: ActorId::new(1),
          target: ActorId::new(2),
        },
        Command::Wait {
          actor: ActorId::new(2),
        },
      ]
    );
  }

  #[test]
  fn parses_ranged_attack_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "ranged_attack:1:2".to_owned(),
    ])
    .expect("ranged command should parse");

    assert_eq!(
      input.commands(),
      &[Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }]
    );
  }

  #[test]
  fn parses_reload_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "reload:1".to_owned(),
    ])
    .expect("reload command should parse");

    assert_eq!(
      input.commands(),
      &[Command::Reload {
        actor: ActorId::new(1),
      }]
    );
  }

  #[test]
  fn parses_drop_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "drop:1:101".to_owned(),
    ])
    .expect("drop command should parse");

    assert_eq!(
      input.commands(),
      &[Command::Drop {
        actor: ActorId::new(1),
        item: ItemId::new(101),
      }]
    );
  }

  #[test]
  fn parses_interact_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "interact:1:2:-3".to_owned(),
    ])
    .expect("interact command should parse");

    assert_eq!(
      input.commands(),
      &[Command::Interact {
        actor: ActorId::new(1),
        position: Position::new(2, -3),
      }]
    );
  }

  #[test]
  fn parses_break_command_tokens() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "break:1:2:-3".to_owned(),
    ])
    .expect("break command should parse");

    assert_eq!(
      input.commands(),
      &[Command::Break {
        actor: ActorId::new(1),
        position: Position::new(2, -3),
      }]
    );
  }

  #[test]
  fn rejects_duplicate_and_unknown_arguments() {
    assert_eq!(
      parse_args([
        "--seed".to_owned(),
        "7".to_owned(),
        "--seed".to_owned(),
        "8".to_owned(),
        "--commands".to_owned(),
        "wait:1".to_owned(),
      ]),
      Err(CliError::DuplicateArgument("--seed"))
    );
    assert_eq!(
      parse_args(["--seed".to_owned(), "7".to_owned(), "--wat".to_owned()]),
      Err(CliError::UnknownArgument("--wat".to_owned()))
    );
  }

  #[test]
  fn rejects_missing_values_and_malformed_commands() {
    assert_eq!(
      parse_args(["--seed".to_owned()]),
      Err(CliError::MissingValue("--seed"))
    );
    assert_eq!(
      parse_args(["--seed".to_owned(), "7".to_owned()]),
      Err(CliError::MissingArgument("--commands"))
    );
    assert_eq!(
      parse_args([
        "--seed".to_owned(),
        "not-a-number".to_owned(),
        "--commands".to_owned(),
        "wait:1".to_owned(),
      ]),
      Err(CliError::InvalidSeed("not-a-number".to_owned()))
    );
    assert_eq!(
      parse_args([
        "--seed".to_owned(),
        "7".to_owned(),
        "--commands".to_owned(),
        "move:1:sideways".to_owned(),
      ]),
      Err(CliError::InvalidCommand("move:1:sideways".to_owned()))
    );
  }

  #[test]
  fn runs_the_fixed_scenario_through_core_and_renders_evidence() {
    let input = parse_args([
      "--seed".to_owned(),
      "7".to_owned(),
      "--commands".to_owned(),
      "attack:1:2,wait:2".to_owned(),
    ])
    .unwrap();
    let output = run(input).expect("valid commands should run");
    let rendered = output.render();

    assert_eq!(output.seed(), 7);
    assert_eq!(output.events().len(), 2);
    assert_eq!(
      rendered,
      "seed=7\n\
event=Attacked { attacker: ActorId(1), target: ActorId(2), damage: Damage(1), remaining_hit_points: HitPoints(1) }\n\
event=Waited { actor: ActorId(2), at: ActionTime(0) }\n\
outcome=in_progress\n\
digest=9203069779232099541\n"
    );
  }

  #[test]
  fn parses_each_supported_command_shape() {
    let input = parse_args([
      "--seed".to_owned(),
      "1".to_owned(),
      "--commands".to_owned(),
      "move:1:east,wait:2,attack:1:2,chase:2:1".to_owned(),
    ])
    .unwrap();

    assert_eq!(input.commands().len(), 4);
    assert_eq!(
      input.commands()[0],
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      }
    );
  }
}
