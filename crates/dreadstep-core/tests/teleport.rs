//! Contract tests for validated tester teleport.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position, Tile,
  WorldError, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::from_tiles(
      4,
      2,
      vec![
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Floor,
        Tile::Floor,
      ],
    )
    .expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("world should be valid")
}

#[test]
fn teleport_preserves_actor_and_scheduler_state_while_changing_position() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .execute(dreadstep_core::Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("first wait should be accepted");
  world
    .execute(dreadstep_core::Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("second wait should be accepted");

  let before = world
    .actor(ActorId::new(1))
    .expect("actor should exist")
    .clone();
  let current_time = world.current_time();
  let digest = world.digest();
  world
    .teleport(ActorId::new(1), Position::new(3, 1))
    .expect("walkable destination should be accepted");

  let after = world.actor(ActorId::new(1)).expect("actor should exist");
  assert_eq!(after.position(), Position::new(3, 1));
  assert_eq!(after.id(), before.id());
  assert_eq!(after.kind(), before.kind());
  assert_eq!(after.hit_points(), before.hit_points());
  assert_eq!(after.is_alive(), before.is_alive());
  assert_eq!(after.ready_at(), before.ready_at());
  assert_eq!(after.inventory(), before.inventory());
  assert_eq!(world.current_time(), current_time);
  assert_ne!(world.digest(), digest);
}

#[test]
fn teleport_rejects_invalid_destinations_atomically() {
  let mut world = world();
  let before = world.clone();

  assert_eq!(
    world.teleport(ActorId::new(9), Position::new(3, 1)),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);

  assert_eq!(
    world.teleport(ActorId::new(1), Position::new(4, 0)),
    Err(WorldError::TeleportOutOfBounds {
      actor: ActorId::new(1),
      position: Position::new(4, 0),
    })
  );
  assert_eq!(world, before);

  assert_eq!(
    world.teleport(ActorId::new(1), Position::new(1, 1)),
    Err(WorldError::TeleportOnBlockedTile {
      actor: ActorId::new(1),
      position: Position::new(1, 1),
    })
  );
  assert_eq!(world, before);

  assert_eq!(
    world.teleport(ActorId::new(1), Position::new(2, 0)),
    Err(WorldError::TeleportOccupied {
      actor: ActorId::new(1),
      blocker: ActorId::new(2),
      position: Position::new(2, 0),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn dead_actor_records_can_teleport_without_occupying_living_tiles() {
  let mut world = world();
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should be updated");
  world
    .teleport(ActorId::new(1), Position::new(2, 0))
    .expect("living actors may use dead-record tiles");
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
  world
    .teleport(ActorId::new(2), Position::new(0, 0))
    .expect("dead records do not occupy living tiles");

  let actor = world.actor(ActorId::new(2)).expect("record should remain");
  assert!(!actor.is_alive());
  assert_eq!(actor.position(), Position::new(0, 0));
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("living actor should remain")
      .position(),
    Position::new(2, 0)
  );
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
}
