//! Protocol evidence for the scheduled player-facing item drop contract.

use dreadstep_core::{ActorId as CoreActorId, Command as CoreCommand, Event as CoreEvent};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event, ItemId};
use serde_json::to_value;

#[test]
fn drop_request_round_trips_through_core() {
  let request = CommandRequest::Drop {
    actor: ActorId::new(1),
    item: ItemId::new(101),
  };
  let core = CoreCommand::from(request);
  assert_eq!(
    core,
    CoreCommand::Drop {
      actor: CoreActorId::new(1),
      item: dreadstep_core::ItemId::new(101),
    }
  );
  assert_eq!(CommandRequest::from(core), request);
}

#[test]
fn drop_event_and_errors_have_typed_json_shapes() {
  assert_eq!(
    Event::from(CoreEvent::ItemDropped {
      actor: CoreActorId::new(1),
      item: dreadstep_core::ItemId::new(101),
    }),
    Event::ItemDropped {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    }
  );
  assert_eq!(
    to_value(CommandRequest::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
    .expect("request should serialize"),
    serde_json::json!({ "drop": { "actor": 1, "item": 101 } })
  );
  assert_eq!(
    to_value(CommandError::DropRequiresPlayer(ActorId::new(2))).expect("error should serialize"),
    serde_json::json!({ "drop_requires_player": 2 })
  );
}
