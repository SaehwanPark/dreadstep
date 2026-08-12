//! Contract tests for the bounded scheduled ranged attack.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Damage, Event, GridMap, HitPoints, Position,
  ReplayTrace, Tile, WorldState,
};

fn world(target: Position, hit_points: u16) -> WorldState {
  world_with_ammo(target, hit_points, Actor::DEFAULT_RANGED_AMMO)
}

fn world_with_ammo(target: Position, hit_points: u16, ranged_ammo: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(7, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_ranged_ammo(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
        ranged_ammo,
      ),
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
fn ranged_attack_accepts_distance_two_and_advances_two_ticks() {
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
    2
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("attacker exists")
      .ranged_ammo(),
    Actor::DEFAULT_RANGED_AMMO - 1
  );
  assert_eq!(result.next_actor(), Some(ActorId::new(2)));
  assert_eq!(result.current_time().value(), 0);
  assert_ne!(world.digest(), before_digest);
}

#[test]
fn ranged_attack_reports_current_time_after_two_tick_transition() {
  let mut world = world(Position::new(2, 0), 2);

  for actor in [1, 2, 1, 2] {
    world
      .execute(Command::Wait {
        actor: ActorId::new(actor),
      })
      .expect("both actors should be scheduled in stable order");
  }

  let result = world
    .execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("distance-two ranged attack should be accepted");

  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("attacker exists")
      .ready_at()
      .value(),
    4
  );
  assert_eq!(result.next_actor(), Some(ActorId::new(2)));
  assert_eq!(result.current_time().value(), 2);
}

#[test]
fn cover_is_walkable_and_part_of_the_map_digest() {
  let actors = vec![
    Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
    Actor::with_hit_points(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(2, 0),
      HitPoints::new(2),
    ),
  ];
  let mut with_cover = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Cover, Tile::Floor])
      .expect("cover map should be valid"),
    actors.clone(),
  )
  .expect("cover map should accept the actors");
  let all_floor = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("floor map should be valid"),
    actors,
  )
  .expect("floor map should accept the actors");

  assert_ne!(with_cover.digest(), all_floor.digest());
  with_cover
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .expect("cover should be walkable for movement");
  assert_eq!(
    with_cover
      .actor(ActorId::new(1))
      .expect("actor remains visible")
      .position(),
    Position::new(1, 0)
  );
}

#[test]
fn ranged_attack_rejects_empty_ammunition_atomically_and_hides_the_action() {
  let mut world = world_with_ammo(Position::new(2, 0), 2, 0);
  let before = world.clone();

  assert!(!world.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
  assert_eq!(
    world.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::RangedAttackNoAmmunition(ActorId::new(1)))
  );
  assert_eq!(world, before);
}

#[test]
fn ranged_ammunition_is_part_of_the_world_digest() {
  let three_shot = world_with_ammo(Position::new(2, 0), 2, 3);
  let two_shot = world_with_ammo(Position::new(2, 0), 2, 2);

  assert_ne!(three_shot, two_shot);
  assert_ne!(three_shot.digest(), two_shot.digest());
}

#[test]
fn ranged_attack_accepts_distance_three_and_emits_death() {
  let mut world = world(Position::new(3, 0), 1);

  for actor in [1, 2, 1, 2] {
    world
      .execute(Command::Wait {
        actor: ActorId::new(actor),
      })
      .expect("both actors should be scheduled in stable order");
  }

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
  assert_eq!(result.current_time().value(), 4);
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
fn ranged_attack_rejects_walkable_cover_in_the_interior_ray_atomically() {
  let mut covered = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::Cover, Tile::Floor, Tile::Floor],
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
  .expect("cover should remain walkable for actor placement");
  let before = covered.clone();

  assert!(!covered.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
  assert_eq!(
    covered.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::RangedAttackNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  assert_eq!(covered, before);
}

#[test]
fn ranged_attack_allows_cover_at_either_endpoint() {
  for tiles in [
    vec![Tile::Cover, Tile::Floor, Tile::Floor, Tile::Floor],
    vec![Tile::Floor, Tile::Floor, Tile::Cover, Tile::Floor],
  ] {
    let mut world = WorldState::new(
      GridMap::from_tiles(4, 1, tiles).expect("endpoint cover map should be valid"),
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
    .expect("cover endpoints should remain walkable for actors");

    let result = world
      .execute(Command::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
      .expect("cover at an endpoint is not an interior blocker");
    assert_eq!(
      result.events(),
      &[Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::RANGED,
        remaining_hit_points: HitPoints::new(1),
      }]
    );
    assert_eq!(result.current_time().value(), 0);
    assert_eq!(
      world
        .actor(ActorId::new(1))
        .expect("attacker remains visible")
        .ready_at()
        .value(),
      2
    );
    assert_eq!(
      world
        .actor(ActorId::new(1))
        .expect("attacker remains visible")
        .ranged_ammo(),
      Actor::DEFAULT_RANGED_AMMO - 1
    );
  }
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
