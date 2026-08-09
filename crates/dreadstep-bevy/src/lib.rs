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
use dreadstep_content::{ContentError, starter_floor, starter_item_floor};
use dreadstep_core::{
  ActionTime, Actor, ActorId, Command, CommandError, Direction, Event, GridMap, GroundItemStack,
  Item, ItemDefinitionId, ItemId, Position, ReplayTrace, StateDigest, Tile, WorldState,
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

/// A disposable camera-anchor projection for future viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationCamera {
  actor: ActorId,
  center: Option<Position>,
}

impl PresentationCamera {
  /// Creates a camera anchor for one controlled actor before a runtime projection exists.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      center: None,
    }
  }

  /// Returns the actor whose position supplies this camera anchor.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest authoritative center, or `None` for an unknown actor.
  #[must_use]
  pub const fn center(self) -> Option<Position> {
    self.center
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

/// A disposable ECS mirror of one opaque item projected on the ground.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneGroundItem {
  id: ItemId,
  definition: ItemDefinitionId,
  position: Position,
  stack_index: usize,
}

impl SceneGroundItem {
  fn from_core(position: Position, stack_index: usize, item: Item) -> Self {
    Self {
      id: item.id(),
      definition: item.definition(),
      position,
      stack_index,
    }
  }

  /// Returns the globally unique item identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the opaque content reference carried by this item instance.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns the map position where this item is projected.
  #[must_use]
  pub const fn position(self) -> Position {
    self.position
  }

  /// Returns this item's zero-based insertion-order index within its ground stack.
  #[must_use]
  pub const fn stack_index(self) -> usize {
    self.stack_index
  }
}

/// A disposable ECS mirror of one opaque item in an actor inventory.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneInventoryItem {
  id: ItemId,
  owner: ActorId,
  definition: ItemDefinitionId,
  inventory_index: usize,
}

impl SceneInventoryItem {
  fn from_core(owner: ActorId, inventory_index: usize, item: Item) -> Self {
    Self {
      id: item.id(),
      owner,
      definition: item.definition(),
      inventory_index,
    }
  }

  /// Returns the globally unique item identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the actor that currently owns this item instance.
  #[must_use]
  pub const fn owner(self) -> ActorId {
    self.owner
  }

  /// Returns the opaque content reference carried by this item instance.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns this item's zero-based insertion-order index in its owner's inventory.
  #[must_use]
  pub const fn inventory_index(self) -> usize {
    self.inventory_index
  }
}

/// A marker for the keyed scene entity representing the selected actor.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SceneFocus;

/// A disposable ECS mirror of the projected camera center.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneCamera {
  center: Position,
}

impl SceneCamera {
  /// Creates a disposable camera projection for a known core center.
  #[must_use]
  pub const fn new(center: Position) -> Self {
    Self { center }
  }

  /// Returns the authoritative map position represented by this camera anchor.
  #[must_use]
  pub const fn center(self) -> Position {
    self.center
  }
}

/// A deterministic read-only projection consumed by future map, actor, ground-item, and inventory
/// renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationSnapshot {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<Actor>,
  ground_items: Vec<GroundItemStack>,
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
      ground_items: world.ground_items().to_vec(),
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

  /// Returns immutable ground-item stacks in core-provided row-major and insertion order.
  #[must_use]
  pub fn ground_items(&self) -> &[GroundItemStack] {
    &self.ground_items
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

  /// Starts a presentation state from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_item_floor()?))
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

  /// Starts a runtime from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_item_run(seed)?,
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
  sync_scene_focus(world);
  sync_camera(world);
  sync_scene_camera(world);
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
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let position = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut focus) = world.get_resource_mut::<PresentationFocus>() else {
    return;
  };
  focus.actor = actor;
  focus.position = position;
}

fn sync_scene_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  if world.get_resource::<PresentationFocus>().is_none() {
    return;
  }
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let actor_is_known = snapshot.actors().iter().any(|record| record.id() == actor);
  let (marked_entities, target_entity) = {
    let mut query = world.query::<(Entity, Option<&SceneActor>, Option<&SceneFocus>)>();
    query.iter(world).fold(
      (Vec::new(), None),
      |(mut marked_entities, target_entity), (entity, scene_actor, marker)| {
        if marker.is_some() {
          marked_entities.push(entity);
        }
        let target_entity = target_entity.or_else(|| {
          actor_is_known
            .then(|| {
              scene_actor
                .filter(|record| record.id() == actor)
                .map(|_| entity)
            })
            .flatten()
        });
        (marked_entities, target_entity)
      },
    )
  };
  for entity in marked_entities {
    if Some(entity) != target_entity {
      world.entity_mut(entity).remove::<SceneFocus>();
    }
  }
  if let Some(entity) = target_entity {
    world.entity_mut(entity).insert(SceneFocus);
  }
}

fn sync_camera(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let center = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut camera) = world.get_resource_mut::<PresentationCamera>() else {
    return;
  };
  camera.actor = actor;
  camera.center = center;
}

fn sync_scene_camera(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
  {
    return;
  }
  let Some(camera) = world.get_resource::<PresentationCamera>() else {
    return;
  };
  let center = camera.center();
  let Some(center) = center else {
    let mut query = world.query::<(Entity, &SceneCamera)>();
    let stale_entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    for entity in stale_entities {
      let _ = world.despawn(entity);
    }
    return;
  };
  let mut query = world.query::<(Entity, &SceneCamera)>();
  let mut existing = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  // Entity's full ordering includes generation; the allocation index is the stable key for
  // retaining the original camera anchor when duplicate disposable mirrors are cleaned up.
  existing.sort_unstable_by_key(|entity| entity.index());
  if let Some(entity) = existing.first().copied() {
    world.entity_mut(entity).insert(SceneCamera::new(center));
    for stale in existing.into_iter().skip(1) {
      let _ = world.despawn(stale);
    }
  } else {
    world.spawn(SceneCamera::new(center));
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
/// Tile entities are keyed by position, actor entities by [`ActorId`], and ground or inventory-item
/// entities by globally unique [`ItemId`]. Existing entities keep their Bevy identity when their key
/// remains in the snapshot; stale entities are despawned before new keys are spawned. The ECS world
/// is only a presentation mirror and cannot change core state.
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

  sync_ground_items(scene, snapshot);
  sync_inventory_items(scene, snapshot);
}

fn sync_ground_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_ground_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneGroundItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_ground_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_ground_items: BTreeSet<_> = snapshot
    .ground_items()
    .iter()
    .flat_map(|stack| stack.items().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_ground_items {
    if expected_ground_items.contains(item_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for stack in snapshot.ground_items() {
    for (stack_index, item) in stack.items().iter().enumerate() {
      let scene_item = SceneGroundItem::from_core(stack.position(), stack_index, *item);
      if let Some(entity) = existing_ground_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene.entity_mut(*entity).insert(scene_item);
      } else {
        scene.spawn(scene_item);
      }
    }
  }
}

fn sync_inventory_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_inventory_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneInventoryItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_inventory_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_inventory_items: BTreeSet<_> = snapshot
    .actors()
    .iter()
    .flat_map(|actor| actor.inventory().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_inventory_items {
    if expected_inventory_items.contains(item_id) {
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
    for (inventory_index, item) in actor.inventory().iter().enumerate() {
      let scene_item = SceneInventoryItem::from_core(actor.id(), inventory_index, *item);
      if let Some(entity) = existing_inventory_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene.entity_mut(*entity).insert(scene_item);
      } else {
        scene.spawn(scene_item);
      }
    }
  }
}
