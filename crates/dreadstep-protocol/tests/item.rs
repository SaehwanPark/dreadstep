//! Contract tests for protocol item identity and snapshot projection.

use dreadstep_core::{
  Actor as CoreActor, ActorId as CoreActorId, ActorKind as CoreActorKind, GridMap as CoreGridMap,
  Item as CoreItem, ItemDefinitionId as CoreItemDefinitionId, ItemId as CoreItemId,
  Position as CorePosition, Tile as CoreTile, WorldError as CoreWorldError,
  WorldState as CoreWorldState,
};
use dreadstep_protocol::{
  ActorId, ActorSnapshot, ItemDefinitionId, ItemId, Position, WorldError, WorldSnapshot,
};

fn item_world() -> CoreWorldState {
  CoreWorldState::new(
    CoreGridMap::filled(1, 1, CoreTile::Floor).expect("map should be valid"),
    vec![CoreActor::new(
      CoreActorId::new(1),
      CoreActorKind::Player,
      CorePosition::new(0, 0),
    )],
  )
  .expect("world should be valid")
}

#[test]
fn item_ids_are_typed_and_duplicate_identity_maps_to_protocol_world_error() {
  let item = ItemId::new(7);
  let definition = ItemDefinitionId::new(11);
  assert_eq!(item.value(), 7);
  assert_eq!(definition.value(), 11);
  assert_eq!(
    WorldError::from(CoreWorldError::DuplicateItemId(CoreItemId::new(7))),
    WorldError::DuplicateItemId(ItemId::new(7))
  );
  assert_eq!(
    WorldError::from(CoreWorldError::UnknownActor(CoreActorId::new(9))),
    WorldError::UnknownActor(ActorId::new(9))
  );
}

#[test]
fn actor_snapshot_projects_owned_items_in_insertion_order() {
  let mut world = item_world();
  world
    .give_item(
      CoreActorId::new(1),
      CoreItem::new(CoreItemId::new(1), CoreItemDefinitionId::new(10)),
    )
    .expect("item should be accepted");

  let snapshot = WorldSnapshot::from_world(&world);
  let actor: &ActorSnapshot = &snapshot.actors()[0];
  assert_eq!(actor.id(), ActorId::new(1));
  assert_eq!(actor.inventory().len(), 1);
  assert_eq!(actor.inventory()[0].id(), ItemId::new(1));
  assert_eq!(actor.inventory()[0].definition(), ItemDefinitionId::new(10));
  assert_eq!(actor.position(), Position::new(0, 0));
}

#[test]
fn equivalent_ordered_inventories_have_equal_digests_and_snapshots() {
  let mut first = item_world();
  let mut second = item_world();
  let first_item = CoreItem::new(CoreItemId::new(1), CoreItemDefinitionId::new(10));
  let second_item = CoreItem::new(CoreItemId::new(2), CoreItemDefinitionId::new(20));

  for world in [&mut first, &mut second] {
    world
      .give_item(CoreActorId::new(1), first_item)
      .expect("first item should be accepted");
    world
      .give_item(CoreActorId::new(1), second_item)
      .expect("second item should be accepted");
  }

  assert_eq!(first.digest(), second.digest());
  assert_eq!(
    WorldSnapshot::from_world(&first),
    WorldSnapshot::from_world(&second)
  );

  let mut reversed = item_world();
  reversed
    .give_item(CoreActorId::new(1), second_item)
    .expect("second item should be accepted");
  reversed
    .give_item(CoreActorId::new(1), first_item)
    .expect("first item should be accepted");
  assert_ne!(first.digest(), reversed.digest());
  assert_ne!(
    WorldSnapshot::from_world(&first),
    WorldSnapshot::from_world(&reversed)
  );
}
