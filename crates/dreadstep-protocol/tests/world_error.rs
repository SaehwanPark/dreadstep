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
  ];

  for (core, protocol) in errors {
    assert_eq!(WorldError::from(core), protocol);
  }
}
