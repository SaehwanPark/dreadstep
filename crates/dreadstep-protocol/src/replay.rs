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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize)]
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
