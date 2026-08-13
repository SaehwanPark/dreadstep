//! Versioned protocol projection tests for canonical run outcomes.

use dreadstep_core::{
  Actor, ActorId as CoreActorId, ActorKind as CoreActorKind, GridMap, Position as CorePosition,
  Tile, WorldState,
};
use dreadstep_protocol::{ActorId, PROTOCOL_VERSION, RunOutcome, WorldSnapshot};

#[test]
fn snapshot_projects_in_progress_outcome_and_current_protocol_version() {
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(
        CoreActorId::new(1),
        CoreActorKind::Player,
        CorePosition::new(0, 0),
      ),
      Actor::new(
        CoreActorId::new(2),
        CoreActorKind::Enemy,
        CorePosition::new(1, 0),
      ),
    ],
  )
  .expect("world should be valid");
  let snapshot = WorldSnapshot::from_world(&world);

  assert_eq!(PROTOCOL_VERSION, 27);
  assert_eq!(snapshot.protocol_version(), PROTOCOL_VERSION);
  assert_eq!(snapshot.outcome(), RunOutcome::InProgress);
  assert_eq!(snapshot.actors()[0].id(), ActorId::new(1));
  let json = serde_json::to_value(snapshot).expect("snapshot should serialize");
  assert_eq!(json["outcome"], "in_progress");
}

#[test]
fn outcome_schema_is_explicit_and_stable() {
  let schema = serde_json::to_value(schemars::schema_for!(WorldSnapshot))
    .expect("snapshot schema should serialize");
  assert!(schema["properties"]["outcome"].is_object());
  assert!(schema["$defs"]["RunOutcome"].is_object());
}

#[test]
fn outcome_values_use_stable_snake_case_wire_names() {
  for (outcome, expected) in [
    (RunOutcome::InProgress, "in_progress"),
    (RunOutcome::Defeat, "defeat"),
    (RunOutcome::Victory, "victory"),
  ] {
    assert_eq!(
      serde_json::to_value(outcome).expect("outcome should serialize"),
      expected
    );
  }
}
