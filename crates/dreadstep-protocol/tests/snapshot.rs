//! Contract tests for the versioned read-only world snapshot projection.

use dreadstep_core::{
  Actor, ActorId as CoreActorId, ActorKind as CoreActorKind, Command, GridMap,
  HitPoints as CoreHitPoints, Position as CorePosition, Tile, WorldState,
};
use dreadstep_protocol::{
  ActionTime, ActorId, ActorKind, ActorSnapshot, HitPoints, LifeState, PROTOCOL_VERSION, Position,
  WorldSnapshot,
};

fn world_with_actors() -> WorldState {
  let map = GridMap::filled(3, 1, Tile::Floor).expect("test map should be valid");
  WorldState::new(
    map,
    vec![
      Actor::new(
        CoreActorId::new(2),
        CoreActorKind::Enemy,
        CorePosition::new(1, 0),
      ),
      Actor::new(
        CoreActorId::new(1),
        CoreActorKind::Player,
        CorePosition::new(0, 0),
      ),
    ],
  )
  .expect("test world should be valid")
}

#[test]
fn snapshot_has_version_and_stable_actor_order() {
  let snapshot = WorldSnapshot::from_world(&world_with_actors());

  assert_eq!(snapshot.protocol_version(), PROTOCOL_VERSION);
  assert_eq!(
    snapshot
      .actors()
      .iter()
      .map(ActorSnapshot::id)
      .collect::<Vec<_>>(),
    [ActorId::new(1), ActorId::new(2)]
  );
  let player = &snapshot.actors()[0];
  assert_eq!(player.kind(), ActorKind::Player);
  assert_eq!(player.position(), Position::new(0, 0));
  assert_eq!(player.hit_points(), HitPoints::new(10));
  assert_eq!(player.life(), LifeState::Alive);
  assert_eq!(player.ready_at(), ActionTime::new(0));
  assert_eq!(player.ranged_ammo(), 3);
}

#[test]
fn equivalent_worlds_produce_equal_snapshots() {
  assert_eq!(
    WorldSnapshot::from_world(&world_with_actors()),
    WorldSnapshot::from_world(&world_with_actors())
  );
}

#[test]
fn snapshot_retains_dead_actor_for_inspection() {
  let mut world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(
        CoreActorId::new(1),
        CoreActorKind::Player,
        CorePosition::new(0, 0),
      ),
      Actor::with_hit_points(
        CoreActorId::new(2),
        CoreActorKind::Enemy,
        CorePosition::new(1, 0),
        CoreHitPoints::new(1),
      ),
    ],
  )
  .expect("test world should be valid");
  let before = WorldSnapshot::from_world(&world);
  world
    .execute(Command::Attack {
      actor: CoreActorId::new(1),
      target: CoreActorId::new(2),
    })
    .expect("adjacent attack should succeed");

  let snapshot = WorldSnapshot::from_world(&world);
  assert_ne!(snapshot, before);
  assert_ne!(snapshot.digest(), before.digest());
  assert_eq!(snapshot.current_time(), ActionTime::new(1));
  let dead_actor = snapshot
    .actors()
    .iter()
    .find(|actor| actor.id() == ActorId::new(2))
    .expect("dead actor should remain visible");
  assert!(!dead_actor.is_alive());
  assert_eq!(dead_actor.hit_points(), HitPoints::new(0));
  assert_eq!(dead_actor.life(), LifeState::Dead);
  assert_eq!(snapshot.next_actor(), Some(ActorId::new(1)));
}
