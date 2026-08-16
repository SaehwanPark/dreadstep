//! Contract tests for opaque core item ownership.

use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, Item, ItemDefinitionId, ItemId, ItemRarity, Position, Tile,
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
fn give_item_preserves_insertion_order_and_changes_the_world_digest() {
  let mut world = world();
  let before = world.digest();
  let first = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let second = Item::new(ItemId::new(2), ItemDefinitionId::new(20));

  world
    .give_item(ActorId::new(1), first)
    .expect("first item should be accepted");
  world
    .give_item(ActorId::new(1), second)
    .expect("second item should be accepted");

  let inventory = world
    .actor(ActorId::new(1))
    .expect("actor should exist")
    .inventory();
  assert_eq!(inventory, &[first, second]);
  assert_ne!(world.digest(), before);
}

#[test]
fn item_rarity_defaults_to_common_and_changes_digest_without_changing_effects() {
  let common = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let magic = common.with_rarity(ItemRarity::Magic);
  let rare = common.with_rarity(ItemRarity::Rare);

  assert_eq!(common.rarity(), ItemRarity::Common);
  assert_eq!(magic.rarity(), ItemRarity::Magic);
  assert_eq!(rare.rarity(), ItemRarity::Rare);
  assert_eq!(common.effect(), magic.effect());
  assert_ne!(common, magic);

  let mut common_world = world();
  common_world
    .give_item(ActorId::new(1), common)
    .expect("common item should be accepted");
  let mut magic_world = world();
  magic_world
    .give_item(ActorId::new(1), magic)
    .expect("magic item should be accepted");
  assert_ne!(common_world.digest(), magic_world.digest());
}

#[test]
fn give_item_rejects_unknown_actor_and_duplicate_identity_atomically() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("first item should be accepted");
  let before = world.clone();

  assert_eq!(
    world.give_item(ActorId::new(2), item),
    Err(WorldError::DuplicateItemId(ItemId::new(1)))
  );
  assert_eq!(world, before);

  assert_eq!(
    world.give_item(
      ActorId::new(9),
      Item::new(ItemId::new(2), ItemDefinitionId::new(20))
    ),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);
}

#[test]
fn dead_actor_records_remain_valid_item_ownership_targets() {
  let mut world = world();
  world
    .set_hit_points(ActorId::new(2), dreadstep_core::HitPoints::new(0))
    .expect("known actor should be updated");
  let item = Item::new(ItemId::new(3), ItemDefinitionId::new(30));
  world
    .give_item(ActorId::new(2), item)
    .expect("dead record should own opaque items");

  let actor = world
    .actor(ActorId::new(2))
    .expect("dead record should remain");
  assert!(!actor.is_alive());
  assert_eq!(actor.inventory(), &[item]);
}
