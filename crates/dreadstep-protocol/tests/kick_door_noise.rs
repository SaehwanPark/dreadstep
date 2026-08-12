//! Contract tests for protocol kick-open-door and noise evidence.

use dreadstep_core::{
  ActorId as CoreActorId, Command as CoreCommand, CommandError as CoreCommandError,
  Event as CoreEvent, Position as CorePosition,
};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event, Position};

#[test]
fn kick_command_noise_event_and_error_round_trip_with_stable_json() {
  let request = CommandRequest::Kick {
    actor: ActorId::new(1),
    position: Position::new(2, 3),
  };
  assert_eq!(
    CoreCommand::from(request),
    CoreCommand::Kick {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }
  );
  assert_eq!(
    CommandRequest::from(CoreCommand::Kick {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }),
    request
  );
  assert_eq!(
    serde_json::to_value(request).expect("kick request should serialize"),
    serde_json::json!({"kick": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );

  let event = Event::from(CoreEvent::NoiseCreated {
    actor: CoreActorId::new(1),
    position: CorePosition::new(2, 3),
    radius: 3,
  });
  assert_eq!(
    event,
    Event::NoiseCreated {
      actor: ActorId::new(1),
      position: Position::new(2, 3),
      radius: 3,
    }
  );
  assert_eq!(
    serde_json::to_value(event).expect("noise event should serialize"),
    serde_json::json!({"noise_created": {"actor": 1, "position": {"x": 2, "y": 3}, "radius": 3}})
  );

  let error = CommandError::from(CoreCommandError::KickTargetInvalid {
    actor: CoreActorId::new(1),
    position: CorePosition::new(2, 3),
  });
  assert_eq!(
    serde_json::to_value(error).expect("kick error should serialize"),
    serde_json::json!({"kick_target_invalid": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );
}
