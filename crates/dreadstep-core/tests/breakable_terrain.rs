//! Deterministic breakable-terrain contract tests.

use dreadstep_core::{
  ActionCost, ActionTime, Actor, ActorId, ActorKind, BlockReason, Command, CommandError, Direction,
  Event, GridMap, Position, Tile, WorldState,
};

fn breakable_world() -> WorldState {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Breakable, Tile::Floor])
    .expect("breakable map should validate");
  WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("breakable world should validate")
}

#[test]
fn breakable_terrain_blocks_entry_and_ranged_sight_until_broken() {
  let mut world = breakable_world();
  assert!(!Tile::Breakable.is_walkable());
  assert!(Tile::Breakable.blocks_ranged_line_of_sight());
  let blocked = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("blocked movement remains an accepted action");
  assert_eq!(
    blocked.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Terrain,
    }]
  );

  let mut world = breakable_world();
  let result = world
    .execute(Command::Break {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent breakable terrain should break");
  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Floor));
  assert_eq!(
    result.events(),
    &[Event::BreakableBroken {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
  assert_eq!(
    result.current_time(),
    ActionTime::new(ActionCost::STANDARD.value())
  );
}

#[test]
fn break_discovery_and_invalid_targets_are_deterministic_and_atomic() {
  let world = breakable_world();
  assert!(world.legal_commands().contains(&Command::Break {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  }));

  for position in [
    Position::new(0, 0),
    Position::new(2, 0),
    Position::new(1, 1),
    Position::new(9, 9),
  ] {
    let mut world = breakable_world();
    let before = world.clone();
    let digest = world.digest();
    assert!(matches!(
      world.execute(Command::Break {
        actor: ActorId::new(1),
        position,
      }),
      Err(CommandError::BreakTargetInvalid { .. })
    ));
    assert_eq!(world, before);
    assert_eq!(world.digest(), digest);
  }

  let mut world = breakable_world();
  world
    .execute(Command::Break {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("first break should succeed");
  let before = world.clone();
  assert!(matches!(
    world.execute(Command::Break {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }),
    Err(CommandError::BreakTargetInvalid { .. })
  ));
  assert_eq!(world, before);
}
