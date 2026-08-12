//! Contract tests for the bounded scheduled ranged attack.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Damage, Event, GridMap, HitPoints, Position,
  ReplayTrace, Tile, WorldState,
};

fn world(target: Position, hit_points: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(7, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        target,
        HitPoints::new(hit_points),
      ),
    ],
  )
  .expect("test world should be valid")
}

#[test]
fn ranged_attack_accepts_distance_two_and_advances_one_action() {
  let mut world = world(Position::new(2, 0), 2);
  let before_digest = world.digest();

  let result = world
    .execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("distance-two ranged attack should be accepted");

  assert_eq!(
    result.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::RANGED,
      remaining_hit_points: HitPoints::new(1),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("target exists")
      .hit_points(),
    HitPoints::new(1)
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("attacker exists")
      .ready_at()
      .value(),
    1
  );
  assert_ne!(world.digest(), before_digest);
}

#[test]
fn ranged_attack_accepts_distance_three_and_emits_death() {
  let mut world = world(Position::new(3, 0), 1);

  let result = world
    .execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("distance-three ranged attack should be accepted");

  assert_eq!(
    result.events(),
    &[
      Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::RANGED,
        remaining_hit_points: HitPoints::new(0),
      },
      Event::Died {
        actor: ActorId::new(2),
      },
    ]
  );
  assert!(
    !world
      .actor(ActorId::new(2))
      .expect("dead target retained")
      .is_alive()
  );
}

#[test]
fn ranged_attack_rejects_melee_and_far_targets_atomically() {
  for target in [Position::new(1, 0), Position::new(4, 0)] {
    let mut world = world(target, 2);
    let before = world.clone();

    assert_eq!(
      world.execute(Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      }),
      Err(CommandError::RangedAttackOutOfRange {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
      })
    );
    assert_eq!(world, before);
  }
}

#[test]
fn ranged_attack_rejects_wall_blocked_and_diagonal_targets_atomically() {
  let mut blocked = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::Wall, Tile::Floor, Tile::Floor],
    )
    .expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should be valid");
  let before = blocked.clone();
  assert_eq!(
    blocked.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::RangedAttackNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  assert_eq!(blocked, before);

  let mut diagonal = WorldState::new(
    GridMap::filled(3, 3, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 1),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should be valid");
  let before = diagonal.clone();
  assert_eq!(
    diagonal.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::RangedAttackNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  assert_eq!(diagonal, before);
}

#[test]
fn ranged_attack_rejects_invalid_identity_and_scheduler_requests_atomically() {
  let mut world = world(Position::new(2, 0), 2);
  for (command, expected) in [
    (
      Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(99),
      },
      CommandError::UnknownTarget(ActorId::new(99)),
    ),
    (
      Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(1),
      },
      CommandError::CannotAttackSelf(ActorId::new(1)),
    ),
    (
      Command::RangedAttack {
        actor: ActorId::new(2),
        target: ActorId::new(1),
      },
      CommandError::ActorNotScheduled {
        requested: ActorId::new(2),
        scheduled: ActorId::new(1),
      },
    ),
  ] {
    let before = world.clone();
    let before_digest = world.digest();
    assert_eq!(world.execute(command), Err(expected));
    assert_eq!(world, before);
    assert_eq!(world.digest(), before_digest);
  }

  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("target should be killable for the dead-target rejection");
  let before = world.clone();
  assert_eq!(
    world.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::TargetDead(ActorId::new(2)))
  );
  assert_eq!(world, before);
}

#[test]
fn ranged_attack_replay_hash_is_distinct_and_repeatable() {
  let command = Command::RangedAttack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  };
  let mut first = ReplayTrace::new(7);
  first.record(command);
  let mut equivalent = ReplayTrace::new(7);
  equivalent.record(command);
  let mut melee = ReplayTrace::new(7);
  melee.record(Command::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  });

  assert_eq!(first.digest(), equivalent.digest());
  assert_ne!(first.digest(), melee.digest());
}
