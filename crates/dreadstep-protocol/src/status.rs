//! Versioned actor status projections.

use dreadstep_core::StatusKind as CoreStatusKind;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A closed protocol status kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
  /// Adds one action-time tick to the next accepted action.
  Chilled,
}

impl From<CoreStatusKind> for StatusKind {
  fn from(status: CoreStatusKind) -> Self {
    match status {
      CoreStatusKind::Chilled => Self::Chilled,
    }
  }
}

/// A read-only status projection with its remaining affected actions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
pub struct StatusSnapshot {
  kind: StatusKind,
  remaining_actions: u8,
}

impl StatusSnapshot {
  pub(crate) fn from_core(status: dreadstep_core::Status) -> Self {
    Self {
      kind: status.kind().into(),
      remaining_actions: status.remaining_actions(),
    }
  }

  /// Returns the status kind.
  #[must_use]
  pub const fn kind(self) -> StatusKind {
    self.kind
  }

  /// Returns affected actions remaining.
  #[must_use]
  pub const fn remaining_actions(self) -> u8 {
    self.remaining_actions
  }
}
