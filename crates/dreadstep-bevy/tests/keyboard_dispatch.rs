//! Deterministic headless keyboard dispatch behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{PresentationInput, PresentationPlugin, PresentationRuntime, SceneActor};
use dreadstep_core::{ActorId, Position};

type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;

fn input_app(actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn actor_projection(app: &mut App) -> ActorProjection {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneActor)>();
  query
    .iter(world)
    .map(|(entity, actor)| (actor.id(), (entity, *actor)))
    .collect()
}

#[test]
fn accepted_key_dispatches_through_runtime_and_syncs_scene() {
  let mut app = input_app(ActorId::new(1));
  app.update();

  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();

  let runtime = app.world().resource::<PresentationRuntime>();
  let snapshot = runtime.snapshot();
  let player = snapshot
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(1))
    .expect("player should remain visible");
  assert_eq!(player.position(), Position::new(2, 1));
  assert!(
    !app
      .world()
      .resource::<ButtonInput<KeyCode>>()
      .just_pressed(KeyCode::ArrowRight)
  );
  let scene_player = actor_projection(&mut app)
    .remove(&ActorId::new(1))
    .expect("player scene actor should exist");
  assert_eq!(scene_player.1.position(), Position::new(2, 1));
}

#[test]
fn simultaneous_keys_use_fixed_priority_and_are_consumed_once() {
  let mut app = input_app(ActorId::new(1));
  app.update();

  let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
  input.press(KeyCode::ArrowRight);
  input.press(KeyCode::KeyW);
  app.update();

  let first_position = app
    .world()
    .resource::<PresentationRuntime>()
    .snapshot()
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(1))
    .expect("player should remain visible")
    .position();
  assert_eq!(first_position, Position::new(2, 1));
  assert!(
    !app
      .world()
      .resource::<ButtonInput<KeyCode>>()
      .just_pressed(KeyCode::ArrowRight)
  );
  assert!(
    !app
      .world()
      .resource::<ButtonInput<KeyCode>>()
      .just_pressed(KeyCode::KeyW)
  );

  app.update();
  assert_eq!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .snapshot()
      .actors()
      .iter()
      .find(|actor| actor.id() == ActorId::new(1))
      .expect("player should remain visible")
      .position(),
    first_position
  );
}

#[test]
fn wait_key_dispatches_without_moving_the_controlled_actor() {
  let mut app = input_app(ActorId::new(1));
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();

  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::Enter);
  app.update();

  let after = app.world().resource::<PresentationRuntime>().snapshot();
  let before_player = before
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(1))
    .expect("player should remain visible");
  let after_player = after
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(1))
    .expect("player should remain visible");
  assert_eq!(
    after_player.ready_at().value(),
    before_player.ready_at().value() + 1
  );
  assert_eq!(after_player.position(), before_player.position());
}

#[test]
fn missing_control_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    app.world().resource::<PresentationRuntime>().snapshot(),
    before
  );
  assert!(
    app
      .world()
      .resource::<ButtonInput<KeyCode>>()
      .just_pressed(KeyCode::ArrowRight)
  );
}

#[test]
fn rejected_key_consumes_input_without_mutating_runtime_or_scene() {
  let mut app = input_app(ActorId::new(99));
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = actor_projection(&mut app);

  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();

  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before);
  assert_eq!(runtime.replay_digest(), before_digest);
  assert_eq!(actor_projection(&mut app), before_scene);
  assert!(
    !app
      .world()
      .resource::<ButtonInput<KeyCode>>()
      .just_pressed(KeyCode::ArrowRight)
  );
}
