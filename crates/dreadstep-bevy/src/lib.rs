//! Deterministic Bevy presentation boundary for Dreadstep.
//!
//! [`PresentationState`] owns a core world and translates human-client intent into the same
//! semantic commands used by headless and agent adapters. It deliberately exposes only immutable
//! projections and core outcomes; rendering, ECS storage, windowing, and presentation effects
//! remain outside authoritative game state.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use bevy::app::{App, Plugin, Update};
use bevy::ecs::{component::Component, entity::Entity, resource::Resource, world::World};
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_content::{ContentError, starter_floor};
use dreadstep_core::{
  ActionTime, Actor, ActorId, Command, CommandError, Direction, Event, GridMap, Position,
  ReplayTrace, StateDigest, Tile, WorldState,
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

/// Selects the core actor addressed by keyboard intents.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationInput {
  actor: ActorId,
}

impl PresentationInput {
  /// Creates keyboard control for one explicit actor identity.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self { actor }
  }

  /// Returns the actor addressed by keyboard intents.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }
}

/// A disposable focus projection for future camera and viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationFocus {
  actor: ActorId,
  position: Option<Position>,
}

impl PresentationFocus {
  /// Creates an empty focus projection for one controlled actor.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      position: None,
    }
  }

  /// Returns the actor whose position is being projected.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest known core position, or `None` for an unknown actor.
  #[must_use]
  pub const fn position(self) -> Option<Position> {
    self.position
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
  ready_at: dreadstep_core::ActionTime,
  alive: bool,
}

impl SceneActor {
  fn from_core(actor: &Actor) -> Self {
    Self {
      id: actor.id(),
      kind: actor.kind(),
      position: actor.position(),
      hit_points: actor.hit_points(),
      ready_at: actor.ready_at(),
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

  /// Returns the projected core scheduler readiness time.
  #[must_use]
  pub const fn ready_at(self) -> dreadstep_core::ActionTime {
    self.ready_at
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

/// The Bevy-owned handle for one deterministic presentation run.
///
/// The wrapped [`PresentationState`] remains the only source of simulation truth. Bevy systems
/// may read snapshots or submit explicit core commands through this resource, while ECS scene
/// components remain disposable projections.
#[derive(Debug, Eq, PartialEq, Resource)]
pub struct PresentationRuntime {
  state: PresentationState,
  output: Option<PresentationOutput>,
}

impl PresentationRuntime {
  /// Starts a runtime from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_run(seed)?,
      output: None,
    })
  }

  /// Wraps an already validated presentation state as an app resource.
  #[must_use]
  pub fn new(state: PresentationState) -> Self {
    Self {
      state,
      output: None,
    }
  }

  /// Returns the explicit seed preserved by the runtime.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.state.seed()
  }

  /// Returns a read-only snapshot of the authoritative core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    self.state.snapshot()
  }

  /// Returns the deterministic digest of accepted runtime commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.state.replay_digest()
  }

  /// Returns the latest accepted command output without consuming it.
  #[must_use]
  pub const fn output(&self) -> Option<&PresentationOutput> {
    self.output.as_ref()
  }

  /// Takes the latest accepted command output, if one is pending.
  pub fn take_output(&mut self) -> Option<PresentationOutput> {
    self.output.take()
  }

  /// Delegates one command to the wrapped presentation state and core simulation.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is rejected. Rejected commands do not
  /// mutate the core world or replay trace, and clear any stale output so consumers never observe
  /// an earlier command as feedback for a rejected one.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    self.output = None;
    let output = self.state.execute(command)?;
    self.output = Some(output.clone());
    Ok(output)
  }
}

/// A headless Bevy plugin that keeps disposable scene mirrors synchronized with runtime state.
///
/// The plugin expects [`PresentationRuntime`] to be inserted by the application. Until a runtime
/// is present, its update system is a safe no-op so app construction can install plugins before
/// selecting or restoring a run. Keyboard dispatch is also optional: it runs only when the app
/// provides [`PresentationInput`] and `ButtonInput<KeyCode>` resources.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, update_presentation);
  }
}

const KEY_PRIORITY: [KeyCode; 10] = [
  KeyCode::ArrowUp,
  KeyCode::ArrowDown,
  KeyCode::ArrowLeft,
  KeyCode::ArrowRight,
  KeyCode::KeyW,
  KeyCode::KeyS,
  KeyCode::KeyA,
  KeyCode::KeyD,
  KeyCode::Enter,
  KeyCode::Space,
];

fn update_presentation(world: &mut World) {
  dispatch_keyboard_input(world);
  sync_runtime_scene(world);
  sync_focus(world);
}

fn dispatch_keyboard_input(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(key) = world
    .get_resource::<ButtonInput<KeyCode>>()
    .and_then(|input| {
      KEY_PRIORITY
        .iter()
        .copied()
        .find(|key| input.just_pressed(*key))
    })
  else {
    return;
  };
  let Some(intent) = KeyboardIntent::from_key(key) else {
    return;
  };
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  {
    let Some(mut input) = world.get_resource_mut::<ButtonInput<KeyCode>>() else {
      return;
    };
    for supported_key in KEY_PRIORITY {
      input.clear_just_pressed(supported_key);
    }
  }
  let _ = world
    .resource_mut::<PresentationRuntime>()
    .execute(intent.command(actor));
}

fn sync_runtime_scene(world: &mut World) {
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  sync_scene(world, &snapshot);
}

fn sync_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(position) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
    .and_then(|snapshot| {
      snapshot
        .actors()
        .iter()
        .find(|record| record.id() == actor)
        .map(Actor::position)
    })
  else {
    if let Some(mut focus) = world.get_resource_mut::<PresentationFocus>() {
      focus.actor = actor;
      focus.position = None;
    }
    return;
  };
  if let Some(mut focus) = world.get_resource_mut::<PresentationFocus>() {
    focus.actor = actor;
    focus.position = Some(position);
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
  let mut existing_tiles: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneTile)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, tile)| {
        entities
          .entry(tile_key(tile.position()))
          .or_default()
          .push(entity);
        entities
      })
  };
  for entities in existing_tiles.values_mut() {
    entities.sort_unstable();
  }
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
  for (key, entities) in &existing_tiles {
    if expected_tiles.contains(key) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for (position, terrain) in positions
    .iter()
    .copied()
    .zip(snapshot.tiles().iter().copied())
  {
    if let Some(entity) = existing_tiles
      .get(&tile_key(position))
      .and_then(|entities| entities.first())
    {
      scene
        .entity_mut(*entity)
        .insert(SceneTile::new(position, terrain));
    } else {
      scene.spawn(SceneTile::new(position, terrain));
    }
  }

  let mut existing_actors: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneActor)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, actor)| {
        entities.entry(actor.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_actors.values_mut() {
    entities.sort_unstable();
  }
  let expected_actors: BTreeSet<_> = snapshot.actors().iter().map(Actor::id).collect();
  for (actor_id, entities) in &existing_actors {
    if expected_actors.contains(actor_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for actor in snapshot.actors() {
    let scene_actor = SceneActor::from_core(actor);
    if let Some(entity) = existing_actors
      .get(&actor.id())
      .and_then(|entities| entities.first())
    {
      scene.entity_mut(*entity).insert(scene_actor);
    } else {
      scene.spawn(scene_actor);
    }
  }
}
