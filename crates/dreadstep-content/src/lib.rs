//! Validated content definitions for Dreadstep.
//!
//! Authored data will enter through this boundary and become typed domain values. Content
//! may describe rules supported by `dreadstep-core`, but it must not introduce hidden
//! simulation behavior.

#![forbid(unsafe_code)]

use std::{collections::BTreeSet, error::Error, fmt};

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, ItemDefinitionId, MapError, Position, Tile,
  WorldError, WorldState,
};

/// Errors raised while validating or building authored content and core-world inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentError {
  /// The authored rectangular map is invalid.
  Map(MapError),
  /// The authored actor set violates a core world invariant.
  World(WorldError),
  /// The authored item catalog repeats one opaque definition identity.
  DuplicateItemDefinitionId(ItemDefinitionId),
}

impl fmt::Display for ContentError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Map(error) => write!(formatter, "content map error: {error}"),
      Self::World(error) => write!(formatter, "content world error: {error}"),
      Self::DuplicateItemDefinitionId(definition) => write!(
        formatter,
        "content item definition id {} is duplicated",
        definition.value()
      ),
    }
  }
}

impl Error for ContentError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Map(error) => Some(error),
      Self::World(error) => Some(error),
      Self::DuplicateItemDefinitionId(_) => None,
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

/// Typed authored input for one ordered catalog of opaque item-definition identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCatalogDefinition {
  definitions: Vec<ItemDefinitionId>,
}

impl ItemCatalogDefinition {
  /// Creates authored item-definition references; validation runs in [`Self::build`].
  #[must_use]
  pub const fn new(definitions: Vec<ItemDefinitionId>) -> Self {
    Self { definitions }
  }

  /// Converts authored references into an immutable, validated content catalog.
  ///
  /// Declaration order is preserved. Core remains the owner of item instances and ownership;
  /// this catalog only answers which opaque definition identities the content names.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError::DuplicateItemDefinitionId`] when one identity occurs more than once.
  pub fn build(&self) -> Result<ItemCatalog, ContentError> {
    let mut seen = BTreeSet::new();
    for definition in &self.definitions {
      if !seen.insert(*definition) {
        return Err(ContentError::DuplicateItemDefinitionId(*definition));
      }
    }
    Ok(ItemCatalog {
      definitions: self.definitions.clone(),
    })
  }
}

/// An immutable deterministic catalog of content-known opaque item-definition identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemCatalog {
  definitions: Vec<ItemDefinitionId>,
}

impl ItemCatalog {
  /// Returns definitions in their authored declaration order.
  #[must_use]
  pub fn definitions(&self) -> &[ItemDefinitionId] {
    &self.definitions
  }

  /// Returns whether content declares the supplied opaque definition identity.
  #[must_use]
  pub fn contains(&self, definition: ItemDefinitionId) -> bool {
    self.definitions.contains(&definition)
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

/// Returns the deterministic authored starter item-definition references.
#[must_use]
pub fn starter_item_catalog_definition() -> ItemCatalogDefinition {
  ItemCatalogDefinition::new(vec![
    ItemDefinitionId::new(1),
    ItemDefinitionId::new(2),
    ItemDefinitionId::new(3),
  ])
}

/// Builds the validated deterministic starter item-definition catalog.
///
/// # Errors
///
/// Returns [`ContentError::DuplicateItemDefinitionId`] if the authored starter references are
/// accidentally repeated.
pub fn starter_item_catalog() -> Result<ItemCatalog, ContentError> {
  starter_item_catalog_definition().build()
}
