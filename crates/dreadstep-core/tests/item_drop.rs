//! Deterministic core item-drop behavior.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, GroundItemStack, HitPoints, Item, ItemDefinitionId, ItemId,
  Position, Tile, WorldError, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::filled(3, 2, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(0, 0)),
      Actor::new(ActorId::new(3), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("world should be valid")
}

#[test]
fn drop_moves_items_into_ordered_ground_stacks_and_changes_digest() {
  let mut world = world();
  let first = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let second = Item::new(ItemId::new(2), ItemDefinitionId::new(20));
  world
    .give_item(ActorId::new(1), first)
    .expect("first item should be accepted");
  world
    .give_item(ActorId::new(1), second)
    .expect("second item should be accepted");
  let before = world.digest();

  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("owned item should drop");
  world
    .drop_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should drop");

  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor exists")
      .inventory(),
    &[]
  );
  assert_eq!(world.ground_items().len(), 1);
  assert_eq!(world.ground_items()[0].position(), Position::new(1, 1));
  assert_eq!(world.ground_items()[0].items(), &[first, second]);
  assert_ne!(world.digest(), before);
}

#[test]
fn ground_stacks_are_projected_in_row_major_position_order() {
  let mut world = world();
  let first = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let second = Item::new(ItemId::new(2), ItemDefinitionId::new(20));
  let third = Item::new(ItemId::new(3), ItemDefinitionId::new(30));
  world
    .give_item(ActorId::new(1), first)
    .expect("first item should be accepted");
  world
    .give_item(ActorId::new(2), second)
    .expect("second item should be accepted");
  world
    .give_item(ActorId::new(3), third)
    .expect("third item should be accepted");

  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("first item should drop");
  world
    .drop_item(ActorId::new(2), ItemId::new(2))
    .expect("second item should drop");
  world
    .drop_item(ActorId::new(3), ItemId::new(3))
    .expect("third item should drop");

  assert_eq!(
    world
      .ground_items()
      .iter()
      .map(GroundItemStack::position)
      .collect::<Vec<_>>(),
    vec![
      Position::new(0, 0),
      Position::new(2, 0),
      Position::new(1, 1)
    ]
  );
}

#[test]
fn dead_actor_records_remain_valid_drop_sources() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(2), item)
    .expect("item should be accepted");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should become dead");

  world
    .drop_item(ActorId::new(2), ItemId::new(1))
    .expect("dead actor should remain a valid source");

  let actor = world.actor(ActorId::new(2)).expect("dead record remains");
  assert!(!actor.is_alive());
  assert_eq!(actor.inventory(), &[]);
  assert_eq!(world.ground_items()[0].position(), Position::new(0, 0));
  assert_eq!(world.ground_items()[0].items(), &[item]);
}

#[test]
fn drop_rejections_are_typed_and_atomic_and_ground_ids_stay_unique() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  let before = world.clone();

  assert_eq!(
    world.drop_item(ActorId::new(9), ItemId::new(1)),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);
  assert_eq!(
    world.drop_item(ActorId::new(1), ItemId::new(2)),
    Err(WorldError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
  );
  assert_eq!(world, before);

  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("owned item should drop");
  assert_eq!(
    world.give_item(ActorId::new(2), item),
    Err(WorldError::DuplicateItemId(item.id()))
  );
}
