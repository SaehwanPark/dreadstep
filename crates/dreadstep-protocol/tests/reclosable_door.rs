//! Contract tests for the reclosable-door protocol boundary.

use dreadstep_core::{
  ActorId as CoreActorId, Command as CoreCommand, CommandError as CoreCommandError,
  Event as CoreEvent, Position as CorePosition,
};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event, Position, Scenario, Tile};

#[test]
fn close_request_and_door_closed_event_use_stable_wire_values() {
  let request = CommandRequest::Close {
    actor: ActorId::new(1),
    position: Position::new(2, 3),
  };
  assert_eq!(
    CoreCommand::from(request),
    CoreCommand::Close {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }
  );
  assert_eq!(CommandRequest::from(CoreCommand::from(request)), request);
  assert_eq!(
    serde_json::to_value(request).expect("close request should serialize"),
    serde_json::json!({"close": {"actor": 1, "position": {"x": 2, "y": 3}}})
  );
  assert_eq!(
    Event::from(CoreEvent::DoorClosed {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
    }),
    Event::DoorClosed {
      actor: ActorId::new(1),
      position: Position::new(2, 3),
    }
  );
}

#[test]
fn close_rejections_and_open_door_scenario_tile_are_typed() {
  assert_eq!(
    CommandError::from(CoreCommandError::DoorCloseOccupied {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 3),
      occupant: CoreActorId::new(2),
    }),
    CommandError::DoorCloseOccupied {
      actor: ActorId::new(1),
      position: Position::new(2, 3),
      occupant: ActorId::new(2),
    }
  );
  let scenario = Scenario::new(1, 1, vec![Tile::OpenDoor], Vec::new());
  assert_eq!(scenario.tiles(), &[Tile::OpenDoor]);
}
