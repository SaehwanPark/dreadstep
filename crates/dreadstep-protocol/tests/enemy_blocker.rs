//! Versioned protocol contracts for the stationary Blocker archetype.

use dreadstep_core::{
  Actor as CoreActor, ActorId as CoreActorId, EnemyBehavior as CoreEnemyBehavior,
  GridMap as CoreGridMap, Position as CorePosition, Tile as CoreTile, WorldState,
};
use dreadstep_protocol::{EnemyBehavior, PROTOCOL_VERSION, WorldSnapshot};
use serde_json::json;

#[test]
fn blocker_behavior_has_a_stable_snake_case_projection() {
  assert_eq!(
    EnemyBehavior::from(CoreEnemyBehavior::Blocker),
    EnemyBehavior::Blocker
  );
  assert_eq!(
    CoreEnemyBehavior::from(EnemyBehavior::Blocker),
    CoreEnemyBehavior::Blocker
  );
  assert_eq!(
    serde_json::to_value(EnemyBehavior::Blocker).expect("behavior should serialize"),
    json!("blocker")
  );
}

#[test]
fn version_33_snapshot_projects_the_authored_blocker_behavior() {
  let world = WorldState::new(
    CoreGridMap::filled(3, 1, CoreTile::Floor).expect("test map should be valid"),
    vec![CoreActor::with_enemy_behavior(
      CoreActorId::new(2),
      CorePosition::new(1, 0),
      CoreEnemyBehavior::Blocker,
    )],
  )
  .expect("world should be valid");

  let snapshot = WorldSnapshot::from_world(&world);
  let digest = snapshot.digest().value();
  assert_eq!(PROTOCOL_VERSION, 33);
  assert_eq!(snapshot.protocol_version(), 33);
  assert_eq!(
    serde_json::to_value(snapshot).expect("snapshot should serialize"),
    json!({
      "protocol_version": 33,
      "outcome": "in_progress",
      "current_time": 0,
      "next_actor": 2,
      "digest": digest,
      "actors": [{
        "id": 2,
        "kind": "enemy",
        "behavior": "blocker",
        "position": {"x": 1, "y": 0},
        "hit_points": 10,
        "life": "alive",
        "ready_at": 0,
        "melee_reach": 1,
        "ranged_ammo": 3,
        "inventory_capacity": 4,
        "inventory": [],
        "equipped_item": null,
        "heard_noise": null,
        "status": null
      }],
      "ground_items": []
    })
  );
}
