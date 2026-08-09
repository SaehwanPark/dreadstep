//! Protocol projection of deterministic item-pickup errors.

use dreadstep_core::{ActorId as CoreActorId, ItemId as CoreItemId, WorldError as CoreWorldError};
use dreadstep_protocol::{ActorId, ItemId, WorldError};

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
