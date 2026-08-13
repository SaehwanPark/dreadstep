//! Stable opaque identities for actors and item instances.
//!
//! Identities are assigned by callers and validated by world construction. They never encode
//! gameplay rules on their own.

/// A stable identity for an actor in a world.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActorId(u32);

impl ActorId {
  /// Creates an actor identity from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// A globally unique identity for one opaque item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
  /// Creates an item identity from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// An opaque content reference for an item instance.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ItemDefinitionId(u32);

impl ItemDefinitionId {
  /// Creates an item-definition reference from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric value of this definition reference.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}
