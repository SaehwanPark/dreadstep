//! Protocol projection of deterministic ground-item state.

use dreadstep_core::{
  Actor, ActorId as CoreActorId, ActorKind, GridMap, Item, ItemDefinitionId as CoreDefinitionId,
  ItemId as CoreItemId, Position as CorePosition, Tile, WorldState,
};
use dreadstep_protocol::{ItemDefinitionId, ItemId, Position, WorldSnapshot};

#[test]
fn world_snapshot_projects_ground_items_in_row_major_order() {
  let mut world = WorldState::new(
    GridMap::filled(3, 2, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(
        CoreActorId::new(1),
        ActorKind::Player,
        CorePosition::new(1, 1),
      ),
      Actor::new(
        CoreActorId::new(2),
        ActorKind::Enemy,
        CorePosition::new(0, 0),
      ),
      Actor::new(
        CoreActorId::new(3),
        ActorKind::Enemy,
        CorePosition::new(2, 0),
      ),
    ],
  )
  .expect("world should be valid");
  let first = Item::new(CoreItemId::new(1), CoreDefinitionId::new(10));
  let second = Item::new(CoreItemId::new(2), CoreDefinitionId::new(20));
  let third = Item::new(CoreItemId::new(3), CoreDefinitionId::new(30));
  world
    .give_item(CoreActorId::new(1), first)
    .expect("first item should be accepted");
  world
    .give_item(CoreActorId::new(2), second)
    .expect("second item should be accepted");
  world
    .give_item(CoreActorId::new(3), third)
    .expect("third item should be accepted");
  world
    .drop_item(CoreActorId::new(1), CoreItemId::new(1))
    .expect("first item should drop");
  world
    .drop_item(CoreActorId::new(2), CoreItemId::new(2))
    .expect("second item should drop");
  world
    .drop_item(CoreActorId::new(3), CoreItemId::new(3))
    .expect("third item should drop");

  let snapshot = WorldSnapshot::from_world(&world);
  assert_eq!(snapshot.ground_items().len(), 3);
  assert_eq!(snapshot.ground_items()[0].position(), Position::new(0, 0));
  assert_eq!(snapshot.ground_items()[0].items().len(), 1);
  assert_eq!(snapshot.ground_items()[0].items()[0].id(), ItemId::new(2));
  assert_eq!(
    snapshot.ground_items()[0].items()[0].definition(),
    ItemDefinitionId::new(20)
  );
  assert_eq!(snapshot.ground_items()[1].position(), Position::new(2, 0));
  assert_eq!(snapshot.ground_items()[1].items()[0].id(), ItemId::new(3));
  assert_eq!(
    snapshot.ground_items()[1].items()[0].definition(),
    ItemDefinitionId::new(30)
  );
  assert_eq!(snapshot.ground_items()[2].position(), Position::new(1, 1));
  assert_eq!(snapshot.ground_items()[2].items()[0].id(), ItemId::new(1));
  assert_eq!(
    snapshot.ground_items()[2].items()[0].definition(),
    ItemDefinitionId::new(10)
  );
}
