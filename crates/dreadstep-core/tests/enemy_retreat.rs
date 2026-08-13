//! Contract tests for the authored kiter retreat policy.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, EnemyBehavior, Event, GridMap, HitPoints,
  Position, Tile, WorldState,
};

fn world_with_kiter(map: GridMap, kiter: Position, player: Position) -> WorldState {
  WorldState::new(
    map,
    vec![
      Actor::with_enemy_behavior(ActorId::new(2), kiter, EnemyBehavior::Kiter),
      Actor::new(ActorId::new(1), ActorKind::Player, player),
    ],
  )
  .expect("test world should be valid")
}

fn floor_map(width: u32, height: u32) -> GridMap {
  GridMap::filled(width, height, Tile::Floor).expect("test map should be valid")
}

fn schedule_kiter(world: &mut WorldState) {
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should be scheduled first");
}

#[test]
fn kiter_legal_commands_put_retreat_before_combat_commands() {
  let mut world = world_with_kiter(floor_map(4, 3), Position::new(1, 1), Position::new(2, 1));
  schedule_kiter(&mut world);
  let commands = world.legal_commands();

  let retreat = commands
    .iter()
    .position(|command| matches!(command, Command::Retreat { .. }))
    .expect("adjacent kiter should advertise retreat");
  let attack = commands
    .iter()
    .position(|command| matches!(command, Command::Attack { .. }))
    .expect("adjacent kiter should retain attack fallback");
  assert!(retreat < attack);
  assert_eq!(
    commands[retreat],
    Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }
  );
}

#[test]
fn retreat_chooses_the_farthest_cardinal_tile_with_north_tie_break() {
  let mut world = world_with_kiter(floor_map(5, 4), Position::new(2, 1), Position::new(3, 1));
  schedule_kiter(&mut world);
  let result = world
    .execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("kiter should retreat from an adjacent player");

  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(2),
      from: Position::new(2, 1),
      to: Position::new(2, 0),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("kiter exists")
      .position(),
    Position::new(2, 0)
  );
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("kiter exists")
      .ready_at()
      .value(),
    1
  );
}

#[test]
fn retreat_rejects_non_kiter_non_adjacent_dead_and_unknown_targets_atomically() {
  let mut non_kiter = WorldState::new(
    floor_map(4, 1),
    vec![
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");
  schedule_kiter(&mut non_kiter);
  let before = non_kiter.clone();
  assert_eq!(
    non_kiter.execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::RetreatRequiresKiter(ActorId::new(2)))
  );
  assert_eq!(non_kiter, before);

  let mut non_adjacent =
    world_with_kiter(floor_map(5, 1), Position::new(1, 0), Position::new(3, 0));
  schedule_kiter(&mut non_adjacent);
  let before = non_adjacent.clone();
  assert_eq!(
    non_adjacent.execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::RetreatTargetNotAdjacent {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  assert_eq!(non_adjacent, before);

  let mut unknown = world_with_kiter(floor_map(3, 1), Position::new(1, 0), Position::new(2, 0));
  schedule_kiter(&mut unknown);
  let before = unknown.clone();
  assert_eq!(
    unknown.execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(99),
    }),
    Err(CommandError::UnknownTarget(ActorId::new(99)))
  );
  assert_eq!(unknown, before);

  let mut dead = world_with_kiter(floor_map(3, 1), Position::new(1, 0), Position::new(2, 0));
  schedule_kiter(&mut dead);
  dead
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("tester can kill target");
  let before = dead.clone();
  assert_eq!(
    dead.execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::TargetDead(ActorId::new(1)))
  );
  assert_eq!(dead, before);
}

#[test]
fn retreat_rejects_when_every_distance_increasing_tile_is_blocked() {
  let map = GridMap::from_tiles(
    3,
    3,
    vec![
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
  )
  .expect("test map should be valid");
  let mut world = world_with_kiter(map, Position::new(1, 1), Position::new(2, 1));
  schedule_kiter(&mut world);
  let before = world.clone();

  assert!(
    !world
      .legal_commands()
      .iter()
      .any(|command| matches!(command, Command::Retreat { .. }))
  );

  assert_eq!(
    world.execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::RetreatNoEscape(ActorId::new(2)))
  );
  assert_eq!(world, before);
}

#[test]
fn retreat_digest_and_replay_command_identity_distinguish_behavior() {
  let kiter = world_with_kiter(floor_map(3, 1), Position::new(1, 0), Position::new(2, 0));
  let pursuer = WorldState::new(
    floor_map(3, 1),
    vec![
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");

  assert_ne!(kiter.digest(), pursuer.digest());
  assert_eq!(EnemyBehavior::default(), EnemyBehavior::Pursuer);
}
