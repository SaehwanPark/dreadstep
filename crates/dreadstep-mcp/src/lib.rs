//! Model Context Protocol adapter boundary for Dreadstep.
//!
//! This crate will translate explicit player and tester operations into project-owned
//! semantic commands. It must not become a generic shell, filesystem escape hatch, or
//! hidden source of game truth. Milestone 0 intentionally adds no MCP runtime dependency.

#![forbid(unsafe_code)]

use std::{error::Error, fmt};

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Position, ReplayTrace, Tile, WorldState,
};
use dreadstep_protocol::{
  ActorId as ProtocolActorId, ActorSnapshot, CommandError, CommandRequest, Event, ReplayEvidence,
  StateDigest, WorldSnapshot,
};

/// Errors returned by the in-memory MCP player session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionError {
  /// The fixed developer scenario could not be constructed.
  Scenario(String),
  /// Core rejected the requested command.
  CommandRejected(CommandError),
}

impl fmt::Display for SessionError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Scenario(error) => write!(formatter, "scenario error: {error}"),
      Self::CommandRejected(error) => write!(formatter, "command rejected: {error}"),
    }
  }
}

impl Error for SessionError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Scenario(_) => None,
      Self::CommandRejected(error) => Some(error),
    }
  }
}

/// Evidence returned after one accepted player action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOutput {
  seed: u64,
  events: Vec<Event>,
  snapshot: WorldSnapshot,
}

impl SessionOutput {
  /// Returns the session seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns protocol event evidence in execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the post-action world snapshot.
  #[must_use]
  pub const fn snapshot(&self) -> &WorldSnapshot {
    &self.snapshot
  }
}

/// A deterministic in-memory player session around one fixed developer scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
  seed: u64,
  world: WorldState,
  trace: ReplayTrace,
}

impl Session {
  /// Starts a session with an explicit seed and fixed developer scenario.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::Scenario`] if the fixed map or actor setup is invalid.
  pub fn start_run(seed: u64) -> Result<Self, SessionError> {
    Ok(Self {
      seed,
      world: fixed_scenario()?,
      trace: ReplayTrace::new(seed),
    })
  }

  /// Returns the explicit session seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns a read-only protocol snapshot of the current world.
  #[must_use]
  pub fn observe(&self) -> WorldSnapshot {
    WorldSnapshot::from_world(&self.world)
  }

  /// Returns one protocol actor snapshot, or no value for an unknown identity.
  #[must_use]
  pub fn inspect(&self, actor: ProtocolActorId) -> Option<ActorSnapshot> {
    self
      .observe()
      .actors()
      .iter()
      .find(|snapshot| snapshot.id() == actor)
      .cloned()
  }

  /// Returns protocol requests currently accepted by the core scheduler and rules.
  #[must_use]
  pub fn legal_actions(&self) -> Vec<CommandRequest> {
    self
      .world
      .legal_commands()
      .into_iter()
      .map(CommandRequest::from)
      .collect()
  }

  /// Returns accepted requests in execution order.
  #[must_use]
  pub fn history(&self) -> Vec<CommandRequest> {
    self
      .trace
      .commands()
      .iter()
      .copied()
      .map(CommandRequest::from)
      .collect()
  }

  /// Returns accepted requests in execution order using the named player operation.
  #[must_use]
  pub fn get_history(&self) -> Vec<CommandRequest> {
    self.history()
  }

  /// Returns the deterministic core replay digest for accepted actions.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    StateDigest::new(self.trace.digest().value())
  }

  /// Returns the explicit seed, accepted requests, and deterministic trace digest.
  #[must_use]
  pub fn get_replay(&self) -> ReplayEvidence {
    ReplayEvidence::new(self.seed, self.history(), self.replay_digest())
  }

  /// Applies one protocol request through the core and returns protocol evidence.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::CommandRejected`] when core rejects the request. Rejected commands
  /// produce no output and leave the session unchanged.
  pub fn act(&mut self, request: CommandRequest) -> Result<SessionOutput, SessionError> {
    let command = Command::from(request);
    let result = self
      .world
      .execute(command)
      .map_err(|error| SessionError::CommandRejected(error.into()))?;
    self.trace.record(command);
    Ok(SessionOutput {
      seed: self.seed,
      events: result.events().iter().copied().map(Event::from).collect(),
      snapshot: self.observe(),
    })
  }
}

fn fixed_scenario() -> Result<WorldState, SessionError> {
  let map = GridMap::filled(3, 1, Tile::Floor)
    .map_err(|error| SessionError::Scenario(error.to_string()))?;
  WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .map_err(|error| SessionError::Scenario(error.to_string()))
}
