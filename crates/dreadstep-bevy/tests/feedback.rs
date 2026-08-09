//! Deterministic presentation feedback behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationInput, PresentationPlugin, PresentationRuntime, PresentationState, SceneActor,
  SceneTile,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, Event, GridMap, Position, Tile, WorldState,
};

type TileProjection = BTreeMap<(i32, i32), (Entity, SceneTile)>;
type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;

fn input_app() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn single_actor_runtime() -> PresentationRuntime {
  let map = GridMap::filled(3, 1, Tile::Floor).expect("map should validate");
  let world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  PresentationRuntime::new(PresentationState::new(7, world))
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

#[test]
fn fresh_runtime_has_no_feedback() {
  let runtime = PresentationRuntime::start_run(7).expect("content should validate");
  assert!(runtime.output().is_none());
}

#[test]
fn direct_command_publishes_and_consumes_one_output() {
  let mut runtime = PresentationRuntime::start_run(7).expect("content should validate");
  let output = runtime
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("player should move");

  assert_eq!(runtime.output(), Some(&output));
  let taken = runtime.take_output().expect("output should be pending");
  assert_eq!(taken, output);
  assert!(runtime.take_output().is_none());
}

#[test]
fn later_accepted_command_replaces_consumed_output() {
  let mut runtime = single_actor_runtime();
  let first = runtime
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("first move should succeed");
  assert_eq!(runtime.take_output(), Some(first));

  let second = runtime
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::West,
    })
    .expect("second move should succeed");

  assert_eq!(runtime.output(), Some(&second));
  assert_eq!(
    second.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(1, 0),
      to: Position::new(0, 0),
    }]
  );
}

#[test]
fn keyboard_command_publishes_exact_event_and_snapshot_evidence() {
  let mut app = input_app();
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();

  let mut runtime = app.world_mut().resource_mut::<PresentationRuntime>();
  let output = runtime
    .take_output()
    .expect("keyboard output should be pending");
  assert_eq!(
    output.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(1, 1),
      to: Position::new(2, 1),
    }]
  );
  let current_snapshot = runtime.snapshot();
  assert_eq!(output.snapshot(), &current_snapshot);
  assert_eq!(
    output.snapshot().actors()[0].position(),
    Position::new(2, 1)
  );
  assert!(runtime.output().is_none());
}

#[test]
fn rejected_command_clears_stale_feedback_without_mutating_core() {
  let mut runtime = PresentationRuntime::start_run(7).expect("content should validate");
  runtime
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("player should move");
  assert!(runtime.output().is_some());
  let before = (runtime.snapshot(), runtime.replay_digest());

  runtime
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect_err("player should be unscheduled after moving");

  assert!(runtime.output().is_none());
  assert_eq!((runtime.snapshot(), runtime.replay_digest()), before);
}

#[test]
fn rejected_keyboard_command_clears_feedback_and_preserves_complete_scene() {
  let mut app = input_app();
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  assert!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .output()
      .is_some()
  );
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = scene_projection(&mut app);

  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .release(KeyCode::ArrowRight);
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();

  let runtime = app.world().resource::<PresentationRuntime>();
  assert!(runtime.output().is_none());
  assert_eq!(runtime.snapshot(), before);
  assert_eq!(runtime.replay_digest(), before_digest);
  assert_eq!(scene_projection(&mut app), before_scene);
}
