//! Reclosable-door contract tests.

use dreadstep_core::{
  ActionCost, ActionTime, Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap,
  Position, Tile, WorldState,
};

fn open_door_world() -> WorldState {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::OpenDoor, Tile::Floor])
    .expect("open-door map should validate");
  WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("open-door world should validate")
}

#[test]
fn open_door_is_walkable_and_transparent() {
  let world = open_door_world();
  assert!(world.map().is_walkable(Position::new(1, 0)));
  assert!(!Tile::OpenDoor.blocks_ranged_line_of_sight());
}

#[test]
fn close_reverts_adjacent_open_door_and_costs_standard_time() {
  let mut world = open_door_world();
  let before_digest = world.digest();
  let result = world
    .execute(Command::Close {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent open door should close");

  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Door));
  assert_eq!(
    result.events(),
    &[Event::DoorClosed {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
  assert_eq!(
    result.current_time(),
    ActionTime::new(ActionCost::STANDARD.value())
  );
  assert_ne!(world.digest(), before_digest);

  let close = Command::Close {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  };
  let mut first = dreadstep_core::ReplayTrace::new(7);
  first.record(close);
  let mut second = dreadstep_core::ReplayTrace::new(7);
  second.record(close);
  assert_eq!(first.commands(), &[close]);
  assert_eq!(first.digest(), second.digest());
}

#[test]
fn close_is_advertised_and_invalid_targets_are_atomic() {
  let world = open_door_world();
  assert_eq!(
    world.legal_commands(),
    vec![
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::North,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::South,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::West,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::East,
      },
      Command::Wait {
        actor: ActorId::new(1),
      },
      Command::Close {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
    ]
  );

  for position in [
    Position::new(0, 0),
    Position::new(2, 0),
    Position::new(1, 1),
    Position::new(9, 9),
  ] {
    let mut world = open_door_world();
    let before = world.clone();
    let digest = world.digest();
    assert!(matches!(
      world.execute(Command::Close {
        actor: ActorId::new(1),
        position,
      }),
      Err(CommandError::CloseTargetInvalid { .. })
    ));
    assert_eq!(world, before);
    assert_eq!(world.digest(), digest);
  }
}

#[test]
fn occupied_open_door_rejects_close_atomically() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::OpenDoor, Tile::Floor])
    .expect("open-door map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
    ],
  )
  .expect("occupied open-door world should validate");
  let before = world.clone();
  assert!(
    !world
      .legal_commands()
      .iter()
      .any(|command| { matches!(command, Command::Close { .. }) })
  );
  assert!(matches!(
    world.execute(Command::Close {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }),
    Err(CommandError::DoorCloseOccupied { .. })
  ));
  assert_eq!(world, before);
}
