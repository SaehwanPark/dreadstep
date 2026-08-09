//! Deterministic headless camera-anchor behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationCamera, PresentationFocus, PresentationInput, PresentationPlugin,
  PresentationRuntime, PresentationState, SceneActor, SceneCamera, SceneGroundItem,
  SceneInventoryItem, SceneTile,
};
use dreadstep_content::starter_item_floor;
use dreadstep_core::{ActorId, ItemId, Position};

type TileProjection = BTreeMap<(i32, i32), (Entity, SceneTile)>;
type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;
type GroundProjection = BTreeMap<ItemId, (Entity, SceneGroundItem)>;
type InventoryProjection = BTreeMap<ItemId, (Entity, SceneInventoryItem)>;

fn camera_app(actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationFocus::new(actor));
  app.insert_resource(PresentationCamera::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn item_camera_app(actor: ActorId) -> App {
  let mut world = starter_item_floor().expect("item content should validate");
  world
    .drop_item(ActorId::new(1), ItemId::new(101))
    .expect("starter item should be owned");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationFocus::new(actor));
  app.insert_resource(PresentationCamera::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn scene_projection(app: &mut App) -> (TileProjection, ActorProjection) {
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
  (tiles, actors)
}

fn complete_scene_projection(
  app: &mut App,
) -> (
  TileProjection,
  ActorProjection,
  GroundProjection,
  InventoryProjection,
) {
  let (tiles, actors) = scene_projection(app);
  let world = app.world_mut();
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

fn camera_entities(app: &mut App) -> Vec<(Entity, Position)> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneCamera)>();
  let mut entities = query
    .iter(world)
    .map(|(entity, camera)| (entity, camera.center()))
    .collect::<Vec<_>>();
  entities.sort_unstable_by_key(|(entity, _)| *entity);
  entities
}

fn assert_camera(app: &mut App, actor: ActorId, center: Option<Position>) {
  let camera = *app.world().resource::<PresentationCamera>();
  assert_eq!(camera.actor(), actor);
  assert_eq!(camera.center(), center);
}

#[test]
fn startup_projects_camera_center_and_one_scene_entity() {
  let mut app = camera_app(ActorId::new(1));

  app.update();

  assert_camera(&mut app, ActorId::new(1), Some(Position::new(1, 1)));
  assert_eq!(camera_entities(&mut app).len(), 1);
  assert_eq!(camera_entities(&mut app)[0].1, Position::new(1, 1));
}

#[test]
fn accepted_move_updates_camera_without_replacing_scene_entity() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before = camera_entities(&mut app)[0].0;
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_camera(&mut app, ActorId::new(1), Some(Position::new(2, 1)));
  assert_eq!(
    camera_entities(&mut app),
    vec![(before, Position::new(2, 1))]
  );
}

#[test]
fn changing_controlled_actor_updates_camera_identity_and_center() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before = camera_entities(&mut app)[0].0;
  app.insert_resource(PresentationInput::new(ActorId::new(2)));

  app.update();

  assert_camera(&mut app, ActorId::new(2), Some(Position::new(5, 1)));
  assert_eq!(
    camera_entities(&mut app),
    vec![(before, Position::new(5, 1))]
  );
}

#[test]
fn unknown_actor_clears_camera_without_mutating_runtime_or_complete_scene() {
  let mut app = item_camera_app(ActorId::new(1));
  app.update();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_replay = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = complete_scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert_camera(&mut app, ActorId::new(99), None);
  assert!(camera_entities(&mut app).is_empty());
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_replay);
  assert_eq!(complete_scene_projection(&mut app), before_scene);
}

#[test]
fn duplicate_scene_cameras_are_deduplicated_by_entity_identity() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let stable = camera_entities(&mut app)[0].0;
  let duplicate = app
    .world_mut()
    .spawn(SceneCamera::new(Position::new(9, 9)))
    .id();
  assert_ne!(stable, duplicate);

  app.update();

  assert_eq!(
    camera_entities(&mut app),
    vec![(stable, Position::new(1, 1))]
  );
}

#[test]
fn recycled_lower_entity_index_does_not_replace_the_retained_camera() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let stable = camera_entities(&mut app)[0].0;
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
  let duplicate = app
    .world_mut()
    .spawn(SceneCamera::new(Position::new(9, 9)))
    .id();
  assert_ne!(stable, duplicate);

  app.update();

  assert_eq!(
    camera_entities(&mut app),
    vec![(stable, Position::new(1, 1))]
  );
}

#[test]
fn missing_camera_resource_is_a_safe_noop() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before_scene = camera_entities(&mut app);
  app.world_mut().remove_resource::<PresentationCamera>();

  app.update();

  assert_eq!(camera_entities(&mut app), before_scene);
  let world = app.world_mut();
  assert_eq!(world.query::<&SceneTile>().iter(world).count(), 35);
  assert_eq!(world.query::<&SceneActor>().iter(world).count(), 4);
}

#[test]
fn missing_runtime_preserves_camera_projection() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before = *app.world().resource::<PresentationCamera>();
  let before_scene = camera_entities(&mut app);
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert_eq!(*app.world().resource::<PresentationCamera>(), before);
  assert_eq!(camera_entities(&mut app), before_scene);
}

#[test]
fn missing_input_preserves_camera_projection() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before = *app.world().resource::<PresentationCamera>();
  let before_scene = camera_entities(&mut app);
  app.world_mut().remove_resource::<PresentationInput>();

  app.update();

  assert_eq!(*app.world().resource::<PresentationCamera>(), before);
  assert_eq!(camera_entities(&mut app), before_scene);
}
