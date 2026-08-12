//! Contract tests for the protocol breakable-terrain boundary.

use dreadstep_core::{
  ActorId as CoreActorId, Command as CoreCommand, CommandError as CoreCommandError,
  Event as CoreEvent, Position as CorePosition,
};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event, Position};

#[test]
fn break_command_event_and_error_round_trip_with_stable_json() {
  let request = CommandRequest::Break {
    actor: ActorId::new(1),
    position: Position::new(2, 3),
  };
  assert_eq!(
    CoreCommand::from(request),
    CoreCommand::Break {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }
  );
  assert_eq!(
    CommandRequest::from(CoreCommand::Break {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }),
    request
  );
  assert_eq!(
    serde_json::to_value(request).expect("break request should serialize"),
    serde_json::json!({"break": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );

  let event = Event::from(CoreEvent::BreakableBroken {
    actor: CoreActorId::new(1),
    position: CorePosition::new(2, 3),
  });
  assert_eq!(
    event,
    Event::BreakableBroken {
      actor: ActorId::new(1),
      position: Position::new(2, 3),
    }
  );
  assert_eq!(
    serde_json::to_value(event).expect("break event should serialize"),
    serde_json::json!({"breakable_broken": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );

  let error = CommandError::from(CoreCommandError::BreakTargetInvalid {
    actor: CoreActorId::new(1),
    position: CorePosition::new(2, 3),
  });
  assert_eq!(
    serde_json::to_value(error).expect("break error should serialize"),
    serde_json::json!({"break_target_invalid": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );
}
