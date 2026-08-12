//! Protocol evidence for the deterministic ranged reload contract.

use dreadstep_core::{ActorId as CoreActorId, Command as CoreCommand, Event as CoreEvent};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event};
use serde_json::to_value;

#[test]
fn reload_request_round_trips_through_core() {
  let request = CommandRequest::Reload {
    actor: ActorId::new(1),
  };
  let core = CoreCommand::from(request);
  assert_eq!(
    core,
    CoreCommand::Reload {
      actor: CoreActorId::new(1)
    }
  );
  assert_eq!(CommandRequest::from(core), request);
}

#[test]
fn reload_event_and_errors_have_typed_json_shapes() {
  assert_eq!(
    Event::from(CoreEvent::Reloaded {
      actor: CoreActorId::new(1),
      ammunition: 3,
    }),
    Event::Reloaded {
      actor: ActorId::new(1),
      ammunition: 3,
    }
  );
  assert_eq!(
    to_value(CommandRequest::Reload {
      actor: ActorId::new(1),
    })
    .expect("request should serialize"),
    serde_json::json!({ "reload": { "actor": 1 } })
  );
  assert_eq!(
    to_value(CommandError::ReloadNotNeeded(ActorId::new(1))).expect("error should serialize"),
    serde_json::json!({ "reload_not_needed": 1 })
  );
}
