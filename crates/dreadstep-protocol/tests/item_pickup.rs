//! Protocol projection of deterministic item-pickup errors.

use dreadstep_core::{
  ActorId as CoreActorId, CommandError as CoreCommandError, ItemId as CoreItemId,
  WorldError as CoreWorldError,
};
use dreadstep_protocol::{ActorId, CommandError, ItemId, WorldError};

#[test]
fn item_not_on_ground_maps_to_typed_protocol_world_error() {
  assert_eq!(
    WorldError::from(CoreWorldError::ItemNotOnGround {
      actor: CoreActorId::new(2),
      item: CoreItemId::new(9),
    }),
    WorldError::ItemNotOnGround {
      actor: ActorId::new(2),
      item: ItemId::new(9),
    }
  );
}

#[test]
fn item_not_on_ground_maps_to_typed_protocol_command_error() {
  assert_eq!(
    CommandError::from(CoreCommandError::ItemNotOnGround {
      actor: CoreActorId::new(2),
      item: CoreItemId::new(9),
    }),
    CommandError::ItemNotOnGround {
      actor: ActorId::new(2),
      item: ItemId::new(9),
    }
  );
}

#[test]
fn pickup_requires_player_maps_to_typed_protocol_command_error() {
  assert_eq!(
    CommandError::from(CoreCommandError::PickupRequiresPlayer(CoreActorId::new(2))),
    CommandError::PickupRequiresPlayer(ActorId::new(2))
  );
}
