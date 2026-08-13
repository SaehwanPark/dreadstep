//! Contract tests for the authored Frost Flask throw.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Direction, Event, GridMap, Item,
  ItemDefinitionId, ItemId, Position, ThrowableEffect, Tile, WorldState,
};

fn world() -> WorldState {
  let mut world = WorldState::new(
    GridMap::from_tiles(4, 1, vec![Tile::Floor; 4]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  world
}

fn assert_rejected(mut world: WorldState, command: Command, expected: CommandError) {
  let before = world.clone();
  let digest = world.digest();
  assert_eq!(world.execute(command), Err(expected));
  assert_eq!(world, before);
  assert_eq!(world.digest(), digest);
}

#[test]
fn throw_consumes_flask_applies_chilled_and_costs_standard_time() {
  let mut world = world();
  let result = world
    .execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    })
    .unwrap();
  assert_eq!(
    result.events(),
    &[
      Event::ItemThrown {
        actor: ActorId::new(1),
        item: ItemId::new(104),
        target: ActorId::new(2),
      },
      Event::StatusApplied {
        actor: ActorId::new(2),
        status: dreadstep_core::StatusKind::Chilled,
        remaining_actions: 2,
      },
    ]
  );
  assert!(world.actor(ActorId::new(1)).unwrap().inventory().is_empty());
  assert_eq!(world.actor(ActorId::new(1)).unwrap().ready_at().value(), 1);
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    2
  );
}

#[test]
fn throw_is_discovered_in_item_then_target_order_and_rejects_atomically() {
  let mut world = world();
  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .find(|command| matches!(command, Command::Throw { .. })),
    Some(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    })
  );
  let before = world.clone();
  let digest = world.digest();
  assert_eq!(
    world.execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(999),
      target: ActorId::new(2),
    }),
    Err(CommandError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(999),
    })
  );
  assert_eq!(world, before);
  assert_eq!(world.digest(), digest);
  assert_eq!(
    world.execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(1),
    }),
    Err(CommandError::CannotThrowSelf(ActorId::new(1)))
  );
}

#[test]
fn throw_legal_order_sorts_item_then_target_identity() {
  let mut world = WorldState::new(
    GridMap::from_tiles(5, 1, vec![Tile::Floor; 5]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(9), ActorKind::Enemy, Position::new(2, 0)),
      Actor::new(ActorId::new(3), ActorKind::Enemy, Position::new(3, 0)),
    ],
  )
  .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(105),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();

  let throws = world
    .legal_commands()
    .into_iter()
    .filter_map(|command| match command {
      Command::Throw { item, target, .. } => Some((item, target)),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(
    throws,
    vec![
      (ItemId::new(104), ActorId::new(3)),
      (ItemId::new(104), ActorId::new(9)),
      (ItemId::new(105), ActorId::new(3)),
      (ItemId::new(105), ActorId::new(9)),
    ]
  );
}

#[test]
fn throw_rejects_blocked_or_diagonal_targets_without_consumption() {
  let mut blocked = world();
  blocked.set_tile(Position::new(1, 0), Tile::Wall);
  let before = blocked.clone();
  assert!(matches!(
    blocked.execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    }),
    Err(CommandError::ThrowNoLineOfSight { .. })
  ));
  assert_eq!(blocked, before);
}

#[test]
#[expect(
  clippy::too_many_lines,
  reason = "the throw rejection matrix keeps each invalid boundary explicit"
)]
fn throw_rejections_cover_role_item_target_and_geometry_atomically() {
  let mut non_throwable = world();
  non_throwable.actor(ActorId::new(1)).expect("player exists");
  non_throwable
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(101), ItemDefinitionId::new(1)),
    )
    .unwrap();
  assert_rejected(
    non_throwable,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(101),
      target: ActorId::new(2),
    },
    CommandError::ItemNotThrowable {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    },
  );

  let mut equipped = world();
  equipped
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(104),
    })
    .unwrap();
  equipped
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  assert_rejected(
    equipped,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    },
    CommandError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(104),
    },
  );

  let mut wrong_role = world();
  wrong_role
    .give_item(
      ActorId::new(2),
      Item::with_throwable_effect(
        ItemId::new(105),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  wrong_role
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert_rejected(
    wrong_role,
    Command::Throw {
      actor: ActorId::new(2),
      item: ItemId::new(105),
      target: ActorId::new(1),
    },
    CommandError::ThrowRequiresPlayer(ActorId::new(2)),
  );

  assert_rejected(
    world(),
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(99),
    },
    CommandError::UnknownTarget(ActorId::new(99)),
  );

  let mut dead_target = world();
  dead_target
    .set_hit_points(ActorId::new(2), dreadstep_core::HitPoints::new(0))
    .unwrap();
  assert_rejected(
    dead_target,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    },
    CommandError::TargetDead(ActorId::new(2)),
  );

  let mut out_of_range = WorldState::new(
    GridMap::from_tiles(5, 1, vec![Tile::Floor; 5]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .unwrap();
  out_of_range
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  assert_rejected(
    out_of_range,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    },
    CommandError::ThrowOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    },
  );

  let mut too_far = WorldState::new(
    GridMap::from_tiles(5, 1, vec![Tile::Floor; 5]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(4, 0)),
    ],
  )
  .unwrap();
  too_far
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  assert_rejected(
    too_far,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    },
    CommandError::ThrowOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    },
  );

  let mut diagonal = WorldState::new(
    GridMap::from_tiles(4, 4, vec![Tile::Floor; 16]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 1)),
    ],
  )
  .unwrap();
  diagonal
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  assert_rejected(
    diagonal,
    Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    },
    CommandError::ThrowNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    },
  );
}

#[test]
#[expect(
  clippy::too_many_lines,
  reason = "the throw refresh and status-accounting scenario keeps scheduler order explicit"
)]
fn throw_refreshes_target_and_expires_a_chilled_thrower_once() {
  let mut refresh_world = WorldState::new(
    GridMap::from_tiles(4, 1, vec![Tile::Floor; 4]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .unwrap();
  for item_id in [104, 105] {
    refresh_world
      .give_item(
        ActorId::new(1),
        Item::with_throwable_effect(
          ItemId::new(item_id),
          ItemDefinitionId::new(5),
          ThrowableEffect::Chill,
        ),
      )
      .unwrap();
  }
  refresh_world
    .execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    })
    .unwrap();
  refresh_world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  let refresh = refresh_world
    .execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(105),
      target: ActorId::new(2),
    })
    .unwrap();
  assert!(matches!(
    refresh.events(),
    [
      Event::ItemThrown { .. },
      Event::StatusApplied {
        actor,
        remaining_actions: 2,
        ..
      },
    ] if *actor == ActorId::new(2)
  ));
  assert_eq!(
    refresh_world
      .actor(ActorId::new(2))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    2
  );

  let mut chilled_thrower = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::ChillTrap, Tile::Floor, Tile::Floor],
    )
    .unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(3, 0)),
    ],
  )
  .unwrap();
  chilled_thrower
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();
  chilled_thrower
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .unwrap();
  chilled_thrower
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  chilled_thrower
    .execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    })
    .unwrap();
  chilled_thrower
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  let expiry = chilled_thrower
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert!(matches!(
    expiry.events(),
    [Event::Waited { .. }, Event::StatusExpired { actor, .. }]
      if *actor == ActorId::new(1)
  ));
  assert!(
    chilled_thrower
      .actor(ActorId::new(1))
      .unwrap()
      .status()
      .is_none()
  );
}

#[test]
fn throwing_while_chilled_consumes_the_thrower_status_only() {
  let mut world = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::ChillTrap, Tile::Floor, Tile::Floor],
    )
    .unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(3, 0)),
    ],
  )
  .unwrap();
  world
    .give_item(
      ActorId::new(1),
      Item::with_throwable_effect(
        ItemId::new(104),
        ItemDefinitionId::new(5),
        ThrowableEffect::Chill,
      ),
    )
    .unwrap();

  world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .unwrap();
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  let result = world
    .execute(Command::Throw {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    })
    .unwrap();

  assert!(matches!(
    result.events(),
    [
      Event::ItemThrown { .. },
      Event::StatusApplied { actor, .. },
    ] if *actor == ActorId::new(2)
  ));
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    1
  );
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    2
  );
}
