//! Contract tests for the versioned Frost Flask throw boundary.

use dreadstep_core::{
  ActorId as CoreActorId, Command as CoreCommand, Event as CoreEvent, Item as CoreItem,
  ItemDefinitionId as CoreItemDefinitionId, ItemId as CoreItemId,
  ThrowableEffect as CoreThrowableEffect,
};
use dreadstep_protocol::{ActorId, CommandRequest, Event, ItemId, ThrowableEffect};

#[test]
fn throw_request_round_trips_and_uses_a_stable_tagged_json_shape() {
  let request = CommandRequest::Throw {
    actor: ActorId::new(1),
    item: ItemId::new(104),
    target: ActorId::new(2),
  };

  assert_eq!(
    CoreCommand::from(request),
    CoreCommand::Throw {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(104),
      target: CoreActorId::new(2),
    }
  );
  assert_eq!(CommandRequest::from(CoreCommand::from(request)), request);
  assert_eq!(
    serde_json::to_value(request).expect("throw request should serialize"),
    serde_json::json!({"throw": {"actor": 1, "item": 104, "target": 2}})
  );
}

#[test]
fn item_thrown_event_and_throwable_item_effect_project_without_adapter_policy() {
  assert_eq!(
    Event::from(CoreEvent::ItemThrown {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(104),
      target: CoreActorId::new(2),
    }),
    Event::ItemThrown {
      actor: ActorId::new(1),
      item: ItemId::new(104),
      target: ActorId::new(2),
    }
  );
  let thrown = Event::ItemThrown {
    actor: ActorId::new(1),
    item: ItemId::new(104),
    target: ActorId::new(2),
  };
  let thrown_json = serde_json::to_value(thrown).expect("item thrown should serialize");
  assert_eq!(
    thrown_json,
    serde_json::json!({"item_thrown": {"actor": 1, "item": 104, "target": 2}})
  );
  assert_eq!(
    serde_json::from_value::<Event>(thrown_json).expect("item thrown should deserialize"),
    thrown
  );

  let item = CoreItem::with_throwable_effect(
    CoreItemId::new(104),
    CoreItemDefinitionId::new(5),
    CoreThrowableEffect::Chill,
  );
  let snapshot = dreadstep_protocol::ItemSnapshot::from_item(item);
  assert_eq!(snapshot.throwable_effect(), Some(ThrowableEffect::Chill));
  assert_eq!(
    serde_json::to_value(snapshot).expect("throwable item should serialize"),
    serde_json::json!({
      "id": 104,
      "definition": 5,
      "rarity": "common",
      "affix": null,
      "equipment_effect": null,
      "equipment_slot": null,
      "throwable_effect": "chill"
    })
  );
}
