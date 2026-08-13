//! Deterministic Frostcaster Chill casts.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, EnemyBehavior, Event, GridMap, HitPoints,
  Position, ReplayTrace, Tile, WorldState,
};

fn frostcaster_world() -> WorldState {
  let map = GridMap::from_tiles(5, 1, vec![Tile::Floor; 5]).expect("cast map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(3, 0),
        HitPoints::new(10),
      ),
      Actor::with_ranged_ammo(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(10),
        0,
      ),
    ],
  )
  .expect("Frostcaster world should validate");
  assert_eq!(
    world.set_enemy_behavior(ActorId::new(2), EnemyBehavior::Frostcaster),
    Some(EnemyBehavior::Pursuer)
  );
  world
}

#[test]
fn frostcaster_casts_chill_before_ranged_attack_without_ammunition() {
  let mut world = frostcaster_world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  assert!(world.legal_commands().contains(&Command::CastChill {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  }));
  assert!(!world.legal_commands().contains(&Command::RangedAttack {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  }));
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  let before_ammo = world
    .actors()
    .find(|actor| actor.id() == ActorId::new(2))
    .expect("caster exists")
    .ranged_ammo();
  let result = world
    .execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("clear cardinal Chill cast should succeed");
  assert_eq!(
    result.events(),
    &[
      Event::ChillCast {
        caster: ActorId::new(2),
        target: ActorId::new(1),
      },
      Event::StatusApplied {
        actor: ActorId::new(1),
        status: dreadstep_core::StatusKind::Chilled,
        remaining_actions: 2,
      },
    ]
  );
  assert_eq!(
    world
      .actors()
      .find(|actor| actor.id() == ActorId::new(1))
      .and_then(Actor::status)
      .map(dreadstep_core::Status::remaining_actions),
    Some(2)
  );
  assert_eq!(
    world
      .actors()
      .find(|actor| actor.id() == ActorId::new(2))
      .expect("caster remains alive")
      .ranged_ammo(),
    before_ammo
  );
  assert_eq!(
    world
      .actors()
      .find(|actor| actor.id() == ActorId::new(2))
      .expect("caster remains alive")
      .ready_at()
      .value(),
    2
  );
}

#[test]
fn frostcaster_adjacent_attack_stays_ahead_of_cast_fallback() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor; 3]).expect("adjacent map validates");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 0)),
      Actor::with_enemy_behavior(
        ActorId::new(2),
        Position::new(0, 0),
        EnemyBehavior::Frostcaster,
      ),
    ],
  )
  .expect("adjacent world validates");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to adjacent Frostcaster");
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}

#[test]
fn frostcaster_cast_rejections_are_atomic_and_typed() {
  let mut world = frostcaster_world();
  let baseline = world.clone();
  let baseline_digest = world.digest();
  assert_eq!(
    world.execute(Command::CastChill {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(dreadstep_core::CommandError::CastChillRequiresFrostcaster(
      ActorId::new(1)
    ))
  );
  assert_eq!(world, baseline);
  let mut dead_target = frostcaster_world();
  dead_target
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("target should be killable for dead-target coverage");
  let dead_baseline = dead_target.clone();
  assert_eq!(
    dead_target.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::CastChillTargetDead(ActorId::new(1)))
  );
  assert_eq!(dead_target, dead_baseline);
  assert_eq!(world.digest(), baseline_digest);
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  let baseline = world.clone();
  assert_eq!(
    world.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(2),
    }),
    Err(dreadstep_core::CommandError::CannotCastChillSelf(
      ActorId::new(2)
    ))
  );
  assert_eq!(world, baseline);
  assert_eq!(
    world.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(99),
    }),
    Err(dreadstep_core::CommandError::CastChillUnknownTarget(
      ActorId::new(99)
    ))
  );
  assert_eq!(world, baseline);
  world
    .execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("valid cast remains available after rejected attempts");
}

#[test]
fn frostcaster_rejects_diagonal_rays_without_mutation() {
  let mut world = WorldState::new(
    GridMap::filled(4, 4, Tile::Floor).expect("diagonal map should validate"),
    vec![
      Actor::with_enemy_behavior(
        ActorId::new(2),
        Position::new(0, 0),
        EnemyBehavior::Frostcaster,
      ),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(2, 1)),
    ],
  )
  .expect("diagonal world should validate");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  let before = world.clone();
  assert_eq!(
    world.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(CommandError::CastChillNoLineOfSight {
      caster: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn frostcaster_refreshes_chilled_target_and_changes_digest_and_replay() {
  let mut first = frostcaster_world();
  let initial_digest = first.digest();
  first
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  let first_cast = first
    .execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("first cast should succeed");
  assert_ne!(first.digest(), initial_digest);
  assert!(first_cast.events().iter().any(|event| matches!(
    event,
    Event::StatusApplied {
      actor,
      remaining_actions: 2,
      ..
    } if *actor == ActorId::new(1)
  )));

  first
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("chilled player should still take an accepted turn");
  let refreshed = first
    .execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("second cast should refresh Chilled");
  assert_eq!(
    first
      .actor(ActorId::new(1))
      .and_then(Actor::status)
      .map(dreadstep_core::Status::remaining_actions),
    Some(2)
  );
  assert!(refreshed.events().iter().any(|event| matches!(
    event,
    Event::StatusApplied {
      actor,
      remaining_actions: 2,
      ..
    } if *actor == ActorId::new(1)
  )));

  let mut replay = ReplayTrace::new(7);
  replay.record(Command::Wait {
    actor: ActorId::new(1),
  });
  replay.record(Command::CastChill {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  });
  let mut equivalent_replay = ReplayTrace::new(7);
  equivalent_replay.record(Command::Wait {
    actor: ActorId::new(1),
  });
  equivalent_replay.record(Command::CastChill {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  });
  assert_eq!(replay.digest(), equivalent_replay.digest());
  let mut ranged_replay = ReplayTrace::new(7);
  ranged_replay.record(Command::Wait {
    actor: ActorId::new(1),
  });
  ranged_replay.record(Command::RangedAttack {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  });
  assert_ne!(replay.digest(), ranged_replay.digest());
}

#[test]
fn frostcaster_cast_rejects_out_of_range_and_blocked_rays() {
  let map = GridMap::from_tiles(6, 1, vec![Tile::Floor; 6]).expect("blocked cast map validates");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(5, 0)),
      Actor::with_enemy_behavior(
        ActorId::new(2),
        Position::new(1, 0),
        EnemyBehavior::Frostcaster,
      ),
    ],
  )
  .expect("blocked cast world validates");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  let baseline = world.clone();
  assert_eq!(
    world.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(dreadstep_core::CommandError::CastChillOutOfRange {
      caster: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  assert_eq!(world, baseline);
  let mut clear_world = frostcaster_world();
  assert_eq!(
    clear_world.set_tile(Position::new(2, 0), Tile::Wall),
    Some(Tile::Floor)
  );
  clear_world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to Frostcaster");
  assert_eq!(
    clear_world.execute(Command::CastChill {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    }),
    Err(dreadstep_core::CommandError::CastChillNoLineOfSight {
      caster: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}
