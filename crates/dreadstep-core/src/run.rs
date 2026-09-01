//! Run-level progression state and completed-floor metadata.
//!
//! [`WorldState`] remains the authoritative simulation for one floor. [`RunState`] owns only the
//! lifecycle around that world: the stable run seed, current depth, and compact metadata for each
//! floor that has been entered. Content generators stay outside this crate; callers pass a
//! validated next world into [`RunState::advance`].

use std::fmt;

use crate::{
  ActionResult, Command, CommandError, RunOutcome, StateDigest, replay::StableHasher,
  world::WorldState,
};

/// Compact metadata for one entered floor.
///
/// The record stores no duplicate world or generator state. Its digest and outcome are refreshed
/// after each accepted semantic command while the floor is current, then remain stable after the
/// run advances to a later floor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FloorRecord {
  depth: u32,
  state_digest: StateDigest,
  outcome: RunOutcome,
}

impl FloorRecord {
  /// Creates metadata for an entered floor.
  #[must_use]
  pub const fn new(depth: u32, state_digest: StateDigest, outcome: RunOutcome) -> Self {
    Self {
      depth,
      state_digest,
      outcome,
    }
  }

  /// Returns the floor depth represented by this record.
  #[must_use]
  pub const fn depth(self) -> u32 {
    self.depth
  }

  /// Returns the stable world-state digest captured by this record.
  #[must_use]
  pub const fn state_digest(self) -> StateDigest {
    self.state_digest
  }

  /// Returns the outcome captured by this record.
  #[must_use]
  pub const fn outcome(self) -> RunOutcome {
    self.outcome
  }
}

/// Evidence returned after a successful one-floor transition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FloorTransition {
  from_depth: u32,
  to_depth: u32,
  from_digest: StateDigest,
  to_digest: StateDigest,
}

impl FloorTransition {
  /// Returns the depth that was completed.
  #[must_use]
  pub const fn from_depth(self) -> u32 {
    self.from_depth
  }

  /// Returns the newly entered depth.
  #[must_use]
  pub const fn to_depth(self) -> u32 {
    self.to_depth
  }

  /// Returns the completed floor's final world-state digest.
  #[must_use]
  pub const fn from_digest(self) -> StateDigest {
    self.from_digest
  }

  /// Returns the newly entered floor's initial world-state digest.
  #[must_use]
  pub const fn to_digest(self) -> StateDigest {
    self.to_digest
  }
}

/// Typed rejection reasons for a floor transition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FloorAdvanceError {
  /// The current floor has not reached the canonical victory outcome.
  NotVictorious {
    /// The current floor depth.
    depth: u32,
    /// The current floor's terminal projection.
    outcome: RunOutcome,
  },
  /// The current depth cannot advance without overflowing its representation.
  DepthOverflow {
    /// The depth that could not be incremented.
    depth: u32,
  },
  /// The requested next depth is not exactly one greater than the current depth.
  NonContiguousDepth {
    /// The run's current depth.
    current: u32,
    /// The requested destination depth.
    requested: u32,
  },
}

impl fmt::Display for FloorAdvanceError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NotVictorious { depth, outcome } => {
        write!(formatter, "floor {depth} cannot advance from {outcome:?}")
      }
      Self::DepthOverflow { depth } => write!(formatter, "floor depth {depth} cannot advance"),
      Self::NonContiguousDepth { current, requested } => write!(
        formatter,
        "floor transition requested depth {requested} after {current}"
      ),
    }
  }
}

impl std::error::Error for FloorAdvanceError {}

/// Run lifecycle state around the authoritative current-floor simulation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunState {
  seed: u64,
  depth: u32,
  world: WorldState,
  history: Vec<FloorRecord>,
}

impl RunState {
  /// Starts a run at an already validated world and records its initial floor metadata.
  #[must_use]
  pub fn new(seed: u64, depth: u32, world: WorldState) -> Self {
    let initial_record = FloorRecord::new(depth, world.digest(), world.outcome());
    Self {
      seed,
      depth,
      world,
      history: vec![initial_record],
    }
  }

  /// Returns the stable seed associated with this run.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns the current floor depth.
  #[must_use]
  pub const fn depth(&self) -> u32 {
    self.depth
  }

  /// Returns the authoritative current-floor world.
  #[must_use]
  pub const fn world(&self) -> &WorldState {
    &self.world
  }

  /// Returns compact metadata for every floor entered by this run, in depth order.
  #[must_use = "inspect the run's floor history"]
  pub fn history(&self) -> &[FloorRecord] {
    &self.history
  }

  /// Returns a stable digest of run identity, floor history, and current world state.
  ///
  /// This is regression evidence, not a serialized save format or cryptographic integrity check.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    let mut hasher = StableHasher::new();
    hasher.write_bytes(b"DREADSTEP-RUN-V1");
    hasher.write_u64(self.seed);
    hasher.write_u32(self.depth);
    hasher.write_u64(u64::try_from(self.history.len()).unwrap_or(u64::MAX));
    for record in &self.history {
      hasher.write_u32(record.depth);
      hasher.write_u64(record.state_digest.value());
      hasher.write_u8(outcome_code(record.outcome));
    }
    hasher.write_u64(self.world.digest().value());
    hasher.finish()
  }

  /// Applies one semantic command to the current floor and refreshes its history record.
  ///
  /// Tester-only mutations remain methods on [`WorldState`]; they are intentionally not part of
  /// player replay or this run-level command path.
  ///
  /// # Errors
  ///
  /// Returns the underlying [`CommandError`] and leaves the run unchanged when the command is
  /// rejected.
  pub fn execute(&mut self, command: Command) -> Result<ActionResult, CommandError> {
    let result = self.world.execute(command)?;
    self.refresh_current_floor();
    Ok(result)
  }

  /// Replaces the current floor with the caller-generated next world after canonical victory.
  ///
  /// The requested depth must be exactly one greater than the current depth. All validation is
  /// completed before any field is changed, so every rejection is atomic. The supplied
  /// [`WorldState`] is already validated by its constructor; this method deliberately does not
  /// depend on `dreadstep-content` or generate a floor itself.
  ///
  /// # Errors
  ///
  /// Returns [`FloorAdvanceError::NotVictorious`] when the current floor is not won,
  /// [`FloorAdvanceError::DepthOverflow`] when the current depth cannot increment, or
  /// [`FloorAdvanceError::NonContiguousDepth`] for a depth other than the next contiguous value.
  pub fn advance(
    &mut self,
    next_depth: u32,
    next_world: WorldState,
  ) -> Result<FloorTransition, FloorAdvanceError> {
    let outcome = self.world.outcome();
    if outcome != RunOutcome::Victory {
      return Err(FloorAdvanceError::NotVictorious {
        depth: self.depth,
        outcome,
      });
    }
    let expected_depth = self
      .depth
      .checked_add(1)
      .ok_or(FloorAdvanceError::DepthOverflow { depth: self.depth })?;
    if next_depth != expected_depth {
      return Err(FloorAdvanceError::NonContiguousDepth {
        current: self.depth,
        requested: next_depth,
      });
    }

    self.refresh_current_floor();
    let from_depth = self.depth;
    let from_digest = self.world.digest();
    let to_digest = next_world.digest();
    let next_record = FloorRecord::new(next_depth, to_digest, next_world.outcome());

    self.depth = next_depth;
    self.world = next_world;
    self.history.push(next_record);

    Ok(FloorTransition {
      from_depth,
      to_depth: next_depth,
      from_digest,
      to_digest,
    })
  }

  fn refresh_current_floor(&mut self) {
    let record = self
      .history
      .last_mut()
      .expect("a RunState always contains its initial floor record");
    record.state_digest = self.world.digest();
    record.outcome = self.world.outcome();
  }
}

const fn outcome_code(outcome: RunOutcome) -> u8 {
  match outcome {
    RunOutcome::InProgress => 1,
    RunOutcome::Defeat => 2,
    RunOutcome::Victory => 3,
  }
}
