//! Deterministic headless event-message evidence behavior.

use bevy::app::App;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationInput, PresentationMessage, PresentationMessages, PresentationPlugin,
  PresentationRuntime, PresentationState,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, BlockReason, Command, Damage, EnemyBehavior, GridMap, HitPoints,
  Position, Tile, WorldState,
};

fn message_app(runtime: PresentationRuntime, actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(runtime);
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

#[test]
fn frostcaster_cast_maps_to_ordered_typed_messages() {
  let runtime = custom_runtime(
    3,
    vec![Tile::Floor; 3],
    vec![
      Actor::with_enemy_behavior(
        ActorId::new(1),
        Position::new(0, 0),
        EnemyBehavior::Frostcaster,
      ),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  );
  let mut app = message_app(runtime, ActorId::new(2));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::CastChill {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("clear ranged frostcaster cast should succeed");
  app.update();

  assert_eq!(
    messages(&app),
    vec![
      PresentationMessage::ChillCast {
        caster: ActorId::new(1),
        target: ActorId::new(2),
      },
      PresentationMessage::StatusApplied {
        actor: ActorId::new(2),
        status: dreadstep_core::StatusKind::Chilled,
        remaining_actions: 2,
      },
    ]
  );
}

fn custom_runtime(width: u32, tiles: Vec<Tile>, actors: Vec<Actor>) -> PresentationRuntime {
  let map = GridMap::from_tiles(width, 1, tiles).expect("map should validate");
  let world = WorldState::new(map, actors).expect("world should validate");
  PresentationRuntime::new(PresentationState::new(7, world))
}

fn messages(app: &App) -> Vec<PresentationMessage> {
  app
    .world()
    .resource::<PresentationMessages>()
    .messages()
    .to_vec()
}

#[test]
fn fresh_runtime_has_no_messages() {
  let mut app = message_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();

  assert!(messages(&app).is_empty());
}

#[test]
fn accepted_keyboard_move_maps_to_one_ordered_typed_message() {
  let mut app = message_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    messages(&app),
    vec![PresentationMessage::Moved {
      actor: ActorId::new(1),
      from: Position::new(1, 1),
      to: Position::new(2, 1),
    }]
  );
  assert!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .output()
      .is_some()
  );
}

#[test]
fn blocked_movement_maps_reason_and_positions() {
  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Wall],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    messages(&app),
    vec![PresentationMessage::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Terrain,
    }]
  );
}

#[test]
fn waiting_maps_action_time() {
  let runtime = custom_runtime(
    1,
    vec![Tile::Floor],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::Space);

  app.update();

  assert_eq!(
    messages(&app),
    vec![PresentationMessage::Waited {
      actor: ActorId::new(1),
      at: dreadstep_core::ActionTime::new(0),
    }]
  );
}

#[test]
fn attack_and_death_preserve_event_order_and_payloads() {
  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Floor],
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");

  app.update();

  assert_eq!(
    messages(&app),
    vec![
      PresentationMessage::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::MELEE,
        remaining_hit_points: HitPoints::new(0),
      },
      PresentationMessage::Died {
        actor: ActorId::new(2),
      },
    ]
  );
}

#[test]
fn trap_trigger_preserves_movement_damage_and_consumption_order() {
  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Trap],
    vec![Actor::with_hit_points(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
      HitPoints::new(3),
    )],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .expect("trap movement should succeed");
  app.update();

  assert_eq!(
    messages(&app),
    vec![
      PresentationMessage::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      },
      PresentationMessage::TrapTriggered {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        damage: Damage::TRAP,
        remaining_hit_points: HitPoints::new(2),
      },
    ]
  );
}

#[test]
fn breakable_terrain_maps_to_a_typed_message() {
  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Breakable],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Break {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("breakable terrain should break");
  app.update();

  assert_eq!(
    messages(&app),
    vec![PresentationMessage::BreakableBroken {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
}

#[test]
fn kick_noise_maps_to_a_typed_message() {
  let runtime = custom_runtime(
    3,
    vec![Tile::Floor, Tile::Door, Tile::Floor],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut app = message_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("closed door should be kickable");
  app.update();

  assert_eq!(
    messages(&app),
    vec![
      PresentationMessage::DoorOpened {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      PresentationMessage::NoiseCreated {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        radius: 3,
      },
    ]
  );
}

#[test]
fn rejected_command_clears_stale_messages_without_mutating_core() {
  let mut app = message_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  assert!(!messages(&app).is_empty());
  let before = {
    let runtime = app.world().resource::<PresentationRuntime>();
    (runtime.snapshot(), runtime.replay_digest())
  };
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .release(KeyCode::ArrowRight);
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert!(messages(&app).is_empty());
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!((runtime.snapshot(), runtime.replay_digest()), before);
}

#[test]
fn missing_runtime_preserves_existing_messages() {
  let mut app = message_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  let before = messages(&app);
  app.world_mut().remove_resource::<PresentationRuntime>();

  app.update();

  assert_eq!(messages(&app), before);
}

#[test]
fn missing_message_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);

  app.update();

  assert!(app.world().get_resource::<PresentationMessages>().is_none());
}
