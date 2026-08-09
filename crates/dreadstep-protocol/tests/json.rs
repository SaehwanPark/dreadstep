//! Contract tests for versioned snapshot JSON projection.

use dreadstep_core::{
  Actor as CoreActor, ActorId as CoreActorId, ActorKind as CoreActorKind, GridMap as CoreGridMap,
  Item as CoreItem, ItemDefinitionId as CoreItemDefinitionId, ItemId as CoreItemId,
  Position as CorePosition, Tile as CoreTile, WorldState as CoreWorldState,
};
use dreadstep_protocol::{
  ActionTime, ActorId, CommandRequest, Direction, Event, ReplayEvidence, StateDigest, WorldSnapshot,
};

fn snapshot() -> WorldSnapshot {
  let mut world = CoreWorldState::new(
    CoreGridMap::filled(2, 1, CoreTile::Floor).expect("map should be valid"),
    vec![CoreActor::new(
      CoreActorId::new(1),
      CoreActorKind::Player,
      CorePosition::new(0, 0),
    )],
  )
  .expect("world should be valid");
  world
    .give_item(
      CoreActorId::new(1),
      CoreItem::new(CoreItemId::new(4), CoreItemDefinitionId::new(9)),
    )
    .expect("item should be accepted");
  WorldSnapshot::from_world(&world)
}

#[test]
fn snapshot_json_is_versioned_and_contains_stable_actor_item_fields() {
  let value = serde_json::to_value(snapshot()).expect("snapshot should serialize");
  assert_eq!(value["protocol_version"], 2);
  assert_eq!(value["current_time"], 0);
  assert_eq!(value["next_actor"], 1);
  assert!(value["digest"].is_number());
  assert_eq!(value["ground_items"], serde_json::json!([]));
  assert_eq!(value["actors"][0]["id"], 1);
  assert_eq!(value["actors"][0]["kind"], "player");
  assert_eq!(value["actors"][0]["life"], "alive");
  assert_eq!(value["actors"][0]["inventory"][0]["id"], 4);
  assert_eq!(value["actors"][0]["inventory"][0]["definition"], 9);
}

#[test]
fn equivalent_snapshots_have_identical_json_bytes() {
  let first = serde_json::to_string(&snapshot()).expect("snapshot should serialize");
  let second = serde_json::to_string(&snapshot()).expect("snapshot should serialize");
  assert_eq!(first, second);
}

#[test]
fn snapshot_schema_exposes_the_versioned_projection_shape() {
  let schema = schemars::schema_for!(WorldSnapshot);
  let value = serde_json::to_value(schema).expect("schema should serialize");
  let properties = &value["properties"];
  assert!(properties["protocol_version"].is_object());
  assert!(properties["current_time"].is_object());
  assert!(properties["next_actor"].is_object());
  assert!(properties["digest"].is_object());
  assert!(properties["actors"].is_object());
  assert!(properties["ground_items"].is_object());
}

#[test]
fn command_and_event_json_use_explicit_tagged_variants() {
  let request = CommandRequest::Move {
    actor: ActorId::new(3),
    direction: Direction::East,
  };
  assert_eq!(
    serde_json::to_value(request).expect("request should serialize"),
    serde_json::json!({"move": {"actor": 3, "direction": "east"}})
  );
  assert_eq!(
    serde_json::from_value::<CommandRequest>(serde_json::json!({
      "move": {"actor": 3, "direction": "east"}
    }))
    .expect("request should deserialize"),
    request
  );

  let event = Event::Waited {
    actor: ActorId::new(3),
    at: ActionTime::new(7),
  };
  assert_eq!(
    serde_json::to_value(event).expect("event should serialize"),
    serde_json::json!({"waited": {"actor": 3, "at": 7}})
  );
  let command_schema = serde_json::to_value(schemars::schema_for!(CommandRequest))
    .expect("command schema should serialize");
  assert!(command_schema["oneOf"].is_array());
  let event_schema =
    serde_json::to_value(schemars::schema_for!(Event)).expect("event schema should serialize");
  assert!(event_schema["oneOf"].is_array());
}

#[test]
fn replay_evidence_json_is_structured_and_schema_versioned() {
  let evidence = ReplayEvidence::new(
    7,
    vec![CommandRequest::Wait {
      actor: ActorId::new(1),
    }],
    StateDigest::new(11),
  );
  let value = serde_json::to_value(&evidence).expect("replay evidence should serialize");
  assert_eq!(
    value,
    serde_json::json!({
      "seed": 7,
      "commands": [{"wait": {"actor": 1}}],
      "digest": 11
    })
  );
  let schema = serde_json::to_value(schemars::schema_for!(ReplayEvidence))
    .expect("replay schema should serialize");
  assert_eq!(schema["type"], "object");
  assert!(schema["properties"]["seed"].is_object());
  assert!(schema["properties"]["commands"].is_object());
  assert!(schema["properties"]["digest"].is_object());
  let equivalent = ReplayEvidence::new(
    7,
    vec![CommandRequest::Wait {
      actor: ActorId::new(1),
    }],
    StateDigest::new(11),
  );
  assert_eq!(
    serde_json::to_string(&evidence).expect("evidence should serialize"),
    serde_json::to_string(&equivalent).expect("equivalent evidence should serialize")
  );
}
