//! Deterministic adjacent door interaction contract tests.

use dreadstep_core::{
  ActionCost, Actor, ActorId, ActorKind, Command, CommandError, Direction, Event, GridMap,
  Position, ReplayTrace, Tile, WorldState,
};

fn door_world() -> WorldState {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Door, Tile::Floor])
    .expect("door map should validate");
  WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("door world should validate")
}

#[test]
fn closed_door_blocks_entry_and_ranged_sight() {
  let world = door_world();
  assert!(!world.map().is_walkable(Position::new(1, 0)));
  assert!(Tile::Door.blocks_ranged_line_of_sight());
  let mut world = world;
  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("blocked movement remains an accepted standard action");
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor exists")
      .position(),
    Position::new(0, 0)
  );
  assert_eq!(
    result.events()[0],
    Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: dreadstep_core::BlockReason::Terrain,
    }
  );
}

#[test]
fn adjacent_interact_opens_door_and_advances_standard_time() {
  let mut world = door_world();
  let result = world
    .execute(Command::Interact {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent closed door should open");

  assert_eq!(
    world.map().tile_at(Position::new(1, 0)),
    Some(Tile::OpenDoor)
  );
  assert_eq!(
    result.events(),
    &[Event::DoorOpened {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
  assert_eq!(result.current_time().value(), ActionCost::STANDARD.value());
}

#[test]
fn legal_interact_and_replay_digest_are_deterministic() {
  let world = door_world();
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
      Command::Interact {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      Command::Kick {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
    ]
  );
  let mut first = ReplayTrace::new(7);
  first.record(Command::Interact {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  });
  let mut second = ReplayTrace::new(7);
  second.record(Command::Interact {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  });
  assert_eq!(first.digest(), second.digest());
}

#[test]
fn invalid_interactions_are_atomic() {
  let mut world = door_world();
  for position in [
    Position::new(2, 0),
    Position::new(1, 1),
    Position::new(9, 9),
  ] {
    let before = world.clone();
    let digest = world.digest();
    assert!(matches!(
      world.execute(Command::Interact {
        actor: ActorId::new(1),
        position,
      }),
      Err(CommandError::InteractTargetInvalid { .. })
    ));
    assert_eq!(world, before);
    assert_eq!(world.digest(), digest);
  }

  world
    .execute(Command::Interact {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("door should open once");
  let before = world.clone();
  let digest = world.digest();
  assert!(matches!(
    world.execute(Command::Interact {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }),
    Err(CommandError::InteractTargetInvalid { .. })
  ));
  assert_eq!(world, before);
  assert_eq!(world.digest(), digest);
}
