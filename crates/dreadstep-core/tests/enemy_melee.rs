//! Contract tests for scheduled enemy melee intent.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Damage, Event, GridMap, HitPoints, MeleeReach, Position,
  Tile, WorldState,
};

fn adjacent_world(enemy_hp: u16, player_hp: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(0, 0),
        HitPoints::new(enemy_hp),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Player,
        Position::new(1, 0),
        HitPoints::new(player_hp),
      ),
    ],
  )
  .expect("test world should be valid")
}

#[test]
fn adjacent_enemy_discovers_attack_before_any_fallback() {
  let world = adjacent_world(4, 4);

  assert_eq!(
    world.legal_commands().last(),
    Some(&Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
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

#[test]
fn enemy_legal_commands_group_attacks_before_distant_chases() {
  let world = WorldState::new(
    GridMap::filled(4, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(3, 0)),
      Actor::new(ActorId::new(3), ActorKind::Player, Position::new(1, 0)),
    ],
  )
  .expect("test world should be valid");

  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .filter_map(|command| match command {
        Command::Attack { target, .. } => Some(("attack", target)),
        Command::Chase { target, .. } => Some(("chase", target)),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![("attack", ActorId::new(3)), ("chase", ActorId::new(2)),]
  );
}

#[test]
fn extended_player_melee_reach_keeps_ranged_target_exclusive() {
  let world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_melee_reach(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(4),
        MeleeReach::new(2).expect("two is a valid reach"),
      ),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");

  assert_eq!(
    world
      .legal_commands()
      .into_iter()
      .filter(|command| matches!(
        command,
        Command::Attack { .. } | Command::RangedAttack { .. }
      ))
      .collect::<Vec<_>>(),
    vec![Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }]
  );
}

#[test]
fn enemy_attack_uses_fixed_damage_and_standard_schedule() {
  let mut world = adjacent_world(4, 4);
  let before_digest = world.digest();

  let result = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent enemy attack should be accepted");

  assert_eq!(
    result.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::MELEE,
      remaining_hit_points: HitPoints::new(3),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("enemy remains visible")
      .ready_at()
      .value(),
    1
  );
  assert_ne!(world.digest(), before_digest);
}

#[test]
fn enemy_attack_emits_death_and_preserves_dead_target_record() {
  let mut world = adjacent_world(4, 1);

  let result = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent enemy attack should be accepted");

  assert_eq!(
    result.events(),
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
  assert!(
    !world
      .actor(ActorId::new(2))
      .expect("dead player remains inspectable")
      .is_alive()
  );
}

#[test]
fn enemy_attack_out_of_range_is_atomic_and_hidden() {
  let mut world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(2, 0)),
    ],
  )
  .expect("test world should be valid");
  let before = world.clone();

  assert!(!world.legal_commands().iter().any(|command| {
    matches!(
      command,
      Command::Attack { actor, target }
        if *actor == ActorId::new(1) && *target == ActorId::new(2)
    )
  }));
  assert_eq!(
    world.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(dreadstep_core::CommandError::AttackOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  assert_eq!(world, before);
}
