//! Characterization tests moved from the former core `lib.rs` unit module.

use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, BlockReason, Command, CommandError, Damage, Direction,
  Event, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, MapError, Position, ReplayTrace, Tile,
  WorldError, WorldState,
};

fn floor_map(width: u32, height: u32) -> GridMap {
  GridMap::filled(width, height, Tile::Floor).expect("test map should be valid")
}

#[test]
fn moves_the_scheduled_actor_and_reports_the_next_actor() {
  let map = floor_map(3, 1);
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");

  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("actor one is scheduled first");

  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().position(),
    Position::new(1, 0)
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );
  assert_eq!(result.next_actor(), Some(ActorId::new(2)));
  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
}

#[test]
fn reports_terrain_blocking_and_still_consumes_the_action() {
  let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Wall]).unwrap();
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();

  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .unwrap();

  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().position(),
    Position::new(0, 0)
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );
  assert_eq!(
    result.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Terrain,
    }]
  );
}

#[test]
fn reports_actor_blocking_without_moving_either_actor() {
  let map = floor_map(2, 1);
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .unwrap();

  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Actor(ActorId::new(2)),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().position(),
    Position::new(1, 0)
  );
}

#[test]
fn treats_out_of_bounds_movement_as_terrain_blocking() {
  let mut world = WorldState::new(
    floor_map(1, 1),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();

  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::West,
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(-1, 0),
      reason: BlockReason::Terrain,
    }]
  );
}

#[test]
fn adjacent_melee_attack_reduces_hit_points_and_consumes_an_action() {
  let mut world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(3),
      ),
    ],
  )
  .unwrap();

  let result = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::MELEE,
      remaining_hit_points: HitPoints::new(2),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().hit_points(),
    HitPoints::new(2)
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );
  assert_eq!(result.next_actor(), Some(ActorId::new(2)));
}

#[test]
fn killing_an_actor_emits_death_and_removes_it_from_scheduling_and_occupancy() {
  let mut world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .unwrap();

  let attack = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();

  assert_eq!(
    attack.events(),
    &[
      Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::MELEE,
        remaining_hit_points: HitPoints::new(0),
      },
      Event::Died {
        actor: ActorId::new(2),
      },
    ]
  );
  assert!(!world.actor(ActorId::new(2)).unwrap().is_alive());
  assert_eq!(attack.next_actor(), Some(ActorId::new(1)));

  let move_result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .unwrap();
  assert_eq!(
    move_result.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
}

#[test]
fn rejects_unknown_dead_self_and_out_of_range_attack_targets() {
  let mut world = WorldState::new(
    floor_map(3, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .unwrap();

  assert_eq!(
    world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(99),
    }),
    Err(CommandError::UnknownTarget(ActorId::new(99)))
  );
  assert_eq!(
    world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(1),
    }),
    Err(CommandError::CannotAttackSelf(ActorId::new(1)))
  );
  assert_eq!(
    world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::AttackOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );

  let mut dead_target_world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .unwrap();
  dead_target_world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(
    dead_target_world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::TargetDead(ActorId::new(2)))
  );
}

#[test]
fn rejects_an_actor_that_starts_with_zero_hit_points() {
  assert_eq!(
    WorldState::new(
      floor_map(1, 1),
      vec![Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(0),
      )],
    ),
    Err(WorldError::ActorDeadAtStart {
      actor: ActorId::new(1),
    })
  );
}

#[test]
fn enemy_chase_uses_horizontal_priority_for_diagonal_targets() {
  let mut world = WorldState::new(
    floor_map(4, 4),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 2)),
    ],
  )
  .unwrap();

  let result = world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );
}

#[test]
fn enemy_chase_uses_vertical_direction_when_columns_align() {
  let mut world = WorldState::new(
    floor_map(1, 3),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(0, 2)),
    ],
  )
  .unwrap();

  let result = world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(0, 1),
    }]
  );
}

#[test]
fn enemy_chase_reuses_terrain_and_actor_blocking() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Wall, Tile::Floor]).unwrap();
  let mut terrain_world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .unwrap();
  let terrain_result = terrain_world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(
    terrain_result.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Terrain,
    }]
  );
  assert_eq!(
    terrain_world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );

  let mut actor_world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(1, 0)),
    ],
  )
  .unwrap();
  let actor_result = actor_world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(
    actor_result.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Actor(ActorId::new(2)),
    }]
  );
}

#[test]
fn rejects_player_self_unknown_and_dead_chase_targets() {
  let mut player_world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .unwrap();
  assert_eq!(
    player_world.execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::ChaseRequiresEnemy(ActorId::new(1)))
  );

  let mut self_world = WorldState::new(
    floor_map(1, 1),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Enemy,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  assert_eq!(
    self_world.execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(1),
    }),
    Err(CommandError::CannotChaseSelf(ActorId::new(1)))
  );

  assert_eq!(
    self_world.execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(99),
    }),
    Err(CommandError::UnknownTarget(ActorId::new(99)))
  );

  let mut dead_world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Player,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .unwrap();
  dead_world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(
    dead_world.execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::TargetDead(ActorId::new(2)))
  );
}

#[test]
fn enemy_chase_can_enter_a_dead_non_target_tile() {
  let mut world = WorldState::new(
    floor_map(3, 1),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Player,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
      Actor::new(ActorId::new(3), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .unwrap();

  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .unwrap();
  world
    .execute(Command::Wait {
      actor: ActorId::new(3),
    })
    .unwrap();
  let result = world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(3),
    })
    .unwrap();

  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(2)
  );
}

#[test]
fn replay_trace_digest_is_sensitive_to_seed_and_command_order() {
  let move_command = Command::Move {
    actor: ActorId::new(1),
    direction: Direction::East,
  };
  let wait_command = Command::Wait {
    actor: ActorId::new(1),
  };
  let mut first = ReplayTrace::new(7);
  first.record(move_command);
  first.record(wait_command);

  let mut reordered = ReplayTrace::new(7);
  reordered.record(wait_command);
  reordered.record(move_command);

  let mut reseeded = ReplayTrace::new(8);
  reseeded.record(move_command);
  reseeded.record(wait_command);

  let mut identical = ReplayTrace::new(7);
  identical.record(move_command);
  identical.record(wait_command);

  assert_eq!(first.seed(), 7);
  assert_eq!(first.commands(), &[move_command, wait_command]);
  assert_eq!(first.digest(), identical.digest());
  assert_ne!(first.digest(), reordered.digest());
  assert_ne!(first.digest(), reseeded.digest());
}

#[test]
fn equivalent_worlds_have_equal_digests_after_identical_combat_transitions() {
  let actors = vec![
    Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
    Actor::with_hit_points(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(1, 0),
      HitPoints::new(1),
    ),
  ];
  let mut first = WorldState::new(floor_map(2, 1), actors.clone()).unwrap();
  let mut second = WorldState::new(floor_map(2, 1), actors).unwrap();
  let initial_digest = first.digest();
  let commands = [
    Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
    Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    },
  ];
  for command in commands {
    first.execute(command).unwrap();
    second.execute(command).unwrap();
  }

  assert_ne!(initial_digest, first.digest());
  assert_eq!(first.digest(), second.digest());
}

#[test]
fn state_digest_changes_when_map_semantics_differ() {
  let actors = vec![Actor::new(
    ActorId::new(1),
    ActorKind::Player,
    Position::new(0, 0),
  )];
  let floor_world =
    WorldState::new(GridMap::filled(2, 1, Tile::Floor).unwrap(), actors.clone()).unwrap();
  let stairs_world = WorldState::new(
    GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Stairs]).unwrap(),
    actors.clone(),
  )
  .unwrap();
  let wall_world = WorldState::new(
    GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Wall]).unwrap(),
    actors,
  )
  .unwrap();

  assert_ne!(floor_world.digest(), stairs_world.digest());
  assert_ne!(stairs_world.digest(), wall_world.digest());
  assert_ne!(floor_world.digest(), wall_world.digest());
}

#[test]
fn scheduler_orders_equal_ready_times_by_actor_id() {
  let mut world = WorldState::new(
    floor_map(3, 1),
    vec![
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
    ],
  )
  .unwrap();

  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
  let first = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert_eq!(first.next_actor(), Some(ActorId::new(2)));
  let second = world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(second.next_actor(), Some(ActorId::new(1)));
}

#[test]
fn rejects_a_command_for_an_unscheduled_actor() {
  let mut world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .unwrap();

  assert_eq!(
    world.execute(Command::Wait {
      actor: ActorId::new(2)
    }),
    Err(CommandError::ActorNotScheduled {
      requested: ActorId::new(2),
      scheduled: ActorId::new(1),
    })
  );
}

#[test]
fn rejects_invalid_world_occupancy_and_map_data() {
  assert_eq!(GridMap::from_tiles(0, 1, vec![]), Err(MapError::ZeroWidth));
  assert_eq!(
    GridMap::from_tiles(2, 1, vec![Tile::Floor]),
    Err(MapError::TileCountMismatch {
      expected: 2,
      actual: 1,
    })
  );
  assert_eq!(
    GridMap::from_tiles(i32::MAX as u32 + 1, 1, vec![]),
    Err(MapError::CoordinateRange {
      width: i32::MAX as u32 + 1,
      height: 1,
    })
  );

  let map = GridMap::filled(2, 1, Tile::Floor).unwrap();
  assert_eq!(
    WorldState::new(
      map.clone(),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(0, 0)),
      ],
    ),
    Err(WorldError::OverlappingActors {
      first: ActorId::new(1),
      second: ActorId::new(2),
      position: Position::new(0, 0),
    })
  );

  assert_eq!(
    WorldState::new(
      map.clone(),
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(1, 0)),
      ],
    ),
    Err(WorldError::DuplicateActorId(ActorId::new(1)))
  );

  assert_eq!(
    WorldState::new(
      map,
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(2, 0),
      )],
    ),
    Err(WorldError::ActorOutOfBounds {
      actor: ActorId::new(1),
      position: Position::new(2, 0),
    })
  );

  let wall_map = GridMap::from_tiles(1, 1, vec![Tile::Wall]).unwrap();
  assert_eq!(
    WorldState::new(
      wall_map,
      vec![Actor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
      )],
    ),
    Err(WorldError::ActorOnBlockedTile {
      actor: ActorId::new(1),
      position: Position::new(0, 0),
    })
  );
}

#[test]
fn scheduled_pickup_preserves_ground_order_and_advances_action() {
  let mut world = WorldState::new(
    floor_map(1, 1),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("test world should be valid");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(11), ItemDefinitionId::new(101)),
    )
    .expect("first item should be added");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(12), ItemDefinitionId::new(102)),
    )
    .expect("second item should be added");
  world
    .drop_item(ActorId::new(1), ItemId::new(11))
    .expect("first item should drop");
  world
    .drop_item(ActorId::new(1), ItemId::new(12))
    .expect("second item should drop");

  assert_eq!(
    world.legal_commands(),
    vec![
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::North,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::South,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::West,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: Direction::East,
      },
      Command::Wait {
        actor: ActorId::new(1),
      },
      Command::Pickup {
        actor: ActorId::new(1),
        item: ItemId::new(11),
      },
      Command::Pickup {
        actor: ActorId::new(1),
        item: ItemId::new(12),
      },
    ]
  );
  let before = world.digest();
  let result = world
    .execute(Command::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(11),
    })
    .expect("ground pickup should be accepted");
  assert_eq!(
    result.events(),
    &[Event::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(11),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().ready_at(),
    ActionTime::new(1)
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().inventory()[0].id(),
    ItemId::new(11)
  );
  assert_eq!(world.ground_items()[0].items()[0].id(), ItemId::new(12));
  assert_ne!(world.digest(), before);
}

#[test]
fn enemy_pickup_is_not_legal_and_rejected_atomically() {
  let mut world = WorldState::new(
    floor_map(2, 1),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(1, 0)),
    ],
  )
  .expect("test world should be valid");
  let item = Item::new(ItemId::new(11), ItemDefinitionId::new(101));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(1), item.id())
    .expect("item should drop");
  assert!(!world.legal_commands().iter().any(|command| {
    matches!(command, Command::Pickup { actor, item: candidate } if *actor == ActorId::new(1) && *candidate == item.id())
  }));
  let before = world.clone();
  assert_eq!(
    world.execute(Command::Pickup {
      actor: ActorId::new(1),
      item: item.id(),
    }),
    Err(CommandError::PickupRequiresPlayer(ActorId::new(1)))
  );
  assert_eq!(world, before);
}

#[test]
fn rejected_pickup_preserves_world_and_replay_evidence() {
  let mut world = WorldState::new(
    floor_map(1, 1),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("test world should be valid");
  let before = world.clone();
  assert_eq!(
    world.execute(Command::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(CommandError::ItemNotOnGround {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    })
  );
  assert_eq!(world, before);
}
