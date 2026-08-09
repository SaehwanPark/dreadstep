//! Deterministic game rules and domain state for Dreadstep.
//!
//! This crate owns semantic commands, events, and domain errors. It stays independent of
//! presentation, transport, storage, wall-clock time, and operating-system services so
//! the same rules can serve tests, headless tools, agents, and human-facing clients.

#![forbid(unsafe_code)]

use std::{collections::BTreeMap, fmt};

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

/// A tile coordinate in the simulation's row-major grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
  x: i32,
  y: i32,
}

impl Position {
  /// Creates a coordinate from its horizontal and vertical components.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Returns the horizontal component.
  #[must_use]
  pub const fn x(self) -> i32 {
    self.x
  }

  /// Returns the vertical component.
  #[must_use]
  pub const fn y(self) -> i32 {
    self.y
  }

  /// Returns the adjacent coordinate in the supplied cardinal direction.
  #[must_use]
  pub const fn translated(self, direction: Direction) -> Self {
    let (dx, dy) = direction.delta();
    Self {
      x: self.x.saturating_add(dx),
      y: self.y.saturating_add(dy),
    }
  }
}

/// The four movement directions supported by the first rules-kernel slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
  /// One row toward decreasing vertical coordinates.
  North,
  /// One row toward increasing vertical coordinates.
  South,
  /// One column toward decreasing horizontal coordinates.
  West,
  /// One column toward increasing horizontal coordinates.
  East,
}

impl Direction {
  const fn delta(self) -> (i32, i32) {
    match self {
      Self::North => (0, -1),
      Self::South => (0, 1),
      Self::West => (-1, 0),
      Self::East => (1, 0),
    }
  }
}

/// The terrain occupying one grid cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tile {
  /// A cell that actors may enter when it is not occupied.
  Floor,
  /// A cell that blocks movement.
  Wall,
}

impl Tile {
  /// Returns whether this tile permits an actor to enter it.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    matches!(self, Self::Floor)
  }
}

/// Errors produced while constructing a rectangular grid map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
  /// The width is zero, so no columns exist.
  ZeroWidth,
  /// The height is zero, so no rows exist.
  ZeroHeight,
  /// The dimensions cannot be represented as an in-memory tile buffer.
  TooLarge {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// A dimension would contain coordinates outside the signed position domain.
  CoordinateRange {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// The tile buffer does not match the dimensions.
  TileCountMismatch {
    /// The number of tiles implied by the dimensions.
    expected: usize,
    /// The number of tiles supplied by the caller.
    actual: usize,
  },
}

impl fmt::Display for MapError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ZeroWidth => formatter.write_str("map width must be greater than zero"),
      Self::ZeroHeight => formatter.write_str("map height must be greater than zero"),
      Self::TooLarge { width, height } => {
        write!(formatter, "map dimensions {width}x{height} are too large")
      }
      Self::CoordinateRange { width, height } => write!(
        formatter,
        "map dimensions {width}x{height} exceed the signed position range"
      ),
      Self::TileCountMismatch { expected, actual } => {
        write!(
          formatter,
          "map needs {expected} tiles but received {actual}"
        )
      }
    }
  }
}

impl std::error::Error for MapError {}

/// A finite, row-major rectangular map of terrain tiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridMap {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
}

impl GridMap {
  /// Creates a map filled with one terrain tile.
  ///
  /// # Errors
  ///
  /// Returns [`MapError::ZeroWidth`], [`MapError::ZeroHeight`], [`MapError::CoordinateRange`],
  /// or [`MapError::TooLarge`] when the requested dimensions cannot describe a valid buffer.
  pub fn filled(width: u32, height: u32, tile: Tile) -> Result<Self, MapError> {
    let tile_count = tile_count(width, height)?;
    Ok(Self {
      width,
      height,
      tiles: vec![tile; tile_count],
    })
  }

  /// Creates a map from row-major tiles, validating the dimensions and buffer length.
  ///
  /// # Errors
  ///
  /// Returns a [`MapError`] when a dimension is zero, the dimensions are too large, or the
  /// supplied tile count does not match the dimensions.
  pub fn from_tiles(width: u32, height: u32, tiles: Vec<Tile>) -> Result<Self, MapError> {
    let expected = tile_count(width, height)?;
    if tiles.len() != expected {
      return Err(MapError::TileCountMismatch {
        expected,
        actual: tiles.len(),
      });
    }
    Ok(Self {
      width,
      height,
      tiles,
    })
  }

  /// Returns the number of columns in the map.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the number of rows in the map.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns whether a position is inside the map bounds.
  #[must_use]
  pub fn in_bounds(&self, position: Position) -> bool {
    position.x >= 0
      && position.y >= 0
      && u32::try_from(position.x).is_ok_and(|x| x < self.width)
      && u32::try_from(position.y).is_ok_and(|y| y < self.height)
  }

  /// Returns the terrain at a position, or `None` outside the map.
  #[must_use]
  pub fn tile_at(&self, position: Position) -> Option<Tile> {
    self
      .in_bounds(position)
      .then(|| self.tiles[self.index(position)])
  }

  /// Returns whether an actor may enter a position based on terrain alone.
  #[must_use]
  pub fn is_walkable(&self, position: Position) -> bool {
    self.tile_at(position).is_some_and(Tile::is_walkable)
  }

  fn index(&self, position: Position) -> usize {
    let x = usize::try_from(position.x).expect("in-bounds positions have non-negative x");
    let y = usize::try_from(position.y).expect("in-bounds positions have non-negative y");
    let width = usize::try_from(self.width).expect("map width must fit usize");
    y * width + x
  }
}

fn tile_count(width: u32, height: u32) -> Result<usize, MapError> {
  if width == 0 {
    return Err(MapError::ZeroWidth);
  }
  if height == 0 {
    return Err(MapError::ZeroHeight);
  }
  if width > i32::MAX as u32 || height > i32::MAX as u32 {
    return Err(MapError::CoordinateRange { width, height });
  }
  usize::try_from(u64::from(width) * u64::from(height))
    .map_err(|_| MapError::TooLarge { width, height })
}

/// The kind of actor represented in the world.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActorKind {
  /// The player-controlled actor.
  Player,
  /// An actor controlled by the simulation or a future AI adapter.
  Enemy,
}

/// An integer timestamp used by the deterministic action scheduler.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionTime(u64);

impl ActionTime {
  /// Creates an action timestamp from its numeric value.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric timestamp.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }

  fn checked_add(self, cost: ActionCost) -> Option<Self> {
    self.0.checked_add(cost.0).map(Self)
  }
}

/// A non-negative integer action duration.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionCost(u64);

impl ActionCost {
  /// The fixed cost used by movement and waiting in this slice.
  pub const STANDARD: Self = Self(1);

  /// Creates an action cost from its numeric value.
  #[must_use]
  pub const fn new(value: u64) -> Self {
    Self(value)
  }

  /// Returns the numeric cost.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

/// An actor with a stable identity, kind, position, and next ready time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actor {
  id: ActorId,
  kind: ActorKind,
  position: Position,
  ready_at: ActionTime,
}

impl Actor {
  /// Creates an actor that is ready at the beginning of the world timeline.
  #[must_use]
  pub const fn new(id: ActorId, kind: ActorKind, position: Position) -> Self {
    Self {
      id,
      kind,
      position,
      ready_at: ActionTime::new(0),
    }
  }

  /// Returns this actor's stable identity.
  #[must_use]
  pub const fn id(&self) -> ActorId {
    self.id
  }

  /// Returns this actor's kind.
  #[must_use]
  pub const fn kind(&self) -> ActorKind {
    self.kind
  }

  /// Returns this actor's current position.
  #[must_use]
  pub const fn position(&self) -> Position {
    self.position
  }

  /// Returns the timestamp when this actor can next act.
  #[must_use]
  pub const fn ready_at(&self) -> ActionTime {
    self.ready_at
  }
}

/// A command interpreted by the deterministic rules kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
  /// Move one tile in a cardinal direction.
  Move {
    /// The actor issuing the command.
    actor: ActorId,
    /// The direction of movement.
    direction: Direction,
  },
  /// Spend one standard action without changing position.
  Wait {
    /// The actor issuing the command.
    actor: ActorId,
  },
}

impl Command {
  const fn actor(self) -> ActorId {
    match self {
      Self::Move { actor, .. } | Self::Wait { actor } => actor,
    }
  }
}

/// The reason a movement command could not enter its destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockReason {
  /// The destination is outside the map or is a blocking terrain tile.
  Terrain,
  /// The destination is occupied by another actor.
  Actor(ActorId),
}

/// A semantic outcome emitted by a world transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
  /// An actor entered an unoccupied walkable tile.
  Moved {
    /// The actor that moved.
    actor: ActorId,
    /// The position before movement.
    from: Position,
    /// The position after movement.
    to: Position,
  },
  /// An actor attempted movement but remained in place.
  MovementBlocked {
    /// The actor that attempted movement.
    actor: ActorId,
    /// The position before the attempt.
    from: Position,
    /// The requested destination.
    to: Position,
    /// Why the destination could not be entered.
    reason: BlockReason,
  },
  /// An actor spent a standard action without moving.
  Waited {
    /// The actor that waited.
    actor: ActorId,
    /// The action time at which the wait began.
    at: ActionTime,
  },
}

/// Errors produced while constructing a world state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
  /// Two actors use the same stable identity.
  DuplicateActorId(ActorId),
  /// An actor starts outside the map.
  ActorOutOfBounds {
    /// The actor outside the map.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// An actor starts on a blocking terrain tile.
  ActorOnBlockedTile {
    /// The actor on blocked terrain.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// Two distinct actors start on one position.
  OverlappingActors {
    /// The actor inserted first.
    first: ActorId,
    /// The actor that overlaps the first actor.
    second: ActorId,
    /// The shared position.
    position: Position,
  },
}

impl fmt::Display for WorldError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::DuplicateActorId(actor) => {
        write!(formatter, "actor id {} is duplicated", actor.value())
      }
      Self::ActorOutOfBounds { actor, position } => write!(
        formatter,
        "actor {} starts out of bounds at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::ActorOnBlockedTile { actor, position } => write!(
        formatter,
        "actor {} starts on blocked tile at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::OverlappingActors {
        first,
        second,
        position,
      } => write!(
        formatter,
        "actors {} and {} overlap at ({}, {})",
        first.value(),
        second.value(),
        position.x(),
        position.y()
      ),
    }
  }
}

impl std::error::Error for WorldError {}

/// Errors produced when a command cannot be applied to the current world state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
  /// The command addresses no actor in the world.
  UnknownActor(ActorId),
  /// The command addresses an actor other than the deterministic next actor.
  ActorNotScheduled {
    /// The actor addressed by the command.
    requested: ActorId,
    /// The actor selected by ready time and identity.
    scheduled: ActorId,
  },
  /// The actor's next ready time would overflow the integer timeline.
  ScheduleOverflow(ActorId),
}

impl fmt::Display for CommandError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
      Self::ActorNotScheduled {
        requested,
        scheduled,
      } => write!(
        formatter,
        "actor {} is not scheduled; actor {} must act next",
        requested.value(),
        scheduled.value()
      ),
      Self::ScheduleOverflow(actor) => {
        write!(
          formatter,
          "actor {} cannot advance its ready time",
          actor.value()
        )
      }
    }
  }
}

impl std::error::Error for CommandError {}

/// The observable result of one accepted command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResult {
  events: Vec<Event>,
  next_actor: Option<ActorId>,
  current_time: ActionTime,
}

impl ActionResult {
  /// Returns semantic events emitted by this command.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the actor selected to act after this command.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the world's minimum ready time after this command.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }
}

/// The authoritative deterministic state for the current grid slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldState {
  map: GridMap,
  actors: BTreeMap<ActorId, Actor>,
  current_time: ActionTime,
}

impl WorldState {
  /// Validates and creates a world from a map and its initial actors.
  ///
  /// # Errors
  ///
  /// Returns a [`WorldError`] when an actor identity is duplicated, an actor is outside the
  /// map, an actor starts on blocking terrain, or two actors overlap.
  pub fn new(map: GridMap, actors: Vec<Actor>) -> Result<Self, WorldError> {
    let mut indexed_actors = BTreeMap::new();
    for actor in actors {
      let actor_id = actor.id();
      let position = actor.position();
      if indexed_actors.contains_key(&actor_id) {
        return Err(WorldError::DuplicateActorId(actor_id));
      }
      if !map.in_bounds(position) {
        return Err(WorldError::ActorOutOfBounds {
          actor: actor_id,
          position,
        });
      }
      if !map.is_walkable(position) {
        return Err(WorldError::ActorOnBlockedTile {
          actor: actor_id,
          position,
        });
      }
      if let Some(first) = indexed_actors
        .values()
        .find(|existing: &&Actor| existing.position() == position)
      {
        return Err(WorldError::OverlappingActors {
          first: first.id(),
          second: actor_id,
          position,
        });
      }
      indexed_actors.insert(actor_id, actor);
    }
    let current_time = indexed_actors
      .values()
      .map(Actor::ready_at)
      .min()
      .unwrap_or(ActionTime::new(0));
    Ok(Self {
      map,
      actors: indexed_actors,
      current_time,
    })
  }

  /// Returns the immutable map owned by this world.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    &self.map
  }

  /// Returns an actor by stable identity.
  #[must_use]
  pub fn actor(&self, actor: ActorId) -> Option<&Actor> {
    self.actors.get(&actor)
  }

  /// Returns the world's minimum ready time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns the actor selected by ready time, then stable identity.
  #[must_use]
  pub fn next_actor(&self) -> Option<ActorId> {
    self
      .actors
      .values()
      .min_by_key(|actor| (actor.ready_at(), actor.id()))
      .map(Actor::id)
  }

  /// Applies one command from the deterministically scheduled actor.
  ///
  /// # Errors
  ///
  /// Returns [`CommandError::UnknownActor`] for an unknown identity,
  /// [`CommandError::ActorNotScheduled`] when a different actor must act first, or
  /// [`CommandError::ScheduleOverflow`] if the integer timeline cannot advance.
  pub fn execute(&mut self, command: Command) -> Result<ActionResult, CommandError> {
    let actor_id = command.actor();
    let ready_at = self
      .actors
      .get(&actor_id)
      .map(Actor::ready_at)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if let Some(scheduled) = self.next_actor()
      && scheduled != actor_id
    {
      return Err(CommandError::ActorNotScheduled {
        requested: actor_id,
        scheduled,
      });
    }
    let next_ready_at = ready_at
      .checked_add(ActionCost::STANDARD)
      .ok_or(CommandError::ScheduleOverflow(actor_id))?;
    let event = match command {
      Command::Move { direction, .. } => {
        let from = self
          .actors
          .get(&actor_id)
          .map(Actor::position)
          .ok_or(CommandError::UnknownActor(actor_id))?;
        let to = from.translated(direction);
        if !self.map.is_walkable(to) {
          Event::MovementBlocked {
            actor: actor_id,
            from,
            to,
            reason: BlockReason::Terrain,
          }
        } else if let Some(blocker) = self.actor_at(to) {
          Event::MovementBlocked {
            actor: actor_id,
            from,
            to,
            reason: BlockReason::Actor(blocker),
          }
        } else {
          self
            .actors
            .get_mut(&actor_id)
            .ok_or(CommandError::UnknownActor(actor_id))?
            .position = to;
          Event::Moved {
            actor: actor_id,
            from,
            to,
          }
        }
      }
      Command::Wait { .. } => Event::Waited {
        actor: actor_id,
        at: self.current_time,
      },
    };
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .ready_at = next_ready_at;
    self.current_time = self
      .next_actor()
      .and_then(|next| self.actors.get(&next).map(Actor::ready_at))
      .unwrap_or(next_ready_at);
    Ok(ActionResult {
      events: vec![event],
      next_actor: self.next_actor(),
      current_time: self.current_time,
    })
  }

  fn actor_at(&self, position: Position) -> Option<ActorId> {
    self
      .actors
      .values()
      .find(|actor| actor.position() == position)
      .map(Actor::id)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn floor_map(width: u32, height: u32) -> GridMap {
    GridMap::filled(width, height, Tile::Floor).expect("test map should be valid")
  }

  #[test]
  fn moves_the_scheduled_actor_and_reports_the_next_actor() {
    let map = floor_map(3, 1);
    let mut world = WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      ],
    )
    .expect("test world should be valid");

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .expect("actor one is scheduled first");

    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().position(),
      Position::new(1, 0)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(result.next_actor(), Some(ActorId::new(2)));
    assert_eq!(
      result.events(),
      &[Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      }]
    );
  }

  #[test]
  fn reports_terrain_blocking_and_still_consumes_the_action() {
    let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Wall]).unwrap();
    let mut world = WorldState::new(
      map,
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .unwrap();

    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().position(),
      Position::new(0, 0)
    );
    assert_eq!(
      world.actor(ActorId::new(1)).unwrap().ready_at(),
      ActionTime::new(1)
    );
    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Terrain,
      }]
    );
  }

  #[test]
  fn reports_actor_blocking_without_moving_either_actor() {
    let map = floor_map(2, 1);
    let mut world = WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      ],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Actor(ActorId::new(2)),
      }]
    );
    assert_eq!(
      world.actor(ActorId::new(2)).unwrap().position(),
      Position::new(1, 0)
    );
  }

  #[test]
  fn treats_out_of_bounds_movement_as_terrain_blocking() {
    let mut world = WorldState::new(
      floor_map(1, 1),
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    )
    .unwrap();

    let result = world
      .execute(Command::Move {
        actor: ActorId::new(1),
        direction: Direction::West,
      })
      .unwrap();

    assert_eq!(
      result.events(),
      &[Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(-1, 0),
        reason: BlockReason::Terrain,
      }]
    );
  }

  #[test]
  fn scheduler_orders_equal_ready_times_by_actor_id() {
    let mut world = WorldState::new(
      floor_map(3, 1),
      vec![
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      ],
    )
    .unwrap();

    assert_eq!(world.next_actor(), Some(ActorId::new(1)));
    let first = world
      .execute(Command::Wait {
        actor: ActorId::new(1),
      })
      .unwrap();
    assert_eq!(first.next_actor(), Some(ActorId::new(2)));
    let second = world
      .execute(Command::Wait {
        actor: ActorId::new(2),
      })
      .unwrap();
    assert_eq!(second.next_actor(), Some(ActorId::new(1)));
  }

  #[test]
  fn rejects_a_command_for_an_unscheduled_actor() {
    let mut world = WorldState::new(
      floor_map(2, 1),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      ],
    )
    .unwrap();

    assert_eq!(
      world.execute(Command::Wait {
        actor: ActorId::new(2)
      }),
      Err(CommandError::ActorNotScheduled {
        requested: ActorId::new(2),
        scheduled: ActorId::new(1),
      })
    );
  }

  #[test]
  fn rejects_invalid_world_occupancy_and_map_data() {
    assert_eq!(GridMap::from_tiles(0, 1, vec![]), Err(MapError::ZeroWidth));
    assert_eq!(
      GridMap::from_tiles(2, 1, vec![Tile::Floor]),
      Err(MapError::TileCountMismatch {
        expected: 2,
        actual: 1,
      })
    );
    assert_eq!(
      GridMap::from_tiles(i32::MAX as u32 + 1, 1, vec![]),
      Err(MapError::CoordinateRange {
        width: i32::MAX as u32 + 1,
        height: 1,
      })
    );

    let map = GridMap::filled(2, 1, Tile::Floor).unwrap();
    assert_eq!(
      WorldState::new(
        map.clone(),
        vec![
          Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
          Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(0, 0)),
        ],
      ),
      Err(WorldError::OverlappingActors {
        first: ActorId::new(1),
        second: ActorId::new(2),
        position: Position::new(0, 0),
      })
    );

    assert_eq!(
      WorldState::new(
        map.clone(),
        vec![
          Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
          Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(1, 0)),
        ],
      ),
      Err(WorldError::DuplicateActorId(ActorId::new(1)))
    );

    assert_eq!(
      WorldState::new(
        map,
        vec![Actor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(2, 0),
        )],
      ),
      Err(WorldError::ActorOutOfBounds {
        actor: ActorId::new(1),
        position: Position::new(2, 0),
      })
    );

    let wall_map = GridMap::from_tiles(1, 1, vec![Tile::Wall]).unwrap();
    assert_eq!(
      WorldState::new(
        wall_map,
        vec![Actor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
        )],
      ),
      Err(WorldError::ActorOnBlockedTile {
        actor: ActorId::new(1),
        position: Position::new(0, 0),
      })
    );
  }
}
