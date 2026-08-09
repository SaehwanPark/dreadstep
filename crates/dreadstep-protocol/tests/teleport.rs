//! Contract tests for protocol teleport values and error projection.

use dreadstep_core::{
  ActorId as CoreActorId, Position as CorePosition, WorldError as CoreWorldError,
};
use dreadstep_protocol::{ActorId, Position, WorldError};

#[test]
fn teleport_world_errors_map_to_typed_protocol_values() {
  let errors = [
    (
      CoreWorldError::TeleportOutOfBounds {
        actor: CoreActorId::new(1),
        position: CorePosition::new(4, 0),
      },
      WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position: Position::new(4, 0),
      },
    ),
    (
      CoreWorldError::TeleportOnBlockedTile {
        actor: CoreActorId::new(1),
        position: CorePosition::new(1, 1),
      },
      WorldError::TeleportOnBlockedTile {
        actor: ActorId::new(1),
        position: Position::new(1, 1),
      },
    ),
    (
      CoreWorldError::TeleportOccupied {
        actor: CoreActorId::new(1),
        blocker: CoreActorId::new(2),
        position: CorePosition::new(2, 0),
      },
      WorldError::TeleportOccupied {
        actor: ActorId::new(1),
        blocker: ActorId::new(2),
        position: Position::new(2, 0),
      },
    ),
  ];

  for (core, protocol) in errors {
    assert_eq!(WorldError::from(core), protocol);
  }
}
