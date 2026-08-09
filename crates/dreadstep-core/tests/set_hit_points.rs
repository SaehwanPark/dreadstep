//! Contract tests for validated core tester hit-point mutation.

use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, Command, Event, GridMap, HitPoints, Position, Tile,
  WorldError, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should be valid")
}

#[test]
fn set_hit_points_to_zero_preserves_the_record_but_removes_living_presence() {
  let mut world = world();
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should be updated");

  let actor = world
    .actor(ActorId::new(2))
    .expect("dead record should remain");
  assert_eq!(actor.hit_points(), HitPoints::new(0));
  assert!(!actor.is_alive());
  assert_eq!(actor.position(), Position::new(1, 0));
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
  assert_eq!(world.current_time().value(), 0);
}

#[test]
fn killing_the_earliest_actor_advances_time_to_the_next_living_ready_time() {
  let mut world = world();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should be able to wait");
  assert_eq!(world.current_time().value(), 0);
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));

  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should be updated");
  assert_eq!(world.current_time().value(), 1);
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));

  let result = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("surviving actor should still act");
  assert_eq!(
    result.events(),
    &[Event::Waited {
      actor: ActorId::new(1),
      at: ActionTime::new(1),
    }]
  );
}

#[test]
fn reviving_reanchors_readiness_at_current_time_without_rewinding_the_scheduler() {
  let mut world = world();
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(1))
    .expect("known actor should be updated");
  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");
  assert_eq!(world.current_time().value(), 1);

  world
    .set_hit_points(ActorId::new(2), HitPoints::new(3))
    .expect("dead actor should be revivable");

  let revived = world
    .actor(ActorId::new(2))
    .expect("actor should remain present");
  assert_eq!(revived.hit_points(), HitPoints::new(3));
  assert!(revived.is_alive());
  assert_eq!(revived.ready_at().value(), 1);
  assert_eq!(world.current_time().value(), 1);

  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should still act at the current time");
  assert_eq!(world.current_time().value(), 1);
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));
}

#[test]
fn reviving_rejects_a_dead_record_whose_tile_is_now_livingly_occupied() {
  let mut world = world();
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(1))
    .expect("known actor should be updated");
  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should be accepted");
  world
    .spawn(Actor::new(
      ActorId::new(3),
      ActorKind::Enemy,
      Position::new(1, 0),
    ))
    .expect("dead actor tile should be reusable");
  let before = world.clone();

  assert_eq!(
    world.set_hit_points(ActorId::new(2), HitPoints::new(3)),
    Err(WorldError::OverlappingActors {
      first: ActorId::new(3),
      second: ActorId::new(2),
      position: Position::new(1, 0),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn set_hit_points_rejects_unknown_actor_without_mutating_the_world() {
  let mut world = world();
  let before = world.clone();
  let unknown = ActorId::new(9);

  assert_eq!(
    world.set_hit_points(unknown, HitPoints::new(4)),
    Err(WorldError::UnknownActor(unknown))
  );
  assert_eq!(world, before);
}
