//! Behavioral evidence for deterministic ranged ammunition reloading.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap, HitPoints, Tile, WorldState,
};

fn world(ammo: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![Actor::with_ranged_ammo(
      ActorId::new(1),
      ActorKind::Player,
      dreadstep_core::Position::new(0, 0),
      HitPoints::new(10),
      ammo,
    )],
  )
  .expect("world should be valid")
}

#[test]
fn reload_restores_capacity_and_consumes_one_standard_action() {
  let mut world = world(1);
  let result = world
    .execute(Command::Reload {
      actor: ActorId::new(1),
    })
    .expect("partial ammo should reload");

  assert_eq!(
    world.actor(ActorId::new(1)).expect("actor").ranged_ammo(),
    3
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor")
      .ready_at()
      .value(),
    1
  );
  assert_eq!(
    result.events(),
    &[Event::Reloaded {
      actor: ActorId::new(1),
      ammunition: 3,
    }]
  );
}

#[test]
fn reload_is_legal_only_below_capacity_and_changes_digest() {
  let mut world = world(0);
  let before = world.digest();
  assert!(world.legal_commands().contains(&Command::Reload {
    actor: ActorId::new(1),
  }));
  world
    .execute(Command::Reload {
      actor: ActorId::new(1),
    })
    .expect("empty ammo should reload");
  assert_ne!(world.digest(), before);
  assert!(!world.legal_commands().contains(&Command::Reload {
    actor: ActorId::new(1),
  }));
}

#[test]
fn full_ammo_reload_rejects_atomically() {
  let mut world = world(3);
  let before = world.clone();
  let error = world
    .execute(Command::Reload {
      actor: ActorId::new(1),
    })
    .expect_err("full ammo should not reload");
  assert_eq!(error, CommandError::ReloadNotNeeded(ActorId::new(1)));
  assert_eq!(world, before);
}

#[test]
fn over_capacity_ammo_is_not_lowered_by_reload() {
  let mut world = world(4);
  let before = world.clone();
  let error = world
    .execute(Command::Reload {
      actor: ActorId::new(1),
    })
    .expect_err("over-capacity ammo should not reload");
  assert_eq!(error, CommandError::ReloadNotNeeded(ActorId::new(1)));
  assert_eq!(world, before);
}

#[test]
fn reload_rejects_enemy_actor_with_typed_error() {
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![Actor::with_ranged_ammo(
      ActorId::new(1),
      ActorKind::Enemy,
      dreadstep_core::Position::new(0, 0),
      HitPoints::new(10),
      0,
    )],
  )
  .expect("world should be valid");
  let mut world = world;
  let error = world
    .execute(Command::Reload {
      actor: ActorId::new(1),
    })
    .expect_err("enemy reload should reject");
  assert_eq!(error, CommandError::ReloadRequiresPlayer(ActorId::new(1)));
}

#[test]
fn reload_rejects_an_unscheduled_player_atomically() {
  let mut world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::with_ranged_ammo(
        ActorId::new(1),
        ActorKind::Player,
        dreadstep_core::Position::new(0, 0),
        HitPoints::new(10),
        0,
      ),
      Actor::with_ranged_ammo(
        ActorId::new(2),
        ActorKind::Player,
        dreadstep_core::Position::new(2, 0),
        HitPoints::new(10),
        0,
      ),
    ],
  )
  .expect("world should be valid");
  let before = world.clone();
  let error = world
    .execute(Command::Reload {
      actor: ActorId::new(2),
    })
    .expect_err("only the first player may act");
  assert_eq!(
    error,
    CommandError::ActorNotScheduled {
      requested: ActorId::new(2),
      scheduled: ActorId::new(1),
    }
  );
  assert_eq!(world, before);
}
