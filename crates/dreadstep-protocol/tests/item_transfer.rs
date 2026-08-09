//! Protocol mapping for deterministic item transfer errors.

use dreadstep_core::{ActorId as CoreActorId, ItemId as CoreItemId, WorldError as CoreWorldError};
use dreadstep_protocol::{ActorId, ItemId, WorldError};

#[test]
fn item_not_owned_maps_to_typed_protocol_world_error() {
  assert_eq!(
    WorldError::from(CoreWorldError::ItemNotOwned {
      actor: CoreActorId::new(4),
      item: CoreItemId::new(9),
    }),
    WorldError::ItemNotOwned {
      actor: ActorId::new(4),
      item: ItemId::new(9),
    }
  );
}
