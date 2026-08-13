//! Closed actor status effects and their deterministic action lifetimes.

/// The status effects currently supported by the simulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StatusKind {
  /// Adds one scheduler tick to each affected accepted action.
  Chilled,
}

/// One active status and its remaining affected actions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Status {
  kind: StatusKind,
  remaining_actions: u8,
}

impl Status {
  /// The fixed refresh duration for a chilled actor.
  pub const CHILLED_ACTIONS: u8 = 2;

  /// Creates a refreshed chilled status.
  #[must_use]
  pub const fn chilled() -> Self {
    Self {
      kind: StatusKind::Chilled,
      remaining_actions: Self::CHILLED_ACTIONS,
    }
  }

  /// Returns the status kind.
  #[must_use]
  pub const fn kind(self) -> StatusKind {
    self.kind
  }

  /// Returns the number of future accepted actions affected by this status.
  #[must_use]
  pub const fn remaining_actions(self) -> u8 {
    self.remaining_actions
  }

  /// Consumes one affected action, returning the refreshed remaining status.
  #[must_use]
  pub const fn after_action(self) -> Option<Self> {
    if self.remaining_actions <= 1 {
      None
    } else {
      Some(Self {
        kind: self.kind,
        remaining_actions: self.remaining_actions - 1,
      })
    }
  }
}
