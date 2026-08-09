//! Deterministic core item-transfer behavior.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position, Tile,
  WorldError, WorldState,
};

fn world() -> WorldState {
  WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("world should be valid")
}

#[test]
fn transfer_moves_item_preserves_order_and_changes_digest() {
  let mut world = world();
  let first = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let second = Item::new(ItemId::new(2), ItemDefinitionId::new(20));
  let target_existing = Item::new(ItemId::new(3), ItemDefinitionId::new(30));
  world
    .give_item(ActorId::new(1), first)
    .expect("first item should be accepted");
  world
    .give_item(ActorId::new(1), second)
    .expect("second item should be accepted");
  world
    .give_item(ActorId::new(2), target_existing)
    .expect("target item should be accepted");
  let before = world.digest();

  world
    .transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(1))
    .expect("owned item should transfer");

  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("source exists")
      .inventory(),
    &[second]
  );
  assert_eq!(
    world
      .actor(ActorId::new(2))
      .expect("target exists")
      .inventory(),
    &[target_existing, first]
  );
  assert_ne!(world.digest(), before);
}

#[test]
fn same_actor_transfer_is_an_idempotent_noop() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  let before = world.clone();

  world
    .transfer_item(ActorId::new(1), ActorId::new(1), ItemId::new(1))
    .expect("same-actor transfer should be idempotent");

  assert_eq!(world, before);
}

#[test]
fn transfer_rejections_are_typed_and_atomic() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");

  let before = world.clone();
  assert_eq!(
    world.transfer_item(ActorId::new(9), ActorId::new(2), ItemId::new(1)),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);
  assert_eq!(
    world.transfer_item(ActorId::new(1), ActorId::new(9), ItemId::new(1)),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);
  assert_eq!(
    world.transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(2)),
    Err(WorldError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn dead_actor_records_remain_valid_transfer_endpoints() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should become dead");

  world
    .transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(1))
    .expect("dead actor should remain a valid target");

  let source = world.actor(ActorId::new(1)).expect("source remains");
  assert_eq!(source.inventory(), &[]);
  let target = world.actor(ActorId::new(2)).expect("dead record remains");
  assert!(!target.is_alive());
  assert_eq!(target.inventory(), &[item]);

  world
    .transfer_item(ActorId::new(2), ActorId::new(1), ItemId::new(1))
    .expect("dead actor should remain a valid source");

  let source = world.actor(ActorId::new(1)).expect("source remains");
  assert_eq!(source.inventory(), &[item]);
  let target = world.actor(ActorId::new(2)).expect("dead record remains");
  assert!(!target.is_alive());
  assert_eq!(target.inventory(), &[]);
}
