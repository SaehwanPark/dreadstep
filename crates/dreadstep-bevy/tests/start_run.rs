//! Presentation startup over the authored content boundary.

use std::collections::BTreeMap;

use bevy::app::App;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRuntime, PresentationSnapshot, PresentationState, SceneActor,
  SceneGroundItem, SceneInventoryItem, SceneTile,
};
use dreadstep_content::{procedural_floor, starter_floor, starter_item_floor};
use dreadstep_core::{ActorId, ItemDefinitionId, ItemId, ReplayTrace};

#[test]
fn start_run_delegates_to_shared_content_and_preserves_seed() {
  let seed = 41;
  let state = PresentationState::start_run(seed).expect("starter content should validate");
  let expected = PresentationState::new(
    seed,
    starter_floor().expect("same starter content should validate"),
  );

  assert_eq!(state.seed(), seed);
  assert_eq!(state.snapshot(), expected.snapshot());
  assert_eq!(state.replay_digest(), ReplayTrace::new(seed).digest());
}

#[test]
fn start_item_run_delegates_to_shared_content_and_preserves_seed() {
  let seed = 43;
  let state =
    PresentationState::start_item_run(seed).expect("starter item content should validate");
  let expected = PresentationState::new(
    seed,
    starter_item_floor().expect("same starter item content should validate"),
  );
  let runtime =
    PresentationRuntime::start_item_run(seed).expect("starter item runtime should validate");

  assert_eq!(state.seed(), seed);
  assert_eq!(state.snapshot(), expected.snapshot());
  assert_eq!(runtime.snapshot(), expected.snapshot());
  assert_eq!(state.replay_digest(), ReplayTrace::new(seed).digest());
  assert_eq!(runtime.replay_digest(), ReplayTrace::new(seed).digest());
}

#[test]
fn start_procedural_run_delegates_to_seeded_content_and_preserves_seed_and_depth() {
  let seed = 47;
  let depth = 3;
  let state = PresentationState::start_procedural_run(seed, depth)
    .expect("procedural content should validate");
  let expected = PresentationState::new(
    seed,
    procedural_floor(seed, depth).expect("same procedural content should validate"),
  );
  let runtime = PresentationRuntime::start_procedural_run(seed, depth)
    .expect("procedural runtime should validate");

  assert_eq!(state.seed(), seed);
  assert_eq!(state.snapshot(), expected.snapshot());
  assert_eq!(runtime.snapshot(), expected.snapshot());
  assert_eq!(state.replay_digest(), ReplayTrace::new(seed).digest());
  assert_eq!(runtime.replay_digest(), ReplayTrace::new(seed).digest());
  assert_eq!(state.snapshot().actors()[1].hit_points().value(), 6);
}

#[test]
fn item_run_startup_projects_complete_inventory_scene() {
  let mut app = App::new();
  app.insert_resource(
    PresentationRuntime::start_item_run(43).expect("starter item runtime should validate"),
  );
  app.add_plugins(PresentationPlugin);

  app.update();

  let world = app.world_mut();
  let snapshot = world.resource::<PresentationRuntime>().snapshot();
  assert_complete_tile_scene(world, &snapshot);
  assert_complete_actor_scene(world, &snapshot);
  assert_eq!(world.query::<&SceneGroundItem>().iter(world).count(), 0);
  assert_eq!(world.query::<&SceneInventoryItem>().iter(world).count(), 3);
  let inventory: BTreeMap<_, _> = world
    .query::<&SceneInventoryItem>()
    .iter(world)
    .map(|item| {
      (
        item.id(),
        (item.owner(), item.definition(), item.inventory_index()),
      )
    })
    .collect();
  assert_eq!(
    inventory,
    BTreeMap::from([
      (
        ItemId::new(100),
        (ActorId::new(2), ItemDefinitionId::new(1), 0),
      ),
      (
        ItemId::new(101),
        (ActorId::new(1), ItemDefinitionId::new(2), 0),
      ),
      (
        ItemId::new(102),
        (ActorId::new(1), ItemDefinitionId::new(3), 1),
      ),
    ])
  );
  assert!(snapshot.ground_items().is_empty());
}

fn assert_complete_tile_scene(
  world: &mut bevy::ecs::world::World,
  snapshot: &PresentationSnapshot,
) {
  let width = usize::try_from(snapshot.width()).expect("snapshot width should fit usize");
  let expected: BTreeMap<_, _> = snapshot
    .tiles()
    .iter()
    .enumerate()
    .map(|(index, terrain)| {
      (
        (
          i32::try_from(index % width).expect("tile x should fit i32"),
          i32::try_from(index / width).expect("tile y should fit i32"),
        ),
        *terrain,
      )
    })
    .collect();
  let projected: BTreeMap<_, _> = world
    .query::<&SceneTile>()
    .iter(world)
    .map(|tile| ((tile.position().x(), tile.position().y()), tile.terrain()))
    .collect();
  assert_eq!(projected.len(), expected.len());
  assert_eq!(
    world.query::<&SceneTile>().iter(world).count(),
    expected.len()
  );
  assert_eq!(projected, expected);
}

fn assert_complete_actor_scene(
  world: &mut bevy::ecs::world::World,
  snapshot: &PresentationSnapshot,
) {
  let expected: BTreeMap<_, _> = snapshot
    .actors()
    .iter()
    .map(|actor| {
      (
        actor.id(),
        (
          actor.kind(),
          actor.position(),
          actor.hit_points(),
          actor.ready_at(),
          actor.is_alive(),
        ),
      )
    })
    .collect();
  let projected: BTreeMap<_, _> = world
    .query::<&SceneActor>()
    .iter(world)
    .map(|actor| {
      (
        actor.id(),
        (
          actor.kind(),
          actor.position(),
          actor.hit_points(),
          actor.ready_at(),
          actor.is_alive(),
        ),
      )
    })
    .collect();
  assert_eq!(projected.len(), expected.len());
  assert_eq!(
    world.query::<&SceneActor>().iter(world).count(),
    expected.len()
  );
  assert_eq!(projected, expected);
}
