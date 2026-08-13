//! Deterministic Brute break behavior.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, EnemyBehavior, Event, GridMap, Position, Tile, WorldState,
};

fn brute_world() -> WorldState {
  let map = GridMap::from_tiles(
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
  .expect("brute map should validate");
  WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(2, 0), EnemyBehavior::Brute),
    ],
  )
  .expect("brute world should validate")
}

#[test]
fn scheduled_brute_breaks_the_next_blocking_breakable() {
  let mut world = brute_world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to brute");
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));
  assert!(world.legal_commands().contains(&Command::Break {
    actor: ActorId::new(2),
    position: Position::new(1, 0),
  }));
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Break {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
  );
  let result = world
    .execute(Command::Break {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
    .expect("brute should break the blocking terrain");
  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Floor));
  assert_eq!(
    result.events(),
    &[Event::BreakableBroken {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    }]
  );
}

#[test]
fn brute_does_not_break_off_axis_or_non_next_breakables() {
  let map = GridMap::from_tiles(
    3,
    3,
    vec![
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Breakable,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
    ],
  )
  .expect("brute map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(2, 0), EnemyBehavior::Brute),
    ],
  )
  .expect("brute world should validate");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to brute");
  assert!(
    world
      .legal_commands()
      .iter()
      .all(|command| !matches!(command, Command::Break { .. }))
  );
}

#[test]
fn brute_preference_is_read_only_and_returns_to_chase_after_breaking() {
  let mut world = brute_world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to brute");
  let before_world = world.clone();
  let before_digest = world.digest();
  let before_commands = world.legal_commands();
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Break {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
  );
  assert_eq!(world, before_world);
  assert_eq!(world.digest(), before_digest);
  assert_eq!(world.legal_commands(), before_commands);
  world
    .execute(Command::Break {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
    .expect("brute should break before chasing");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield again");
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::RangedAttack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}

#[test]
fn enemy_preference_keeps_wait_before_generic_move_fallback() {
  let mut world = brute_world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to brute");
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(99)),
    Some(Command::Wait {
      actor: ActorId::new(2),
    })
  );
}
