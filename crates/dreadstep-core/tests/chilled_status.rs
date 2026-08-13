//! Contract tests for the authored one-shot chilled trap.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Event, GridMap, Position, Tile, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::ChillTrap, Tile::Floor]).unwrap(),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .unwrap()
}

#[test]
fn entering_chill_trap_applies_status_and_consumes_tile() {
  let mut world = world();
  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .unwrap();
  assert!(matches!(
    result.events(),
    [
      Event::Moved { .. },
      Event::StatusApplied {
        remaining_actions: 2,
        ..
      }
    ]
  ));
  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Floor));
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    2
  );
}

#[test]
fn chilled_actions_cost_extra_time_and_expire_after_two_actions() {
  let mut world = world();
  world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .unwrap();
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  let first = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert_eq!(world.actor(ActorId::new(1)).unwrap().ready_at().value(), 3);
  assert!(world.actor(ActorId::new(1)).unwrap().status().is_some());
  assert!(matches!(first.events(), [Event::Waited { .. }]));
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .unwrap();
  let second = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert!(matches!(
    second.events(),
    [Event::Waited { .. }, Event::StatusExpired { .. }]
  ));
  assert!(world.actor(ActorId::new(1)).unwrap().status().is_none());
}

#[test]
fn entering_another_chill_trap_refreshes_without_consuming_the_new_status() {
  let mut world = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::ChillTrap, Tile::ChillTrap, Tile::Floor],
    )
    .unwrap(),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .unwrap();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    1
  );
  let refreshed = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .unwrap();
  assert!(matches!(
    refreshed.events(),
    [
      Event::Moved { .. },
      Event::StatusApplied {
        remaining_actions: 2,
        ..
      }
    ]
  ));
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .unwrap()
      .status()
      .unwrap()
      .remaining_actions(),
    2
  );
}
