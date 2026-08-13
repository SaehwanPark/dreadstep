//! Protocol identities and scalar wire values.

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

/// A stable actor identity in the protocol projection.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct ActorId(u32);

impl ActorId {
  /// Creates a protocol actor identity from its numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the numeric identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// An actor kind represented by the protocol projection.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
  /// The player-controlled actor.
  Player,
  /// An actor controlled by simulation or an adapter.
  Enemy,
}

/// A protocol position value.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, JsonSchema, Serialize)]
pub struct Position {
  x: i32,
  y: i32,
}

impl Position {
  /// Creates a protocol position from its coordinates.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Returns the horizontal coordinate.
  #[must_use]
  pub const fn x(self) -> i32 {
    self.x
  }

  /// Returns the vertical coordinate.
  #[must_use]
  pub const fn y(self) -> i32 {
    self.y
  }
}

/// Protocol hit-point evidence for an actor.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize,
)]
pub struct HitPoints(u16);

impl HitPoints {
  /// Creates protocol hit-point evidence.
  #[must_use]
  pub const fn new(value: u16) -> Self {
    Self(value)
  }

  /// Returns the numeric hit-point value.
  #[must_use]
  pub const fn value(self) -> u16 {
    self.0
  }
}

/// A protocol actor's non-zero Manhattan melee reach.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct MeleeReach(u8);

impl JsonSchema for MeleeReach {
  fn schema_name() -> std::borrow::Cow<'static, str> {
    "MeleeReach".into()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    let mut schema = u8::json_schema(generator);
    schema.insert("minimum".to_owned(), 1u8.into());
    schema
  }
}

impl MeleeReach {
  /// The default adjacent melee reach.
  pub const DEFAULT: Self = Self(1);

  /// Creates protocol reach evidence, rejecting zero.
  #[must_use]
  pub const fn new(value: u8) -> Option<Self> {
    if value == 0 { None } else { Some(Self(value)) }
  }

  /// Returns the numeric Manhattan reach.
  #[must_use]
  pub const fn value(self) -> u8 {
    self.0
  }
}

impl Default for MeleeReach {
  fn default() -> Self {
    Self::DEFAULT
  }
}

/// A stable item instance identity in the protocol projection.
#[derive(
  Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize,
)]
pub struct ItemId(u32);

impl ItemId {
  /// Creates an item identity from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric identity.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}

/// An opaque item-definition reference in the protocol projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, JsonSchema, Serialize)]
pub struct ItemDefinitionId(u32);

impl ItemDefinitionId {
  /// Creates a definition reference from its stable numeric value.
  #[must_use]
  pub const fn new(value: u32) -> Self {
    Self(value)
  }

  /// Returns the stable numeric definition reference.
  #[must_use]
  pub const fn value(self) -> u32 {
    self.0
  }
}
