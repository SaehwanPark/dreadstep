//! Contract tests for headless ECS Window configuration attachment.

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::window::{Window, WindowMode, WindowPosition};
use dreadstep_bevy::{PresentationPlugin, PresentationRuntime, PresentationWindow, SceneWindow};

fn window_app(request: PresentationWindow) -> App {
  let mut app = App::new();
  app.insert_resource(request);
  app.add_plugins(PresentationPlugin);
  app
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WindowEvidence {
  entity: Entity,
  scene: SceneWindow,
  physical_width: u32,
  physical_height: u32,
  scale_bits: u32,
  logical_width_bits: u32,
  logical_height_bits: u32,
}

fn window_projection(app: &mut App) -> Vec<WindowEvidence> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneWindow, &Window)>();
  let mut entries = query
    .iter(world)
    .map(|(entity, scene, window)| WindowEvidence {
      entity,
      scene: *scene,
      physical_width: window.resolution.physical_width(),
      physical_height: window.resolution.physical_height(),
      scale_bits: window.resolution.scale_factor().to_bits(),
      logical_width_bits: window.resolution.width().to_bits(),
      logical_height_bits: window.resolution.height().to_bits(),
    })
    .collect::<Vec<_>>();
  entries.sort_unstable_by_key(|entry| entry.entity);
  entries
}

#[test]
fn startup_without_runtime_attaches_one_window_with_checked_resolution_and_scale() {
  let request = PresentationWindow::new(320, 240, 2).expect("request should validate");
  let mut app = window_app(request);
  app.update();

  assert!(app.world().get_resource::<PresentationRuntime>().is_none());
  let entries = window_projection(&mut app);
  assert_eq!(entries.len(), 1);
  let entry = entries[0];
  assert_eq!(entry.scene.logical_width(), 320);
  assert_eq!(entry.scene.logical_height(), 240);
  assert_eq!(entry.scene.pixel_scale(), 2);
  assert_eq!(entry.scene.physical_width(), 640);
  assert_eq!(entry.scene.physical_height(), 480);
  assert_eq!(entry.physical_width, 640);
  assert_eq!(entry.physical_height, 480);
  assert_eq!(entry.scale_bits, 2.0f32.to_bits());
  assert_eq!(entry.logical_width_bits, 320.0f32.to_bits());
  assert_eq!(entry.logical_height_bits, 240.0f32.to_bits());
  let window = app.world().get::<Window>(entry.entity).unwrap();
  assert!(matches!(window.mode, WindowMode::Windowed));
  assert!(matches!(window.position, WindowPosition::Automatic));
  assert!(window.resizable);
  assert!(window.decorations);
  assert!(!window.transparent);
  assert!(window.visible);
  assert_eq!(window.name, None);
}

#[allow(clippy::cast_precision_loss)]
fn projected_scale(pixel_scale: u32) -> f32 {
  pixel_scale as f32
}

#[allow(clippy::cast_precision_loss)]
fn projected_logical_width(request: PresentationWindow, scale: f32) -> f32 {
  request.physical_width() as f32 / scale
}

#[test]
fn large_valid_scale_uses_deterministic_f32_window_adapter() {
  let request = PresentationWindow::new(3, 1, 16_777_217).unwrap();
  let mut app = window_app(request);
  app.update();

  let entry = window_projection(&mut app)[0];
  let scale = projected_scale(request.pixel_scale());
  assert_eq!(entry.scene.pixel_scale(), 16_777_217);
  assert_eq!(scale.to_bits(), 16_777_216.0f32.to_bits());
  assert_eq!(entry.scale_bits, scale.to_bits());
  assert_eq!(
    entry.logical_width_bits,
    projected_logical_width(request, scale).to_bits()
  );
  assert_eq!(entry.logical_height_bits, 1.0f32.to_bits());
}

#[test]
fn changed_request_refreshes_same_window_entity_and_exact_resolution() {
  let mut app = window_app(PresentationWindow::new(320, 240, 2).unwrap());
  app.update();
  let before = window_projection(&mut app)[0].entity;

  app.insert_resource(PresentationWindow::new(800, 600, 1).unwrap());
  app.update();

  let entry = window_projection(&mut app)[0];
  assert_eq!(entry.entity, before);
  assert_eq!(
    entry.scene,
    SceneWindow::new(PresentationWindow::new(800, 600, 1).unwrap())
  );
  assert_eq!(entry.physical_width, 800);
  assert_eq!(entry.physical_height, 600);
  assert_eq!(entry.scale_bits, 1.0f32.to_bits());
  assert_eq!(entry.logical_width_bits, 800.0f32.to_bits());
  assert_eq!(entry.logical_height_bits, 600.0f32.to_bits());
}

#[test]
fn duplicate_window_cleanup_retains_one_projection_entity() {
  let request = PresentationWindow::new(320, 240, 2).unwrap();
  let mut app = window_app(request);
  let recycled = app.world_mut().spawn_empty().id();
  app.update();
  let stable = window_projection(&mut app)[0].entity;
  let duplicate = {
    let world = app.world_mut();
    let recycled = world
      .despawn_no_free(recycled)
      .expect("recycled entity should exist");
    world
      .spawn_at(recycled, (SceneWindow::new(request), Window::default()))
      .expect("recycled entity should be reusable")
      .id()
  };
  assert!(duplicate.index() < stable.index());

  app.update();

  let entries = window_projection(&mut app);
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].entity, stable);
  assert!(app.world().get_entity(duplicate).is_err());
}

#[test]
fn missing_window_request_preserves_projection_and_runtime_authority() {
  let request = PresentationWindow::new(320, 240, 2).unwrap();
  let mut app = window_app(request);
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.update();
  let before = window_projection(&mut app);
  let snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.world_mut().remove_resource::<PresentationWindow>();

  app.update();

  assert_eq!(window_projection(&mut app), before);
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), snapshot);
  assert_eq!(runtime.replay_digest(), digest);
}
