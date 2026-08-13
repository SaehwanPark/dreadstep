//! Deterministic stationary Blocker behavior.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, EnemyBehavior, GridMap, HitPoints, Position, ReplayTrace,
  Tile, WorldState,
};

fn world(enemy: Position, player: Position) -> WorldState {
  let width = enemy.x().max(player.x()).cast_unsigned() + 1;
  WorldState::new(
    GridMap::filled(width, 1, Tile::Floor).expect("blocker map should validate"),
    vec![
      Actor::with_enemy_behavior(ActorId::new(2), enemy, EnemyBehavior::Blocker),
      Actor::new(ActorId::new(1), ActorKind::Player, player),
    ],
  )
  .expect("blocker world should validate")
}

fn schedule_blocker(world: &mut WorldState) {
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to blocker");
}

#[test]
fn distant_blocker_waits_instead_of_chasing() {
  let mut world = world(Position::new(2, 0), Position::new(0, 0));
  schedule_blocker(&mut world);
  let before = world.clone();
  let before_digest = world.digest();
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Wait {
      actor: ActorId::new(2),
    })
  );
  assert_eq!(world, before);
  assert_eq!(world.digest(), before_digest);
  assert!(world.legal_commands().contains(&Command::Chase {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  }));
  assert!(world.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  }));
  let before_position = world
    .actor(ActorId::new(2))
    .expect("blocker exists")
    .position();
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("blocker wait should execute through the existing command path");
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("blocker exists")
      .position(),
    before_position
  );
}

#[test]
fn blocker_identity_changes_state_digest_but_not_existing_wait_replay_hashing() {
  let blocker = world(Position::new(2, 0), Position::new(0, 0));
  let pursuer = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("pursuer map should validate"),
    vec![
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(2, 0), EnemyBehavior::Pursuer),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
    ],
  )
  .expect("pursuer world should validate");
  assert_ne!(blocker.digest(), pursuer.digest());

  let blocker_wait = Command::Wait {
    actor: ActorId::new(2),
  };
  let mut first = ReplayTrace::new(7);
  first.record(blocker_wait);
  let mut equivalent = ReplayTrace::new(7);
  equivalent.record(blocker_wait);
  assert_eq!(first.digest(), equivalent.digest());
}

#[test]
fn blocker_preference_rejects_invalid_actors_and_waits_for_invalid_targets() {
  let mut world = world(Position::new(2, 0), Position::new(0, 0));
  schedule_blocker(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(99)),
    Some(Command::Wait {
      actor: ActorId::new(2),
    })
  );
  world
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("player target should be mutable in the tester fixture");
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Wait {
      actor: ActorId::new(2),
    })
  );
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(99), ActorId::new(1)),
    None
  );
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("blocker should be mutable in the tester fixture");
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    None
  );
}

#[test]
fn adjacent_blocker_attacks_and_does_not_move() {
  let mut world = world(Position::new(1, 0), Position::new(0, 0));
  schedule_blocker(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  let before_position = world
    .actor(ActorId::new(2))
    .expect("blocker exists")
    .position();
  world
    .execute(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("adjacent blocker should attack");
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("blocker exists")
      .position(),
    before_position
  );
}
