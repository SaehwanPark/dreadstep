//! Contract tests for versioned snapshot JSON projection.

use dreadstep_core::{
  Actor as CoreActor, ActorId as CoreActorId, ActorKind as CoreActorKind, GridMap as CoreGridMap,
  Item as CoreItem, ItemDefinitionId as CoreItemDefinitionId, ItemId as CoreItemId,
  Position as CorePosition, Tile as CoreTile, WorldState as CoreWorldState,
};
use dreadstep_protocol::{
  ActionTime, ActorId, CommandRequest, Direction, Event, ItemId, ReplayEvidence, StateDigest,
  WorldSnapshot,
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

fn equipped_snapshot() -> WorldSnapshot {
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
  world
    .execute(dreadstep_core::Command::Equip {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(4),
    })
    .expect("item should equip");
  WorldSnapshot::from_world(&world)
}

#[test]
fn snapshot_json_is_versioned_and_contains_stable_actor_item_fields() {
  let value = serde_json::to_value(snapshot()).expect("snapshot should serialize");
  assert_eq!(value["protocol_version"], 3);
  assert_eq!(value["current_time"], 0);
  assert_eq!(value["next_actor"], 1);
  assert!(value["digest"].is_number());
  assert_eq!(value["ground_items"], serde_json::json!([]));
  assert_eq!(value["actors"][0]["id"], 1);
  assert_eq!(value["actors"][0]["kind"], "player");
  assert_eq!(value["actors"][0]["life"], "alive");
  assert_eq!(value["actors"][0]["inventory"][0]["id"], 4);
  assert_eq!(value["actors"][0]["inventory"][0]["definition"], 9);
  assert_eq!(value["actors"][0]["equipped_item"], serde_json::Value::Null);
  let equipped_value =
    serde_json::to_value(equipped_snapshot()).expect("snapshot should serialize");
  assert_eq!(equipped_value["protocol_version"], 3);
  assert_eq!(equipped_value["actors"][0]["equipped_item"], 4);
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
#[expect(
  clippy::too_many_lines,
  reason = "the contract intentionally round-trips every tagged command and event variant"
)]
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

  let equipment = CommandRequest::Equip {
    actor: ActorId::new(3),
    item: dreadstep_protocol::ItemId::new(4),
  };
  assert_eq!(
    serde_json::to_value(equipment).expect("equipment request should serialize"),
    serde_json::json!({"equip": {"actor": 3, "item": 4}})
  );
  assert_eq!(
    serde_json::from_value::<CommandRequest>(serde_json::json!({
      "equip": {"actor": 3, "item": 4}
    }))
    .expect("equip request should deserialize"),
    equipment
  );
  let unequipment = CommandRequest::Unequip {
    actor: ActorId::new(3),
  };
  assert_eq!(
    serde_json::to_value(unequipment).expect("unequip request should serialize"),
    serde_json::json!({"unequip": {"actor": 3}})
  );
  assert_eq!(
    serde_json::from_value::<CommandRequest>(serde_json::json!({
      "unequip": {"actor": 3}
    }))
    .expect("unequip request should deserialize"),
    unequipment
  );
  let consumption = CommandRequest::UseItem {
    actor: ActorId::new(3),
    item: ItemId::new(4),
  };
  assert_eq!(
    serde_json::to_value(consumption).expect("consumption request should serialize"),
    serde_json::json!({"use_item": {"actor": 3, "item": 4}})
  );
  assert_eq!(
    serde_json::from_value::<CommandRequest>(serde_json::json!({
      "use_item": {"actor": 3, "item": 4}
    }))
    .expect("consumption request should deserialize"),
    consumption
  );

  let event = Event::Waited {
    actor: ActorId::new(3),
    at: ActionTime::new(7),
  };
  assert_eq!(
    serde_json::to_value(event).expect("event should serialize"),
    serde_json::json!({"waited": {"actor": 3, "at": 7}})
  );
  let equipment_event = Event::ItemEquipped {
    actor: ActorId::new(3),
    item: dreadstep_protocol::ItemId::new(4),
  };
  assert_eq!(
    serde_json::to_value(equipment_event).expect("equipment event should serialize"),
    serde_json::json!({"item_equipped": {"actor": 3, "item": 4}})
  );
  assert_eq!(
    serde_json::from_value::<Event>(serde_json::json!({
      "item_equipped": {"actor": 3, "item": 4}
    }))
    .expect("equipped event should deserialize"),
    equipment_event
  );
  let unequipment_event = Event::ItemUnequipped {
    actor: ActorId::new(3),
    item: ItemId::new(4),
  };
  assert_eq!(
    serde_json::to_value(unequipment_event).expect("unequipped event should serialize"),
    serde_json::json!({"item_unequipped": {"actor": 3, "item": 4}})
  );
  assert_eq!(
    serde_json::from_value::<Event>(serde_json::json!({
      "item_unequipped": {"actor": 3, "item": 4}
    }))
    .expect("unequipped event should deserialize"),
    unequipment_event
  );
  let consumed_event = Event::ItemConsumed {
    actor: ActorId::new(3),
    item: ItemId::new(4),
  };
  assert_eq!(
    serde_json::to_value(consumed_event).expect("consumed event should serialize"),
    serde_json::json!({"item_consumed": {"actor": 3, "item": 4}})
  );
  assert_eq!(
    serde_json::from_value::<Event>(serde_json::json!({
      "item_consumed": {"actor": 3, "item": 4}
    }))
    .expect("consumed event should deserialize"),
    consumed_event
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
