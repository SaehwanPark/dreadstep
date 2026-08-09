//! Validated content definitions for Dreadstep.
//!
//! Authored data will enter through this boundary and become typed domain values. Content
//! may describe rules supported by `dreadstep-core`, but it must not introduce hidden
//! simulation behavior.

#![forbid(unsafe_code)]

use std::{error::Error, fmt};

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, MapError, Position, Tile, WorldError, WorldState,
};

/// Errors raised while converting authored content into a validated core world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentError {
  /// The authored rectangular map is invalid.
  Map(MapError),
  /// The authored actor set violates a core world invariant.
  World(WorldError),
}

impl fmt::Display for ContentError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Map(error) => write!(formatter, "content map error: {error}"),
      Self::World(error) => write!(formatter, "content world error: {error}"),
    }
  }
}

impl Error for ContentError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Map(error) => Some(error),
      Self::World(error) => Some(error),
    }
  }
}

impl From<MapError> for ContentError {
  fn from(error: MapError) -> Self {
    Self::Map(error)
  }
}

impl From<WorldError> for ContentError {
  fn from(error: WorldError) -> Self {
    Self::World(error)
  }
}

/// Typed authored input for one rectangular starter floor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StarterFloorDefinition {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<Actor>,
}

impl StarterFloorDefinition {
  /// Creates authored map and actor input; core validates it when [`Self::build`] is called.
  #[must_use]
  pub const fn new(width: u32, height: u32, tiles: Vec<Tile>, actors: Vec<Actor>) -> Self {
    Self {
      width,
      height,
      tiles,
      actors,
    }
  }

  /// Converts this authored input into the validated core world.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError::Map`] or [`ContentError::World`] when authored dimensions, tiles,
  /// or actor records violate core validation rules.
  pub fn build(&self) -> Result<WorldState, ContentError> {
    let map = GridMap::from_tiles(self.width, self.height, self.tiles.clone())?;
    Ok(WorldState::new(map, self.actors.clone())?)
  }
}

/// Returns the deterministic authored starter-floor definition.
#[must_use]
pub fn starter_floor_definition() -> StarterFloorDefinition {
  StarterFloorDefinition::new(
    7,
    5,
    vec![
      // y = 0
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      // y = 1
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      // y = 2
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      // y = 3
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      // y = 4
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(1, 1),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(5, 1),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(3),
        ActorKind::Enemy,
        Position::new(1, 3),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(4),
        ActorKind::Enemy,
        Position::new(5, 3),
        HitPoints::new(3),
      ),
    ],
  )
}

/// Builds the validated deterministic authored starter floor.
///
/// # Errors
///
/// Returns [`ContentError`] when the authored definition fails core map or world validation.
pub fn starter_floor() -> Result<WorldState, ContentError> {
  starter_floor_definition().build()
}
