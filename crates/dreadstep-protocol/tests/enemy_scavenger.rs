//! Tests for protocol serialization and projection of Scavenger enemy behavior.

use dreadstep_core::{
  Actor as CoreActor, ActorId as CoreActorId, EnemyBehavior as CoreEnemyBehavior,
  GridMap as CoreGridMap, Position as CorePosition, Tile as CoreTile, WorldState as CoreWorldState,
};
use dreadstep_protocol::{EnemyBehavior, PROTOCOL_VERSION, WorldSnapshot};
use serde_json::json;

#[test]
fn scavenger_behavior_round_trips_through_core_and_serializes_as_snake_case() {
  assert_eq!(
    EnemyBehavior::from(CoreEnemyBehavior::Scavenger),
    EnemyBehavior::Scavenger
  );
  assert_eq!(
    CoreEnemyBehavior::from(EnemyBehavior::Scavenger),
    CoreEnemyBehavior::Scavenger
  );
  assert_eq!(
    serde_json::to_value(EnemyBehavior::Scavenger).expect("behavior should serialize"),
    json!("scavenger")
  );
}

#[test]
fn world_snapshot_projects_scavenger_behavior() {
  let core_world = CoreWorldState::new(
    CoreGridMap::filled(2, 2, CoreTile::Floor).expect("map should validate"),
    vec![CoreActor::with_enemy_behavior(
      CoreActorId::new(2),
      CorePosition::new(1, 1),
      CoreEnemyBehavior::Scavenger,
    )],
  )
  .expect("world should validate");

  let snapshot = WorldSnapshot::from_world(&core_world);
  assert_eq!(PROTOCOL_VERSION, 33);
  assert_eq!(snapshot.protocol_version(), PROTOCOL_VERSION);
  assert_eq!(snapshot.actors()[0].behavior(), EnemyBehavior::Scavenger);
}
