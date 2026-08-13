//! Schedule-overflow tests that set private ready times near `u64::MAX`.

use crate::{
  ActionTime, Actor, ActorId, ActorKind, Command, CommandError, GridMap, Position, Tile, WorldState,
};

fn floor_map(width: u32, height: u32) -> GridMap {
  GridMap::filled(width, height, Tile::Floor).expect("test map should be valid")
}

#[test]
fn ranged_cost_overflow_is_filtered_and_rejected_atomically() {
  let mut world = WorldState::new(
    floor_map(7, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");
  let near_max = ActionTime::new(u64::MAX - 1);
  world.current_time = near_max;
  world
    .actors
    .get_mut(&ActorId::new(1))
    .expect("attacker exists")
    .ready_at = near_max;
  world
    .actors
    .get_mut(&ActorId::new(2))
    .expect("target exists")
    .ready_at = ActionTime::new(u64::MAX);

  assert!(world.legal_commands().contains(&Command::Wait {
    actor: ActorId::new(1),
  }));
  assert!(!world.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));

  let result = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("standard cost should reach the maximum timeline value");
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("attacker exists")
      .ready_at(),
    ActionTime::new(u64::MAX)
  );
  assert_eq!(result.current_time(), ActionTime::new(u64::MAX));

  let before = world.clone();
  assert_eq!(
    world.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::ScheduleOverflow(ActorId::new(1)))
  );
  assert_eq!(world, before);
}

#[test]
fn enemy_ranged_cost_overflow_is_filtered_while_standard_actions_remain_legal() {
  let mut world = WorldState::new(
    floor_map(5, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");
  let near_max = ActionTime::new(u64::MAX - 1);
  world.current_time = near_max;
  world
    .actors
    .get_mut(&ActorId::new(1))
    .expect("enemy exists")
    .ready_at = near_max;
  world
    .actors
    .get_mut(&ActorId::new(2))
    .expect("target exists")
    .ready_at = ActionTime::new(u64::MAX);

  assert!(world.legal_commands().contains(&Command::Wait {
    actor: ActorId::new(1),
  }));
  assert!(!world.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
  let before = world.clone();
  assert_eq!(
    world.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::ScheduleOverflow(ActorId::new(1)))
  );
  assert_eq!(world, before);
}
