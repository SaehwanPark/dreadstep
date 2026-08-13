//! Protocol behavior projection for the Brute archetype.

use dreadstep_core::{
  Actor, ActorId as CoreActorId, EnemyBehavior as CoreEnemyBehavior, GridMap,
  Position as CorePosition, Tile, WorldState,
};
use dreadstep_protocol::{ActorId, EnemyBehavior, Position, WorldSnapshot};

#[test]
fn snapshot_projects_brute_behavior_and_snake_case_json() {
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should validate"),
    vec![Actor::with_enemy_behavior(
      CoreActorId::new(1),
      CorePosition::new(1, 0),
      CoreEnemyBehavior::Brute,
    )],
  )
  .expect("world should validate");
  let snapshot = WorldSnapshot::from_world(&world);
  assert_eq!(snapshot.actors()[0].id(), ActorId::new(1));
  assert_eq!(snapshot.actors()[0].position(), Position::new(1, 0));
  assert_eq!(snapshot.actors()[0].behavior(), EnemyBehavior::Brute);
  let json = serde_json::to_value(snapshot).expect("snapshot should serialize");
  assert_eq!(json["actors"][0]["behavior"], "brute");
}
