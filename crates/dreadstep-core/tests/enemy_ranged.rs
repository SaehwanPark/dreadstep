//! Contract tests for deterministic enemy ranged attacks.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Damage, Event, GridMap, HitPoints, Position,
  Tile, WorldState,
};

fn enemy_world(enemy_ammo: u16, target: Position) -> WorldState {
  enemy_world_with_target_hp(enemy_ammo, target, 2)
}

fn enemy_world_with_target_hp(
  enemy_ammo: u16,
  target: Position,
  target_hit_points: u16,
) -> WorldState {
  WorldState::new(
    GridMap::filled(5, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_ranged_ammo(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(0, 0),
        HitPoints::new(10),
        enemy_ammo,
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Player,
        target,
        HitPoints::new(target_hit_points),
      ),
    ],
  )
  .expect("test world should be valid")
}

fn schedule_enemy(mut world: WorldState) -> WorldState {
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("enemy should take the initial turn");
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("player should yield the second turn");
  world
}

#[test]
fn enemy_discovers_clear_ranged_target_before_chase() {
  let world = schedule_enemy(enemy_world(3, Position::new(2, 0)));

  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .filter_map(|command| match command {
        Command::RangedAttack { target, .. } => Some(("ranged", target)),
        Command::Chase { target, .. } => Some(("chase", target)),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![("ranged", ActorId::new(2)), ("chase", ActorId::new(2)),]
  );
}

#[test]
fn enemy_ranged_attack_reuses_damage_and_two_tick_schedule() {
  let mut world = schedule_enemy(enemy_world(3, Position::new(2, 0)));
  let result = world
    .execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("clear enemy ranged attack should be accepted");

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
      .actor(ActorId::new(1))
      .expect("enemy remains present")
      .ranged_ammo(),
    2
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("enemy remains present")
      .ready_at()
      .value(),
    3
  );
}

#[test]
fn enemy_ranged_attack_emits_death_and_retains_dead_target() {
  let mut world = schedule_enemy(enemy_world_with_target_hp(3, Position::new(2, 0), 1));
  let result = world
    .execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("clear lethal enemy ranged attack should be accepted");

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
      .expect("dead target remains inspectable")
      .is_alive()
  );
}

#[test]
fn enemy_ranged_targets_follow_stable_actor_id_order() {
  let mut world = WorldState::new(
    GridMap::filled(5, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(3), ActorKind::Player, Position::new(2, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(3, 0)),
    ],
  )
  .expect("multiple-target enemy world validates");
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("enemy should take the initial turn");
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("lowest-id player should yield the second turn");
  world
    .execute(Command::Wait {
      actor: ActorId::new(3),
    })
    .expect("second player should yield the third turn");

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
fn enemy_ranged_action_is_hidden_without_ammunition_and_rejects_atomically() {
  let mut world = schedule_enemy(enemy_world(0, Position::new(2, 0)));
  let before = world.clone();

  assert!(!world.legal_commands().iter().any(|command| {
    matches!(
      command,
      Command::RangedAttack { actor, target }
        if *actor == ActorId::new(1) && *target == ActorId::new(2)
    )
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
fn enemy_ranged_rejects_blocked_line_of_sight_atomically() {
  let mut world = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Wall, Tile::Floor])
      .expect("blocked-ray map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("blocked-ray enemy world validates");
  let before = world.clone();

  assert_eq!(
    world.execute(Command::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::RangedAttackNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn enemy_ranged_discovery_reuses_clear_cardinal_line_of_sight() {
  let world = WorldState::new(
    GridMap::from_tiles(
      3,
      3,
      vec![
        Tile::Floor,
        Tile::Wall,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
      ],
    )
    .expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
      Actor::new(ActorId::new(3), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(4), ActorKind::Player, Position::new(0, 2)),
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
    vec![ActorId::new(4)]
  );
}
