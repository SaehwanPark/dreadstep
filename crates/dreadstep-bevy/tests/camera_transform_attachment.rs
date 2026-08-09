//! Contract tests for headless ECS camera transform attachment.

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationCamera, PresentationInput, PresentationPlugin, PresentationRuntime,
  PresentationTileSize, SceneCamera,
};
use dreadstep_core::{ActorId, Position};

fn camera_app(tile_size: Option<PresentationTileSize>) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationCamera::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  if let Some(tile_size) = tile_size {
    app.insert_resource(tile_size);
  }
  app.add_plugins(PresentationPlugin);
  app
}

type CameraTransformProjection = (Entity, Position, Transform);

fn camera_transform_projection(app: &mut App) -> Vec<CameraTransformProjection> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneCamera, &Transform)>();
  let mut entries = query
    .iter(world)
    .map(|(entity, camera, transform)| (entity, camera.center(), *transform))
    .collect::<Vec<_>>();
  entries.sort_unstable_by_key(|entry| entry.0);
  entries
}

fn tile_size() -> PresentationTileSize {
  PresentationTileSize::new(32, 24).expect("tile size should validate")
}

#[test]
fn startup_attaches_checked_centered_camera_transform() {
  let mut app = camera_app(Some(tile_size()));
  app.update();

  let projection = camera_transform_projection(&mut app);
  assert_eq!(projection.len(), 1);
  assert_eq!(projection[0].1, Position::new(1, 1));
  assert_eq!(projection[0].2, Transform::from_xyz(48.0, 36.0, 0.0));
}

#[test]
fn movement_and_actor_selection_refresh_same_camera_transform_entity() {
  let mut app = camera_app(Some(tile_size()));
  app.update();
  let before = camera_transform_projection(&mut app)[0].0;

  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  let moved = camera_transform_projection(&mut app);
  assert_eq!(moved[0].0, before);
  assert_eq!(moved[0].1, Position::new(2, 1));
  assert_eq!(moved[0].2, Transform::from_xyz(80.0, 36.0, 0.0));

  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.update();
  let selected = camera_transform_projection(&mut app);
  assert_eq!(selected[0].0, before);
  assert_eq!(selected[0].1, Position::new(5, 1));
  assert_eq!(selected[0].2, Transform::from_xyz(176.0, 36.0, 0.0));
}

#[test]
fn fresh_missing_tile_size_is_default_but_later_removal_retains_transform() {
  let mut app = camera_app(None);
  app.update();
  let projection = camera_transform_projection(&mut app);
  assert_eq!(projection.len(), 1);
  assert_eq!(projection[0].2, Transform::default());

  app.insert_resource(tile_size());
  app.update();
  let before_removal = camera_transform_projection(&mut app);
  assert_eq!(before_removal[0].2, Transform::from_xyz(48.0, 36.0, 0.0));

  app.world_mut().remove_resource::<PresentationTileSize>();
  app.update();
  assert_eq!(camera_transform_projection(&mut app), before_removal);
}

#[test]
fn missing_authority_resources_preserve_existing_camera_transform() {
  let mut missing_runtime = camera_app(Some(tile_size()));
  missing_runtime.update();
  let before = camera_transform_projection(&mut missing_runtime);
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(camera_transform_projection(&mut missing_runtime), before);

  let mut missing_input = camera_app(Some(tile_size()));
  missing_input.update();
  let before = camera_transform_projection(&mut missing_input);
  missing_input
    .world_mut()
    .remove_resource::<PresentationInput>();
  missing_input.update();
  assert_eq!(camera_transform_projection(&mut missing_input), before);

  let mut missing_camera = camera_app(Some(tile_size()));
  missing_camera.update();
  let before = camera_transform_projection(&mut missing_camera);
  missing_camera
    .world_mut()
    .remove_resource::<PresentationCamera>();
  missing_camera.update();
  assert_eq!(camera_transform_projection(&mut missing_camera), before);
}

#[test]
fn unknown_actor_clears_camera_transform_without_mutating_runtime() {
  let mut app = camera_app(Some(tile_size()));
  app.update();
  let snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();

  app.insert_resource(PresentationInput::new(ActorId::new(99)));
  app.update();

  assert!(camera_transform_projection(&mut app).is_empty());
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), snapshot);
  assert_eq!(runtime.replay_digest(), digest);
}
