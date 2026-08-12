//! Contract tests for the bounded typed melee-reach preparation.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Damage, Event, GridMap, HitPoints, MeleeReach,
  Position, Tile, WorldState,
};

fn world_with_reach(reach: MeleeReach, target: Position) -> WorldState {
  WorldState::new(
    GridMap::filled(5, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_melee_reach(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
        reach,
      ),
      Actor::with_hit_points(ActorId::new(2), ActorKind::Enemy, target, HitPoints::new(2)),
    ],
  )
  .expect("test world should be valid")
}

#[test]
fn melee_reach_rejects_zero_and_defaults_to_one_tile() {
  assert_eq!(MeleeReach::new(0), None);
  assert_eq!(MeleeReach::DEFAULT.value(), 1);
  assert_eq!(
    Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)).melee_reach(),
    MeleeReach::DEFAULT
  );
}

#[test]
fn default_melee_reach_rejects_two_tile_attacks_atomically() {
  let mut world = world_with_reach(MeleeReach::DEFAULT, Position::new(2, 0));
  let before = world.clone();

  assert!(!world.legal_commands().contains(&Command::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
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
  assert_eq!(world, before);
}

#[test]
fn explicit_two_tile_melee_reach_accepts_attack_and_preserves_melee_evidence() {
  let reach = MeleeReach::new(2).expect("two is a valid reach");
  let mut world = world_with_reach(reach, Position::new(2, 0));
  let before_digest = world.digest();

  assert!(world.legal_commands().contains(&Command::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
  let result = world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("explicit reach should permit the two-tile attack");

  assert_eq!(
    result.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::MELEE,
      remaining_hit_points: HitPoints::new(1),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("attacker exists")
      .ready_at()
      .value(),
    1
  );
  assert_eq!(result.current_time().value(), 0);
  assert_ne!(world.digest(), before_digest);
}

#[test]
fn extended_melee_targets_keep_stable_actor_id_order() {
  let world = WorldState::new(
    GridMap::filled(4, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_melee_reach(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
        MeleeReach::new(2).expect("two is a valid reach"),
      ),
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
        Command::Attack { target, .. } => Some(target),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![ActorId::new(2), ActorId::new(3)]
  );
}

#[test]
fn melee_reach_is_part_of_the_world_digest() {
  let default_world = world_with_reach(MeleeReach::DEFAULT, Position::new(2, 0));
  let extended_world = world_with_reach(
    MeleeReach::new(2).expect("two is a valid reach"),
    Position::new(2, 0),
  );

  assert_ne!(default_world.digest(), extended_world.digest());
}

#[test]
fn melee_attack_rejections_for_self_unknown_and_dead_targets_are_atomic() {
  let mut self_target = world_with_reach(MeleeReach::DEFAULT, Position::new(1, 0));
  let before_self = self_target.clone();
  assert_eq!(
    self_target.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(1),
    }),
    Err(CommandError::CannotAttackSelf(ActorId::new(1)))
  );
  assert_eq!(self_target, before_self);

  let mut unknown_target = world_with_reach(MeleeReach::DEFAULT, Position::new(1, 0));
  let before_unknown = unknown_target.clone();
  assert_eq!(
    unknown_target.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(99),
    }),
    Err(CommandError::UnknownTarget(ActorId::new(99)))
  );
  assert_eq!(unknown_target, before_unknown);

  let mut dead_target = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .expect("test world should be valid");
  dead_target
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("first attack should kill the target");
  let before_dead = dead_target.clone();
  assert_eq!(
    dead_target.execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(CommandError::TargetDead(ActorId::new(2)))
  );
  assert_eq!(dead_target, before_dead);
}
