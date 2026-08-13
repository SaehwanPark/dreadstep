//! Tester scenario values converted into core maps and actors.

use crate::{ActorId, ActorKind, EnemyBehavior, HitPoints, MeleeReach, Position};

/// Protocol terrain for tester scenario construction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tile {
  /// A walkable cell.
  Floor,
  /// A walkable cell that blocks ranged line of sight.
  Cover,
  /// A blocking cell.
  Wall,
  /// A closed door that blocks movement until opened.
  Door,
  /// A blocking terrain cell that an adjacent actor may break into floor.
  Breakable,
  /// A walkable floor trap that triggers once when entered.
  Trap,
  /// A walkable one-shot trap that applies chilled when entered.
  ChillTrap,
}

/// One typed actor record in a tester scenario's initial world.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioActor {
  id: ActorId,
  kind: ActorKind,
  position: Position,
  hit_points: HitPoints,
  melee_reach: MeleeReach,
  behavior: EnemyBehavior,
}

impl ScenarioActor {
  /// Creates one initial actor record for a tester scenario.
  #[must_use]
  pub const fn new(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
  ) -> Self {
    Self::with_melee_reach(id, kind, position, hit_points, MeleeReach::DEFAULT)
  }

  /// Creates one initial actor record with an explicit melee reach.
  #[must_use]
  pub const fn with_melee_reach(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    melee_reach: MeleeReach,
  ) -> Self {
    Self::with_melee_reach_and_behavior(
      id,
      kind,
      position,
      hit_points,
      melee_reach,
      EnemyBehavior::Pursuer,
    )
  }

  /// Creates one initial actor record with explicit melee reach and enemy behavior.
  #[must_use]
  pub const fn with_melee_reach_and_behavior(
    id: ActorId,
    kind: ActorKind,
    position: Position,
    hit_points: HitPoints,
    melee_reach: MeleeReach,
    behavior: EnemyBehavior,
  ) -> Self {
    Self {
      id,
      kind,
      position,
      hit_points,
      melee_reach,
      behavior,
    }
  }

  /// Returns the actor identity.
  #[must_use]
  pub const fn id(self) -> ActorId {
    self.id
  }

  /// Returns the actor kind.
  #[must_use]
  pub const fn kind(self) -> ActorKind {
    self.kind
  }

  /// Returns the initial position.
  #[must_use]
  pub const fn position(self) -> Position {
    self.position
  }

  /// Returns the initial hit points.
  #[must_use]
  pub const fn hit_points(self) -> HitPoints {
    self.hit_points
  }

  /// Returns the initial actor's melee reach.
  #[must_use]
  pub const fn melee_reach(self) -> MeleeReach {
    self.melee_reach
  }

  /// Returns the authored enemy behavior policy.
  #[must_use]
  pub const fn behavior(self) -> EnemyBehavior {
    self.behavior
  }
}

/// A typed rectangular map and initial actor set for an in-memory tester scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scenario {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<ScenarioActor>,
}

impl Scenario {
  /// Creates a scenario description. Core validates dimensions, tile count, and actors when it
  /// is installed in a session.
  #[must_use]
  pub const fn new(width: u32, height: u32, tiles: Vec<Tile>, actors: Vec<ScenarioActor>) -> Self {
    Self {
      width,
      height,
      tiles,
      actors,
    }
  }

  /// Returns the map width.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the map height.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns row-major terrain input.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns initial actor records in caller-provided order.
  #[must_use]
  pub fn actors(&self) -> &[ScenarioActor] {
    &self.actors
  }
}
