//! Protocol replay evidence wrapping core digest and accepted command order.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CommandRequest;

/// A protocol action timestamp.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize,
)]
pub struct ActionTime(u64);

impl ActionTime {
  /// Creates protocol action-time evidence.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric action time.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// A protocol view of the core's non-cryptographic state digest.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize,
)]
pub struct StateDigest(u64);

impl StateDigest {
  /// Creates protocol digest evidence.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric digest value.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// Schema version for create-new diagnostic replay exports.
pub const REPLAY_EXPORT_SCHEMA_VERSION: u16 = 2;

/// The authored content entry point needed to reconstruct a replay.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayScenario {
  /// The authored item-free starter floor.
  Starter,
  /// The authored item-bearing showcase floor.
  ItemShowcase,
  /// The adapter's exhaustive smoke fixture.
  ///
  /// Smoke runs intentionally mutate core-owned fixtures between commands so they can cover
  /// every command and event kind in one short process. Those setup mutations are diagnostic
  /// evidence, not a reconstructible gameplay start state, so this scenario cannot be verified
  /// by the playback command.
  SmokeFixture,
  /// A seeded procedural floor at an explicit one-based depth.
  Procedural {
    /// The authored procedural depth.
    depth: u32,
  },
}

impl ReplayScenario {
  /// Returns the procedural depth when this scenario is procedural.
  #[must_use]
  pub const fn depth(self) -> Option<u32> {
    match self {
      Self::Starter | Self::ItemShowcase | Self::SmokeFixture => None,
      Self::Procedural { depth } => Some(depth),
    }
  }
}

/// A versioned create-new diagnostic replay export.
///
/// Unlike in-memory [`ReplayEvidence`], this shape contains enough start metadata and final
/// evidence for a verifier to reconstruct the authored world and check every accepted command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, JsonSchema, Serialize)]
pub struct ReplayExport {
  schema_version: u16,
  seed: u64,
  scenario: ReplayScenario,
  commands: Vec<CommandRequest>,
  replay_digest: StateDigest,
  state_digest: StateDigest,
  outcome: crate::RunOutcome,
}

impl ReplayExport {
  /// Creates a diagnostic export with the current schema version.
  #[must_use]
  pub const fn new(
    seed: u64,
    scenario: ReplayScenario,
    commands: Vec<CommandRequest>,
    replay_digest: StateDigest,
    state_digest: StateDigest,
    outcome: crate::RunOutcome,
  ) -> Self {
    Self {
      schema_version: REPLAY_EXPORT_SCHEMA_VERSION,
      seed,
      scenario,
      commands,
      replay_digest,
      state_digest,
      outcome,
    }
  }

  /// Returns the diagnostic export schema version.
  #[must_use]
  pub const fn schema_version(&self) -> u16 {
    self.schema_version
  }

  /// Returns the deterministic run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns the content entry point used to construct the run.
  #[must_use]
  pub const fn scenario(&self) -> ReplayScenario {
    self.scenario
  }

  /// Returns accepted protocol commands in execution order.
  #[must_use]
  pub fn commands(&self) -> &[CommandRequest] {
    &self.commands
  }

  /// Returns the deterministic accepted-command trace digest.
  #[must_use]
  pub const fn replay_digest(&self) -> StateDigest {
    self.replay_digest
  }

  /// Returns the final core world digest.
  #[must_use]
  pub const fn state_digest(&self) -> StateDigest {
    self.state_digest
  }

  /// Returns the final core run outcome.
  #[must_use]
  pub const fn outcome(&self) -> crate::RunOutcome {
    self.outcome
  }
}

/// In-memory replay evidence exposed to an agent without claiming a serialized replay format.
#[derive(Clone, Debug, Eq, PartialEq, JsonSchema, Serialize)]
pub struct ReplayEvidence {
  seed: u64,
  commands: Vec<CommandRequest>,
  digest: StateDigest,
}

impl ReplayEvidence {
  /// Creates replay evidence from an explicit seed, accepted requests, and trace digest.
  #[must_use]
  pub const fn new(seed: u64, commands: Vec<CommandRequest>, digest: StateDigest) -> Self {
    Self {
      seed,
      commands,
      digest,
    }
  }

  /// Returns the explicit run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns accepted protocol requests in execution order.
  #[must_use]
  pub fn commands(&self) -> &[CommandRequest] {
    &self.commands
  }

  /// Returns the deterministic core trace digest.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }
}
