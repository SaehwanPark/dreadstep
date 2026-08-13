//! Deterministic kick-open-door and noise contract tests.

use dreadstep_core::{
  ActionCost, ActionTime, Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap,
  Position, Tile, WorldState,
};

fn kick_world() -> WorldState {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Door, Tile::Floor])
    .expect("kick map should validate");
  WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("kick world should validate")
}

#[test]
fn kick_opens_adjacent_door_emits_ordered_noise_and_uses_standard_time() {
  let mut world = kick_world();
  let result = world
    .execute(Command::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent closed door should be kickable");
  assert_eq!(
    world.map().tile_at(Position::new(1, 0)),
    Some(Tile::OpenDoor)
  );
  assert_eq!(
    result.events(),
    &[
      Event::DoorOpened {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      Event::NoiseCreated {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        radius: 3,
      },
    ]
  );
  assert_eq!(
    result.current_time(),
    ActionTime::new(ActionCost::STANDARD.value())
  );
}

#[test]
fn kick_discovery_and_invalid_targets_are_deterministic_and_atomic() {
  let world = kick_world();
  assert!(world.legal_commands().contains(&Command::Kick {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  }));

  for position in [
    Position::new(0, 0),
    Position::new(2, 0),
    Position::new(1, 1),
    Position::new(9, 9),
  ] {
    let mut world = kick_world();
    let before = world.clone();
    let digest = world.digest();
    assert!(matches!(
      world.execute(Command::Kick {
        actor: ActorId::new(1),
        position,
      }),
      Err(CommandError::KickTargetInvalid { .. })
    ));
    assert_eq!(world, before);
    assert_eq!(world.digest(), digest);
  }

  let mut world = kick_world();
  world
    .execute(Command::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("first kick should succeed");
  let before = world.clone();
  assert!(matches!(
    world.execute(Command::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }),
    Err(CommandError::KickTargetInvalid { .. })
  ));
  assert_eq!(world, before);
}
