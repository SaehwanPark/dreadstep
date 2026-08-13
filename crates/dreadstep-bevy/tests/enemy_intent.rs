//! Deterministic presentation projection of the scheduled enemy's next core command.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationEnemyIntent, PresentationInput, PresentationPlugin, PresentationRuntime,
  PresentationState,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, EnemyBehavior, GridMap, Position, Tile, WorldState,
};

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
fn scheduled_adjacent_enemy_intent_prefers_attack_before_chase() {
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(1, 0)),
    ],
  )
  .expect("adjacent enemy world validates");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  app.update();

  let intent = app.world().resource::<PresentationEnemyIntent>();
  assert_eq!(intent.actor(), Some(ActorId::new(1)));
  assert_eq!(
    intent.command(),
    Some(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
}

#[test]
fn scheduled_brute_intent_prefers_breaking_its_chase_step() {
  let world = WorldState::new(
    GridMap::from_tiles(
      5,
      1,
      vec![
        Tile::Floor,
        Tile::Breakable,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
      ],
    )
    .expect("brute map should validate"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(2, 0), EnemyBehavior::Brute),
    ],
  )
  .expect("brute world should validate");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to brute");
  app.update();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();

  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::Break {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
  );
  assert_eq!(
    app.world().resource::<PresentationRuntime>().snapshot(),
    before_snapshot
  );
}

#[test]
fn scheduled_adjacent_kiter_intent_prefers_core_retreat() {
  let world = WorldState::new(
    GridMap::filled(4, 3, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_enemy_behavior(ActorId::new(1), Position::new(1, 1), EnemyBehavior::Kiter),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 1)),
    ],
  )
  .expect("adjacent kiter world validates");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_replay = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();

  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::Retreat {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_replay);
}

#[test]
fn scheduled_distant_enemy_intent_prefers_ranged_attack_before_chase() {
  let world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("distant enemy world validates");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_replay = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();

  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_replay);
}

#[test]
fn scheduled_frostcaster_intent_prefers_cast_chill_at_clear_range() {
  let world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_enemy_behavior(
        ActorId::new(1),
        Position::new(0, 0),
        EnemyBehavior::Frostcaster,
      ),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("frostcaster world validates");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(ActorId::new(2)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_replay = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();

  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::CastChill {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_replay);
}

#[test]
fn scheduled_enemy_intent_investigates_a_kick_noise_before_chase() {
  let world = WorldState::new(
    GridMap::from_tiles(
      8,
      3,
      vec![
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Door,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
      ],
    )
    .expect("noise map validates"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(5, 1)),
    ],
  )
  .expect("noise world validates");
  let mut runtime = PresentationRuntime::new(PresentationState::new(7, world));
  runtime
    .execute(Command::Kick {
      actor: ActorId::new(1),
      position: Position::new(2, 1),
    })
    .expect("kick should be accepted");
  let mut app = App::new();
  app.insert_resource(runtime);
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(PresentationEnemyIntent::new());
  app.add_plugins(PresentationPlugin);
  app.update();

  assert_eq!(
    app.world().resource::<PresentationEnemyIntent>().command(),
    Some(Command::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
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
