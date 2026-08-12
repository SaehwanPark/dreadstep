//! Deterministic presentation projection of the scheduled enemy's next core command.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationEnemyIntent, PresentationInput, PresentationPlugin, PresentationRuntime,
  PresentationState,
};
use dreadstep_core::{Actor, ActorId, ActorKind, Command, GridMap, Position, Tile, WorldState};

fn intent_app() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("starter run validates"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn no_enemy_app() -> App {
  let world = WorldState::new(
    GridMap::filled(1, 1, Tile::Floor).expect("single-cell map validates"),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("player-only world validates");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

#[test]
fn player_turn_has_no_enemy_intent_and_preserves_authority() {
  let app = intent_app();
  let intent = app.world().resource::<PresentationEnemyIntent>();
  assert_eq!(intent.actor(), None);
  assert_eq!(intent.command(), None);
  assert_eq!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .replay_digest(),
    PresentationRuntime::start_run(7)
      .expect("equivalent starter run validates")
      .replay_digest()
  );
}

#[test]
fn scheduled_enemy_intent_preserves_core_chase_command() {
  let mut app = intent_app();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should schedule the enemy");
  let after_wait_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let after_wait_replay = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();

  let intent = app.world().resource::<PresentationEnemyIntent>();
  assert_eq!(intent.actor(), Some(ActorId::new(2)));
  assert_eq!(
    intent.command(),
    Some(Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  let after_projection = app.world().resource::<PresentationRuntime>();
  assert_eq!(after_projection.snapshot(), after_wait_snapshot);
  assert_eq!(after_projection.replay_digest(), after_wait_replay);
}

#[test]
fn intent_is_empty_when_the_world_has_no_enemy() {
  let app = no_enemy_app();
  let intent = app.world().resource::<PresentationEnemyIntent>();
  assert_eq!(intent.actor(), None);
  assert_eq!(intent.command(), None);
}

#[test]
fn intent_uses_the_controlled_actor_as_the_chase_target() {
  let mut app = intent_app();
  app
    .world_mut()
    .insert_resource(PresentationInput::new(ActorId::new(3)));
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should schedule the enemy");
  app.update();
  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(3),
    })
  );
}

#[test]
fn missing_runtime_clears_enemy_intent_without_panicking() {
  let mut app = intent_app();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should schedule the enemy");
  app.update();
  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().actor(),
    Some(ActorId::new(2))
  );
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  let intent = app.world().resource::<PresentationEnemyIntent>();
  assert_eq!(intent.actor(), None);
  assert_eq!(intent.command(), None);
}
