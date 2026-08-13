//! Contract tests for ranged command rejection conversion.

use dreadstep_core::{ActorId as CoreActorId, CommandError as CoreCommandError};
use dreadstep_protocol::{ActorId, CommandError, Position};
use schemars::schema_for;
use serde_json::json;

#[test]
fn ranged_out_of_range_error_maps_to_the_typed_protocol_variant() {
  assert_eq!(
    CommandError::from(CoreCommandError::RangedAttackOutOfRange {
      attacker: CoreActorId::new(1),
      target: CoreActorId::new(2),
    }),
    CommandError::RangedAttackOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
}

#[test]
fn ranged_no_line_of_sight_error_maps_to_the_typed_protocol_variant() {
  assert_eq!(
    CommandError::from(CoreCommandError::RangedAttackNoLineOfSight {
      attacker: CoreActorId::new(1),
      target: CoreActorId::new(2),
    }),
    CommandError::RangedAttackNoLineOfSight {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
}

#[test]
fn ranged_no_line_of_sight_error_has_a_tagged_json_and_schema_contract() {
  let error = CommandError::RangedAttackNoLineOfSight {
    attacker: ActorId::new(1),
    target: ActorId::new(2),
  };
  let value = serde_json::to_value(error).expect("command error should serialize");
  assert_eq!(
    value,
    json!({
      "ranged_attack_no_line_of_sight": {"attacker": 1, "target": 2}
    })
  );
  assert_eq!(
    serde_json::from_value::<CommandError>(value).expect("command error should deserialize"),
    error
  );

  let schema = serde_json::to_value(schema_for!(CommandError)).expect("schema should serialize");
  assert!(schema["oneOf"].is_array());
  assert!(
    schema
      .to_string()
      .contains("ranged_attack_no_line_of_sight")
  );
}

#[test]
fn ranged_no_ammunition_error_has_a_tagged_json_and_schema_contract() {
  let core = CoreCommandError::RangedAttackNoAmmunition(CoreActorId::new(1));
  let error = CommandError::from(core);
  assert_eq!(
    error,
    CommandError::RangedAttackNoAmmunition(ActorId::new(1))
  );
  let value = serde_json::to_value(error).expect("command error should serialize");
  assert_eq!(value, json!({"ranged_attack_no_ammunition": 1}));
  assert_eq!(
    serde_json::from_value::<CommandError>(value).expect("command error should deserialize"),
    error
  );
  let schema = serde_json::to_value(schema_for!(CommandError)).expect("schema should serialize");
  assert!(schema.to_string().contains("ranged_attack_no_ammunition"));
}

#[test]
fn throw_error_maps_to_a_typed_protocol_variant_and_stable_json() {
  let error = CommandError::from(CoreCommandError::ThrowOutOfRange {
    attacker: CoreActorId::new(1),
    target: CoreActorId::new(2),
  });
  assert_eq!(
    error,
    CommandError::ThrowOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
  let value = serde_json::to_value(error).expect("throw error should serialize");
  assert_eq!(
    value,
    json!({"throw_out_of_range": {"attacker": 1, "target": 2}})
  );
  assert_eq!(
    serde_json::from_value::<CommandError>(value).expect("throw error should deserialize"),
    error
  );
  let schema = serde_json::to_value(schema_for!(CommandError)).expect("schema should serialize");
  let schema_text = schema.to_string();
  assert!(schema_text.contains("throw_requires_player"));
  assert!(schema_text.contains("cannot_throw_self"));
  assert!(schema_text.contains("throw_out_of_range"));
  assert!(schema_text.contains("throw_no_line_of_sight"));
  assert!(schema_text.contains("item_not_throwable"));
}

#[test]
fn retreat_error_has_a_tagged_json_and_schema_contract() {
  let error = CommandError::RetreatNoEscape(ActorId::new(2));
  assert_eq!(
    serde_json::to_value(error).expect("error should serialize"),
    serde_json::json!({ "retreat_no_escape": 2 })
  );
  let schema = schemars::schema_for!(CommandError);
  let schema_json = serde_json::to_value(schema).expect("schema should serialize");
  assert!(schema_json.to_string().contains("retreat_no_escape"));
}

#[test]
fn non_throwable_item_error_is_typed() {
  assert_eq!(
    CommandError::from(CoreCommandError::ItemNotThrowable {
      actor: CoreActorId::new(1),
      item: dreadstep_core::ItemId::new(103),
    }),
    CommandError::ItemNotThrowable {
      actor: ActorId::new(1),
      item: dreadstep_protocol::ItemId::new(103),
    }
  );
}

#[test]
fn inventory_full_error_has_a_tagged_json_and_schema_contract() {
  let core = CoreCommandError::InventoryFull(CoreActorId::new(1));
  let error = CommandError::from(core);
  assert_eq!(error, CommandError::InventoryFull(ActorId::new(1)));
  let value = serde_json::to_value(error).expect("command error should serialize");
  assert_eq!(value, json!({"inventory_full": 1}));
  assert_eq!(
    serde_json::from_value::<CommandError>(value).expect("command error should deserialize"),
    error
  );
  let schema = serde_json::to_value(schema_for!(CommandError)).expect("schema should serialize");
  assert!(schema.to_string().contains("inventory_full"));
}

#[test]
fn interact_target_error_has_a_tagged_json_and_schema_contract() {
  let error = CommandError::from(CoreCommandError::InteractTargetInvalid {
    actor: CoreActorId::new(1),
    position: dreadstep_core::Position::new(2, 3),
  });
  assert_eq!(
    error,
    CommandError::InteractTargetInvalid {
      actor: ActorId::new(1),
      position: Position::new(2, 3),
    }
  );
  let value = serde_json::to_value(error).expect("command error should serialize");
  assert_eq!(
    value,
    json!({"interact_target_invalid": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );
  assert_eq!(
    serde_json::from_value::<CommandError>(value).expect("command error should deserialize"),
    error
  );
  let schema = serde_json::to_value(schema_for!(CommandError)).expect("schema should serialize");
  assert!(schema.to_string().contains("interact_target_invalid"));
}
