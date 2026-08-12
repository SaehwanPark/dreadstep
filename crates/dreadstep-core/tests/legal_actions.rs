//! Contract tests for deterministic core legal-command discovery.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, HitPoints, Position, Tile, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .expect("test world should be valid")
}

#[test]
fn player_legal_commands_have_stable_direction_wait_and_attack_order() {
  assert_eq!(
    world().legal_commands(),
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
      Command::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      },
    ]
  );
}

#[test]
fn enemy_legal_commands_include_chase_after_scheduler_advances() {
  let mut world = world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should be scheduled first");

  assert_eq!(
    world.legal_commands().last(),
    Some(&Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}

#[test]
fn player_legal_commands_exclude_non_adjacent_attack_targets() {
  let world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");

  assert!(!world.legal_commands().iter().any(|command| {
    matches!(
      command,
      Command::Attack { actor, target }
        if *actor == ActorId::new(1) && *target == ActorId::new(2)
    )
  }));
}

#[test]
fn player_legal_commands_include_ranged_targets_at_distance_two_or_three() {
  let world = WorldState::new(
    GridMap::filled(5, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      Actor::new(ActorId::new(3), ActorKind::Enemy, Position::new(3, 0)),
      Actor::new(ActorId::new(4), ActorKind::Enemy, Position::new(4, 0)),
    ],
  )
  .expect("test world should be valid");

  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .filter_map(|command| match command {
        Command::RangedAttack { target, .. } => Some(target),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![ActorId::new(2), ActorId::new(3)]
  );
}

#[test]
fn player_combat_commands_follow_target_id_order_across_melee_and_ranged() {
  let world = WorldState::new(
    GridMap::filled(4, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
      Actor::new(ActorId::new(3), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .expect("test world should be valid");

  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .filter_map(|command| match command {
        Command::Attack { target, .. } => Some((target, "melee")),
        Command::RangedAttack { target, .. } => Some((target, "ranged")),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![(ActorId::new(2), "ranged"), (ActorId::new(3), "melee")]
  );
}

#[test]
fn enemy_legal_commands_exclude_dead_chase_targets() {
  let mut world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
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
  .expect("test world should be valid");
  world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("enemy should be scheduled first");
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("player two should act second");
  world
    .execute(Command::Attack {
      actor: ActorId::new(3),
      target: ActorId::new(2),
    })
    .expect("player three should kill player two");

  assert_eq!(
    world.legal_commands().last(),
    Some(&Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(3),
    })
  );
  assert!(!world.legal_commands().iter().any(|command| {
    matches!(
      command,
      Command::Chase { actor, target }
        if *actor == ActorId::new(1) && *target == ActorId::new(2)
    )
  }));
}
