//! Integer hit points used by actors and item-effect evidence.
//!
//! Zero means a dead actor. Keeping this newtype outside actor and item modules avoids a
//! circular dependency between inventory records and combat resources.

/// The current integer hit points of an actor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HitPoints(u16);

impl HitPoints {
  /// Creates hit points from a numeric value. Zero represents a dead actor.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self(value)
  }

  /// Returns the numeric hit-point value.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }

  /// Returns whether these hit points represent a living actor.
  #[must_use]
  pub const fn is_alive(self) -> bool {
    self.0 > 0
  }

  pub(crate) fn reduced_by(self, damage: crate::Damage) -> Self {
    Self(self.0.saturating_sub(damage.0))
  }
}
