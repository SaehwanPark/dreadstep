//! Deterministic headless viewport projection behavior.

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationCamera, PresentationFocus, PresentationInput, PresentationPlugin,
  PresentationRuntime, PresentationState, PresentationViewport, SceneActor, SceneGroundItem,
  SceneInventoryItem, SceneTile, SceneViewport,
};
use std::collections::BTreeMap;

use dreadstep_content::{starter_floor, starter_item_floor};
use dreadstep_core::{ActorId, ItemId, Position};

type TileProjection = BTreeMap<(i32, i32), (Entity, SceneTile)>;
type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;
type GroundProjection = BTreeMap<ItemId, (Entity, SceneGroundItem)>;
type InventoryProjection = BTreeMap<ItemId, (Entity, SceneInventoryItem)>;

fn viewport_app(actor: ActorId, width: u32, height: u32) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationFocus::new(actor));
  app.insert_resource(PresentationCamera::new(actor));
  app.insert_resource(
    PresentationViewport::new(width, height).expect("viewport dimensions should be non-zero"),
  );
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn edge_viewport_app(width: u32, height: u32) -> App {
  let mut world = starter_floor().expect("content should validate");
  world
    .teleport(ActorId::new(4), Position::new(3, 3))
    .expect("alternate floor position should validate");
  world
    .teleport(ActorId::new(1), Position::new(5, 3))
    .expect("edge floor position should validate");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationFocus::new(ActorId::new(1)));
  app.insert_resource(PresentationCamera::new(ActorId::new(1)));
  app.insert_resource(
    PresentationViewport::new(width, height).expect("viewport dimensions should be non-zero"),
  );
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn item_viewport_app(actor: ActorId, width: u32, height: u32) -> App {
  let mut world = starter_item_floor().expect("item content should validate");
  world
    .drop_item(ActorId::new(1), ItemId::new(101))
    .expect("starter item should be owned");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationFocus::new(actor));
  app.insert_resource(PresentationCamera::new(actor));
  app.insert_resource(
    PresentationViewport::new(width, height).expect("viewport dimensions should be non-zero"),
  );
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn complete_scene_projection(
  app: &mut App,
) -> (
  TileProjection,
  ActorProjection,
  GroundProjection,
  InventoryProjection,
) {
  let world = app.world_mut();
  let tiles = {
    let mut query = world.query::<(Entity, &SceneTile)>();
    query
      .iter(world)
      .map(|(entity, tile)| ((tile.position().x(), tile.position().y()), (entity, *tile)))
      .collect()
  };
  let actors = {
    let mut query = world.query::<(Entity, &SceneActor)>();
    query
      .iter(world)
      .map(|(entity, actor)| (actor.id(), (entity, *actor)))
      .collect()
  };
  let ground = {
    let mut query = world.query::<(Entity, &SceneGroundItem)>();
    query
      .iter(world)
      .map(|(entity, item)| (item.id(), (entity, *item)))
      .collect()
  };
  let inventory = {
    let mut query = world.query::<(Entity, &SceneInventoryItem)>();
    query
      .iter(world)
      .map(|(entity, item)| (item.id(), (entity, *item)))
      .collect()
  };
  (tiles, actors, ground, inventory)
}

fn viewport_entities(app: &mut App) -> Vec<(Entity, SceneViewport)> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneViewport)>();
  let mut entities = query
    .iter(world)
    .map(|(entity, viewport)| (entity, *viewport))
    .collect::<Vec<_>>();
  entities.sort_unstable_by_key(|(entity, _)| *entity);
  entities
}

fn assert_viewport(
  app: &mut App,
  origin: Option<Position>,
  effective_width: u32,
  effective_height: u32,
) {
  let projection = *app.world().resource::<PresentationViewport>();
  assert_eq!(projection.origin(), origin);
  assert_eq!(projection.effective_width(), effective_width);
  assert_eq!(projection.effective_height(), effective_height);
  match origin {
    Some(origin) => {
      let entities = viewport_entities(app);
      assert_eq!(entities.len(), 1);
      assert_eq!(entities[0].1.origin(), origin);
      assert_eq!(entities[0].1.width(), effective_width);
      assert_eq!(entities[0].1.height(), effective_height);
    }
    None => assert!(viewport_entities(app).is_empty()),
  }
}

#[test]
fn zero_sized_viewports_are_rejected() {
  assert!(PresentationViewport::new(0, 1).is_none());
  assert!(PresentationViewport::new(1, 0).is_none());
}

#[test]
fn startup_projects_centered_clamped_viewport() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);

  app.update();

  assert_viewport(&mut app, Some(Position::new(0, 0)), 3, 3);
}

#[test]
fn accepted_move_updates_origin_and_retains_scene_identity() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before = viewport_entities(&mut app)[0].0;
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_viewport(&mut app, Some(Position::new(1, 0)), 3, 3);
  assert_eq!(viewport_entities(&mut app)[0].0, before);
}

#[test]
fn edge_camera_is_clamped_to_the_map_bounds() {
  let mut app = edge_viewport_app(3, 3);

  app.update();

  assert_viewport(&mut app, Some(Position::new(4, 2)), 3, 3);
}

#[test]
fn oversized_viewport_shrinks_to_the_complete_map() {
  let mut app = edge_viewport_app(99, 99);

  app.update();

  assert_viewport(&mut app, Some(Position::new(0, 0)), 7, 5);
}

#[test]
fn changing_controlled_actor_updates_viewport_center() {
  let mut app = item_viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before = viewport_entities(&mut app)[0].0;
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = complete_scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(2)));

  app.update();

  assert_viewport(&mut app, Some(Position::new(4, 0)), 3, 3);
  assert_eq!(viewport_entities(&mut app)[0].0, before);
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
  assert_eq!(complete_scene_projection(&mut app), before_scene);
}

#[test]
fn unknown_actor_clears_viewport_without_mutating_runtime() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert_viewport(&mut app, None, 0, 0);
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before);
  assert_eq!(runtime.replay_digest(), before_digest);
}

#[test]
fn duplicate_viewports_are_deduplicated_deterministically() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let stable = viewport_entities(&mut app)[0].0;
  let duplicate = viewport_entities(&mut app)[0].1;
  app.world_mut().spawn(duplicate);

  app.update();

  let entities = viewport_entities(&mut app);
  assert_eq!(entities.len(), 1);
  assert_eq!(entities[0].0, stable);
  assert_eq!(entities[0].1.origin(), Position::new(0, 0));
  assert_eq!(entities[0].1.width(), 3);
  assert_eq!(entities[0].1.height(), 3);
}

#[test]
fn recycled_lower_entity_index_does_not_replace_the_retained_viewport() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let stable = viewport_entities(&mut app)[0].0;
  let tile_to_recycle = {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &SceneTile)>();
    query
      .iter(world)
      .map(|(entity, _)| entity)
      .find(|entity| *entity != stable)
      .expect("starter scene should have a tile to recycle")
  };
  app.world_mut().despawn(tile_to_recycle);
  let duplicate_component = viewport_entities(&mut app)[0].1;
  let recycled_entity = Entity::from_index_and_generation(
    tile_to_recycle.index(),
    tile_to_recycle.generation().after_versions(1),
  );
  let duplicate = app
    .world_mut()
    .spawn_at(recycled_entity, duplicate_component)
    .expect("despawned index should accept its next generation")
    .id();
  assert_ne!(stable, duplicate);
  assert!(duplicate.index() < stable.index());

  app.update();

  assert_eq!(viewport_entities(&mut app)[0].0, stable);
}

#[test]
fn missing_viewport_resource_preserves_existing_projection() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before = viewport_entities(&mut app);
  app.world_mut().remove_resource::<PresentationViewport>();

  app.update();

  assert_eq!(viewport_entities(&mut app), before);
}

#[test]
fn missing_camera_resource_preserves_existing_projection() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before = viewport_entities(&mut app);
  app.world_mut().remove_resource::<PresentationCamera>();

  app.update();

  assert_eq!(viewport_entities(&mut app), before);
}

#[test]
fn missing_runtime_preserves_viewport_projection() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before_resource = *app.world().resource::<PresentationViewport>();
  let before = viewport_entities(&mut app);
  app.world_mut().remove_resource::<PresentationRuntime>();

  app.update();

  assert_eq!(
    *app.world().resource::<PresentationViewport>(),
    before_resource
  );
  assert_eq!(viewport_entities(&mut app), before);
}

#[test]
fn missing_input_preserves_viewport_projection() {
  let mut app = viewport_app(ActorId::new(1), 3, 3);
  app.update();
  let before_resource = *app.world().resource::<PresentationViewport>();
  let before = viewport_entities(&mut app);
  app.world_mut().remove_resource::<PresentationInput>();

  app.update();

  assert_eq!(
    *app.world().resource::<PresentationViewport>(),
    before_resource
  );
  assert_eq!(viewport_entities(&mut app), before);
}
