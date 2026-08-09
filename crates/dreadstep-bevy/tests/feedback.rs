//! Deterministic presentation feedback behavior.

use bevy::app::App;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{PresentationInput, PresentationPlugin, PresentationRuntime};
use dreadstep_core::{ActorId, Command, Direction, Event, Position};

fn input_app() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
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
  assert_eq!(output.snapshot().digest(), runtime.snapshot().digest());
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
