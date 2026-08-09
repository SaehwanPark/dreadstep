//! Contract tests for headless ECS Camera2d attachment.

use bevy::app::App;
use bevy::camera::visibility::Visibility;
use bevy::camera::{Camera, Camera2d};
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationCamera, PresentationInput, PresentationPlugin, PresentationRuntime, SceneCamera,
};
use dreadstep_core::{ActorId, Position};

fn camera_app(actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationCamera::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

type CameraProjection = (
  Entity,
  Position,
  bool,
  bool,
  Option<Transform>,
  Option<Visibility>,
);

fn camera_projection(app: &mut App) -> Vec<CameraProjection> {
  let world = app.world_mut();
  let mut query = world.query::<(
    Entity,
    &SceneCamera,
    Option<&Camera2d>,
    Option<&Camera>,
    Option<&Transform>,
    Option<&Visibility>,
  )>();
  let mut entries = query
    .iter(world)
    .map(
      |(entity, scene, camera_2d, camera, transform, visibility)| {
        (
          entity,
          scene.center(),
          camera_2d.is_some(),
          camera.is_some(),
          transform.copied(),
          visibility.copied(),
        )
      },
    )
    .collect::<Vec<_>>();
  entries.sort_unstable_by_key(|entry| entry.0);
  entries
}

fn camera2d_entities(app: &mut App) -> Vec<Entity> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &Camera2d)>();
  let mut entities = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  entities.sort_unstable();
  entities
}

#[test]
fn startup_attaches_camera2d_and_required_defaults_to_one_scene_camera() {
  let mut app = camera_app(ActorId::new(1));
  app.update();

  let entries = camera_projection(&mut app);
  assert_eq!(entries.len(), 1);
  let (_, center, has_camera_2d, has_camera, transform, visibility) = entries[0];
  assert_eq!(center, Position::new(1, 1));
  assert!(has_camera_2d);
  assert!(has_camera);
  assert_eq!(transform, Some(Transform::default()));
  assert!(visibility.is_some());
  assert_eq!(camera2d_entities(&mut app), vec![entries[0].0]);
}

#[test]
fn movement_and_actor_selection_retain_camera2d_entity() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let before = camera_projection(&mut app)[0].0;
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  assert_eq!(camera_projection(&mut app)[0].0, before);
  assert!(camera_projection(&mut app)[0].2);
  assert_eq!(camera_projection(&mut app)[0].1, Position::new(2, 1));

  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.update();
  assert_eq!(camera_projection(&mut app)[0].0, before);
  assert!(camera_projection(&mut app)[0].2);
  assert_eq!(camera_projection(&mut app)[0].1, Position::new(5, 1));
}

#[test]
fn duplicate_scene_camera_cleanup_retains_camera2d_on_stable_entity() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let stable = camera_projection(&mut app)[0].0;
  app.world_mut().spawn(SceneCamera::new(Position::new(9, 9)));
  app.update();
  let entries = camera_projection(&mut app);
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].0, stable);
  assert!(entries[0].2);
}

#[test]
fn recycled_lower_entity_index_does_not_replace_camera2d_marker() {
  let mut app = camera_app(ActorId::new(1));
  let recycled = app.world_mut().spawn_empty().id();
  app.update();
  let stable = camera_projection(&mut app)[0].0;
  let duplicate = {
    let world = app.world_mut();
    let recycled = world
      .despawn_no_free(recycled)
      .expect("recycled entity should exist");
    world
      .spawn_at(recycled, SceneCamera::new(Position::new(9, 9)))
      .expect("recycled entity should be reusable")
      .id()
  };
  assert!(
    duplicate.index() < stable.index(),
    "expected recycled lower index, recycled={recycled:?}, duplicate={duplicate:?}, stable={stable:?}"
  );

  app.update();

  assert_eq!(camera_projection(&mut app).len(), 1);
  assert_eq!(camera_projection(&mut app)[0].0, stable);
  assert_eq!(camera2d_entities(&mut app), vec![stable]);
}

#[test]
fn unknown_actor_removes_camera2d_without_mutating_runtime() {
  let mut app = camera_app(ActorId::new(1));
  app.update();
  let snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.insert_resource(PresentationInput::new(ActorId::new(99)));
  app.update();
  assert!(camera_projection(&mut app).is_empty());
  assert!(camera2d_entities(&mut app).is_empty());
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), snapshot);
  assert_eq!(runtime.replay_digest(), digest);
}

#[test]
fn missing_authority_and_camera_resources_preserve_existing_camera_components() {
  let mut missing_runtime = camera_app(ActorId::new(1));
  missing_runtime.update();
  let before = camera_projection(&mut missing_runtime);
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(camera_projection(&mut missing_runtime), before);

  let mut missing_input = camera_app(ActorId::new(1));
  missing_input.update();
  let before = camera_projection(&mut missing_input);
  missing_input
    .world_mut()
    .remove_resource::<PresentationInput>();
  missing_input.update();
  assert_eq!(camera_projection(&mut missing_input), before);

  let mut missing_camera = camera_app(ActorId::new(1));
  missing_camera.update();
  let before = camera_projection(&mut missing_camera);
  missing_camera
    .world_mut()
    .remove_resource::<PresentationCamera>();
  missing_camera.update();
  assert_eq!(camera_projection(&mut missing_camera), before);
}
