//! Deterministic core item-pickup behavior.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldError, WorldState,
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
fn pickup_removes_middle_ground_item_and_appends_to_inventory() {
  let mut world = world();
  let carried = Item::new(ItemId::new(4), ItemDefinitionId::new(40));
  let first = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  let middle = Item::new(ItemId::new(2), ItemDefinitionId::new(20));
  let last = Item::new(ItemId::new(3), ItemDefinitionId::new(30));
  for item in [carried, first, middle, last] {
    world
      .give_item(ActorId::new(1), item)
      .expect("item should be accepted");
  }
  for item in [first, middle, last] {
    world
      .drop_item(ActorId::new(1), item.id())
      .expect("item should drop");
  }
  let before = world.digest();

  world
    .pickup_item(ActorId::new(1), ItemId::new(2))
    .expect("ground item should be picked up");

  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor exists")
      .inventory(),
    &[carried, middle]
  );
  assert_eq!(world.ground_items()[0].items(), &[first, last]);
  assert_ne!(world.digest(), before);
}

#[test]
fn pickup_removes_empty_ground_stack_after_last_item() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(1), item.id())
    .expect("item should drop");

  world
    .pickup_item(ActorId::new(1), item.id())
    .expect("ground item should be picked up");

  assert!(world.ground_items().is_empty());
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor exists")
      .inventory(),
    &[item]
  );
}

#[test]
fn dead_actor_records_remain_valid_pickup_sources() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(2), item)
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(2), item.id())
    .expect("item should drop");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("known actor should become dead");

  world
    .pickup_item(ActorId::new(2), item.id())
    .expect("dead actor should remain a valid source");

  let actor = world.actor(ActorId::new(2)).expect("dead record remains");
  assert!(!actor.is_alive());
  assert_eq!(actor.inventory(), &[item]);
  assert!(world.ground_items().is_empty());
}

#[test]
fn pickup_rejections_are_typed_and_atomic() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(1), item.id())
    .expect("item should drop");
  let before = world.clone();

  assert_eq!(
    world.pickup_item(ActorId::new(9), item.id()),
    Err(WorldError::UnknownActor(ActorId::new(9)))
  );
  assert_eq!(world, before);
  assert_eq!(
    world.pickup_item(ActorId::new(2), item.id()),
    Err(WorldError::ItemNotOnGround {
      actor: ActorId::new(2),
      item: item.id(),
    })
  );
  assert_eq!(world, before);
  assert_eq!(
    world.pickup_item(ActorId::new(1), ItemId::new(2)),
    Err(WorldError::ItemNotOnGround {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn scheduled_pickup_rejects_unscheduled_and_dead_actors_atomically() {
  let mut world = world();
  let item = Item::new(ItemId::new(1), ItemDefinitionId::new(10));
  world
    .give_item(ActorId::new(1), item)
    .expect("item should be accepted");
  world
    .drop_item(ActorId::new(1), item.id())
    .expect("item should drop");
  let before = world.clone();

  assert_eq!(
    world.execute(Command::Pickup {
      actor: ActorId::new(2),
      item: item.id(),
    }),
    Err(CommandError::ActorNotScheduled {
      requested: ActorId::new(2),
      scheduled: ActorId::new(1),
    })
  );
  assert_eq!(world, before);

  world
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("player should become dead");
  let dead_before = world.clone();
  assert_eq!(
    world.execute(Command::Pickup {
      actor: ActorId::new(1),
      item: item.id(),
    }),
    Err(CommandError::ActorDead(ActorId::new(1)))
  );
  assert_eq!(world, dead_before);
}
