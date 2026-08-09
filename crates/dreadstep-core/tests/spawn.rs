//! Contract tests for validated core actor spawning.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, Position, Tile, WorldError, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should be valid")
}

#[test]
fn spawn_adds_a_living_actor_at_a_walkable_unoccupied_position() {
  let mut world = world();
  world
    .spawn(Actor::with_hit_points(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(2, 0),
      HitPoints::new(2),
    ))
    .expect("spawn should be accepted");

  let spawned = world
    .actor(ActorId::new(2))
    .expect("actor should be present");
  assert_eq!(spawned.kind(), ActorKind::Enemy);
  assert_eq!(spawned.position(), Position::new(2, 0));
  assert_eq!(spawned.hit_points(), HitPoints::new(2));
}

#[test]
fn spawn_rejects_invalid_actor_data_without_mutating_the_world() {
  let cases = [
    (
      Actor::new(ActorId::new(1), ActorKind::Enemy, Position::new(2, 0)),
      WorldError::DuplicateActorId(ActorId::new(1)),
    ),
    (
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(3, 0)),
      WorldError::ActorOutOfBounds {
        actor: ActorId::new(2),
        position: Position::new(3, 0),
      },
    ),
    (
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(0),
      ),
      WorldError::ActorDeadAtStart {
        actor: ActorId::new(2),
      },
    ),
  ];

  for (actor, expected) in cases {
    let mut world = world();
    let before = world.digest();
    assert_eq!(world.spawn(actor), Err(expected));
    assert_eq!(world.digest(), before);
    assert_eq!(world.actor(ActorId::new(2)), None);
  }
}

#[test]
fn spawn_rejects_blocked_tiles_and_living_actor_overlap() {
  let mut blocked = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Wall, Tile::Floor])
      .expect("map should be valid"),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should be valid");
  let before_blocked = blocked.digest();
  assert_eq!(
    blocked.spawn(Actor::new(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(1, 0),
    )),
    Err(WorldError::ActorOnBlockedTile {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
  );
  assert_eq!(blocked.digest(), before_blocked);

  let mut overlapping = world();
  let before_overlap = overlapping.digest();
  assert_eq!(
    overlapping.spawn(Actor::new(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(0, 0),
    )),
    Err(WorldError::OverlappingActors {
      first: ActorId::new(1),
      second: ActorId::new(2),
      position: Position::new(0, 0),
    })
  );
  assert_eq!(overlapping.digest(), before_overlap);
}

#[test]
fn spawn_uses_current_time_and_can_reuse_a_dead_actor_tile() {
  let mut world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
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
  .expect("world should be valid");
  world
    .execute(dreadstep_core::Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");
  assert_eq!(world.current_time().value(), 1);

  world
    .spawn(Actor::new(
      ActorId::new(3),
      ActorKind::Enemy,
      Position::new(1, 0),
    ))
    .expect("dead actor tile should be reusable");
  assert_eq!(world.current_time().value(), 1);
  assert_eq!(
    world
      .actor(ActorId::new(3))
      .expect("spawned actor")
      .ready_at()
      .value(),
    1
  );

  world
    .execute(dreadstep_core::Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("wait should be accepted");
  assert_eq!(world.current_time().value(), 1);
  world
    .spawn(Actor::new(
      ActorId::new(4),
      ActorKind::Enemy,
      Position::new(2, 0),
    ))
    .expect("spawn should use current time");
  assert_eq!(
    world
      .actor(ActorId::new(4))
      .expect("spawned actor")
      .ready_at()
      .value(),
    1
  );
}
