//! Deterministic Bevy presentation boundary for Dreadstep.
//!
//! [`PresentationState`] owns a core world and translates human-client intent into the same
//! semantic commands used by headless and agent adapters. It deliberately exposes only immutable
//! projections and core outcomes; rendering, ECS storage, windowing, and presentation effects
//! remain outside authoritative game state.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::{component::Component, entity::Entity, world::World};
use bevy::input::keyboard::KeyCode;
use dreadstep_content::{ContentError, starter_floor};
use dreadstep_core::{
  ActionTime, Actor, ActorId, Command, CommandError, Direction, Event, GridMap, ReplayTrace,
  StateDigest, Tile, WorldState,
};

/// A supported keyboard intent before it is addressed to one core actor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyboardIntent {
  /// Move one tile in a cardinal direction.
  Move(Direction),
  /// Spend one standard action without moving.
  Wait,
}

impl KeyboardIntent {
  /// Converts supported arrow/WASD and wait keys into presentation intent.
  #[must_use]
  pub const fn from_key(key: KeyCode) -> Option<Self> {
    match key {
      KeyCode::ArrowUp | KeyCode::KeyW => Some(Self::Move(Direction::North)),
      KeyCode::ArrowDown | KeyCode::KeyS => Some(Self::Move(Direction::South)),
      KeyCode::ArrowLeft | KeyCode::KeyA => Some(Self::Move(Direction::West)),
      KeyCode::ArrowRight | KeyCode::KeyD => Some(Self::Move(Direction::East)),
      KeyCode::Enter | KeyCode::Space => Some(Self::Wait),
      _ => None,
    }
  }

  /// Addresses this intent to an explicit actor as a canonical core command.
  #[must_use]
  pub const fn command(self, actor: ActorId) -> Command {
    match self {
      Self::Move(direction) => Command::Move { actor, direction },
      Self::Wait => Command::Wait { actor },
    }
  }
}

/// A disposable ECS mirror of one projected map tile.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneTile {
  position: dreadstep_core::Position,
  terrain: Tile,
}

impl SceneTile {
  fn new(position: dreadstep_core::Position, terrain: Tile) -> Self {
    Self { position, terrain }
  }

  /// Returns the core position represented by this scene tile.
  #[must_use]
  pub const fn position(self) -> dreadstep_core::Position {
    self.position
  }

  /// Returns the projected terrain value.
  #[must_use]
  pub const fn terrain(self) -> Tile {
    self.terrain
  }
}

/// A disposable ECS mirror of one projected actor record.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneActor {
  id: ActorId,
  kind: dreadstep_core::ActorKind,
  position: dreadstep_core::Position,
  hit_points: dreadstep_core::HitPoints,
  alive: bool,
}

impl SceneActor {
  fn from_core(actor: &Actor) -> Self {
    Self {
      id: actor.id(),
      kind: actor.kind(),
      position: actor.position(),
      hit_points: actor.hit_points(),
      alive: actor.is_alive(),
    }
  }

  /// Returns the stable actor identity.
  #[must_use]
  pub const fn id(self) -> ActorId {
    self.id
  }

  /// Returns the actor kind.
  #[must_use]
  pub const fn kind(self) -> dreadstep_core::ActorKind {
    self.kind
  }

  /// Returns the projected actor position.
  #[must_use]
  pub const fn position(self) -> dreadstep_core::Position {
    self.position
  }

  /// Returns the projected hit points.
  #[must_use]
  pub const fn hit_points(self) -> dreadstep_core::HitPoints {
    self.hit_points
  }

  /// Returns whether the projected actor is living.
  #[must_use]
  pub const fn is_alive(self) -> bool {
    self.alive
  }
}

/// A deterministic read-only projection consumed by future map and actor renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationSnapshot {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<Actor>,
  current_time: ActionTime,
  next_actor: Option<ActorId>,
  digest: StateDigest,
}

impl PresentationSnapshot {
  fn from_world(world: &WorldState) -> Self {
    Self {
      width: world.map().width(),
      height: world.map().height(),
      tiles: world.map().tiles().to_vec(),
      actors: world.actors().cloned().collect(),
      current_time: world.current_time(),
      next_actor: world.next_actor(),
      digest: world.digest(),
    }
  }

  /// Returns the map width in tiles.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the map height in tiles.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns immutable row-major terrain for the projected map.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns immutable actor records in stable [`ActorId`] order.
  #[must_use]
  pub fn actors(&self) -> &[Actor] {
    &self.actors
  }

  /// Returns the current core action time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns the actor currently selected by core scheduling.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the stable core state digest for this projection.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }
}

/// Evidence returned after one accepted presentation command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationOutput {
  events: Vec<Event>,
  snapshot: PresentationSnapshot,
}

impl PresentationOutput {
  /// Returns semantic core events in deterministic execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the post-command presentation projection.
  #[must_use]
  pub const fn snapshot(&self) -> &PresentationSnapshot {
    &self.snapshot
  }
}

/// A deterministic presentation adapter around one core world and replay trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationState {
  seed: u64,
  world: WorldState,
  trace: ReplayTrace,
}

impl PresentationState {
  /// Starts a presentation state from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_floor()?))
  }

  /// Creates a presentation state around an already validated core world.
  #[must_use]
  pub fn new(seed: u64, world: WorldState) -> Self {
    Self {
      seed,
      world,
      trace: ReplayTrace::new(seed),
    }
  }

  /// Returns the explicit run seed preserved by this presentation boundary.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns a stable read-only projection of the current core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    PresentationSnapshot::from_world(&self.world)
  }

  /// Returns the deterministic digest of accepted presentation commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.trace.digest()
  }

  /// Executes one canonical command through core and projects its semantic outcome.
  ///
  /// Rejected commands are not recorded. Core validates scheduling and gameplay, so this adapter
  /// does not duplicate those rules or mutate presentation state on an error.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is not accepted.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    let result = self.world.execute(command)?;
    self.trace.record(command);
    Ok(PresentationOutput {
      events: result.events().to_vec(),
      snapshot: self.snapshot(),
    })
  }

  /// Returns the immutable core map for adapters that need map-specific inspection.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    self.world.map()
  }
}

fn tile_key(position: dreadstep_core::Position) -> (i32, i32) {
  (position.x(), position.y())
}

fn scene_position(index: usize, width: usize) -> Option<dreadstep_core::Position> {
  if width == 0 {
    return None;
  }
  Some(dreadstep_core::Position::new(
    i32::try_from(index % width).ok()?,
    i32::try_from(index / width).ok()?,
  ))
}

/// Synchronizes a complete core projection into disposable Bevy scene entities.
///
/// Tile entities are keyed by position and actor entities by [`ActorId`]. Existing entities keep
/// their Bevy identity when their key remains in the snapshot; stale entities are despawned before
/// new keys are spawned. The ECS world is only a presentation mirror and cannot change core state.
pub fn sync_scene(scene: &mut World, snapshot: &PresentationSnapshot) {
  let existing_tiles: BTreeMap<_, _> = {
    let mut query = scene.query::<(Entity, &SceneTile)>();
    query
      .iter(scene)
      .map(|(entity, tile)| (tile_key(tile.position()), entity))
      .collect()
  };
  let Ok(width) = usize::try_from(snapshot.width()) else {
    return;
  };
  let Some(positions) = snapshot
    .tiles()
    .iter()
    .enumerate()
    .map(|(index, _)| scene_position(index, width))
    .collect::<Option<Vec<_>>>()
  else {
    return;
  };
  let expected_tiles: BTreeSet<_> = positions.iter().copied().map(tile_key).collect();
  for (_key, entity) in existing_tiles
    .iter()
    .filter(|(key, _)| !expected_tiles.contains(key))
  {
    let _ = scene.despawn(*entity);
  }
  for (position, terrain) in positions
    .iter()
    .copied()
    .zip(snapshot.tiles().iter().copied())
  {
    if let Some(entity) = existing_tiles.get(&tile_key(position)) {
      scene
        .entity_mut(*entity)
        .insert(SceneTile::new(position, terrain));
    } else {
      scene.spawn(SceneTile::new(position, terrain));
    }
  }

  let existing_actors: BTreeMap<_, _> = {
    let mut query = scene.query::<(Entity, &SceneActor)>();
    query
      .iter(scene)
      .map(|(entity, actor)| (actor.id(), entity))
      .collect()
  };
  let expected_actors: BTreeSet<_> = snapshot.actors().iter().map(Actor::id).collect();
  for (_actor_id, entity) in existing_actors
    .iter()
    .filter(|(actor_id, _)| !expected_actors.contains(actor_id))
  {
    let _ = scene.despawn(*entity);
  }
  for actor in snapshot.actors() {
    let scene_actor = SceneActor::from_core(actor);
    if let Some(entity) = existing_actors.get(&actor.id()) {
      scene.entity_mut(*entity).insert(scene_actor);
    } else {
      scene.spawn(scene_actor);
    }
  }
}
