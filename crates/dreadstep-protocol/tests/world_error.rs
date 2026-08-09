//! Contract tests for protocol projection of core world validation errors.

use dreadstep_core::{
  ActorId as CoreActorId, Position as CorePosition, WorldError as CoreWorldError,
};
use dreadstep_protocol::{ActorId, Position, WorldError};

#[test]
fn every_core_world_error_maps_to_protocol_values() {
  let errors = [
    (
      CoreWorldError::UnknownActor(CoreActorId::new(9)),
      WorldError::UnknownActor(ActorId::new(9)),
    ),
    (
      CoreWorldError::DuplicateItemId(dreadstep_core::ItemId::new(7)),
      WorldError::DuplicateItemId(dreadstep_protocol::ItemId::new(7)),
    ),
    (
      CoreWorldError::DuplicateActorId(CoreActorId::new(1)),
      WorldError::DuplicateActorId(ActorId::new(1)),
    ),
    (
      CoreWorldError::ActorOutOfBounds {
        actor: CoreActorId::new(2),
        position: CorePosition::new(3, 0),
      },
      WorldError::ActorOutOfBounds {
        actor: ActorId::new(2),
        position: Position::new(3, 0),
      },
    ),
    (
      CoreWorldError::ActorOnBlockedTile {
        actor: CoreActorId::new(2),
        position: CorePosition::new(1, 0),
      },
      WorldError::ActorOnBlockedTile {
        actor: ActorId::new(2),
        position: Position::new(1, 0),
      },
    ),
    (
      CoreWorldError::OverlappingActors {
        first: CoreActorId::new(1),
        second: CoreActorId::new(2),
        position: CorePosition::new(0, 0),
      },
      WorldError::OverlappingActors {
        first: ActorId::new(1),
        second: ActorId::new(2),
        position: Position::new(0, 0),
      },
    ),
    (
      CoreWorldError::ActorDeadAtStart {
        actor: CoreActorId::new(2),
      },
      WorldError::ActorDeadAtStart {
        actor: ActorId::new(2),
      },
    ),
    (
      CoreWorldError::TeleportOutOfBounds {
        actor: CoreActorId::new(2),
        position: CorePosition::new(4, 0),
      },
      WorldError::TeleportOutOfBounds {
        actor: ActorId::new(2),
        position: Position::new(4, 0),
      },
    ),
    (
      CoreWorldError::TeleportOnBlockedTile {
        actor: CoreActorId::new(2),
        position: CorePosition::new(1, 1),
      },
      WorldError::TeleportOnBlockedTile {
        actor: ActorId::new(2),
        position: Position::new(1, 1),
      },
    ),
    (
      CoreWorldError::TeleportOccupied {
        actor: CoreActorId::new(1),
        blocker: CoreActorId::new(2),
        position: CorePosition::new(0, 0),
      },
      WorldError::TeleportOccupied {
        actor: ActorId::new(1),
        blocker: ActorId::new(2),
        position: Position::new(0, 0),
      },
    ),
  ];

  for (core, protocol) in errors {
    assert_eq!(WorldError::from(core), protocol);
  }
}
