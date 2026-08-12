//! Deterministic fixed inventory-capacity contracts.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldError, WorldState,
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

fn item(id: u32) -> Item {
  Item::new(ItemId::new(id), ItemDefinitionId::new(id + 100))
}

fn fill_actor(world: &mut WorldState, actor: ActorId, start: u32) {
  for id in 1..=u32::try_from(Actor::INVENTORY_CAPACITY).expect("test capacity fits item ids") {
    world
      .give_item(actor, item(start + id))
      .expect("capacity-sized inventory should be accepted");
  }
}

#[test]
fn give_item_rejects_capacity_overflow_atomically() {
  let mut world = world();
  fill_actor(&mut world, ActorId::new(1), 0);
  let before = world.clone();

  assert_eq!(
    world.give_item(ActorId::new(1), item(99)),
    Err(WorldError::InventoryFull(ActorId::new(1)))
  );
  assert_eq!(world, before);
}

#[test]
fn pickup_rejects_full_inventory_without_removing_ground_item() {
  let mut world = world();
  fill_actor(&mut world, ActorId::new(1), 0);
  world
    .give_item(ActorId::new(2), item(99))
    .expect("enemy can hold the ground fixture");
  world
    .drop_item(ActorId::new(2), ItemId::new(99))
    .expect("enemy can drop the ground fixture");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("enemy can become a dead retained record");
  world
    .teleport(ActorId::new(1), Position::new(2, 0))
    .expect("living player can use the dead actor tile");
  let before = world.clone();

  assert_eq!(
    world.pickup_item(ActorId::new(1), ItemId::new(99)),
    Err(WorldError::InventoryFull(ActorId::new(1)))
  );
  assert_eq!(world, before);
}

#[test]
fn transfer_rejects_full_target_without_mutating_source_or_target() {
  let mut world = world();
  world
    .give_item(ActorId::new(1), item(99))
    .expect("source item should be accepted");
  fill_actor(&mut world, ActorId::new(2), 0);
  let before = world.clone();

  assert_eq!(
    world.transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(99)),
    Err(WorldError::InventoryFull(ActorId::new(2)))
  );
  assert_eq!(world, before);
}

#[test]
fn player_pickup_is_hidden_and_direct_command_rejects_full_inventory() {
  let mut world = world();
  fill_actor(&mut world, ActorId::new(1), 0);
  world
    .give_item(ActorId::new(2), item(99))
    .expect("enemy can hold the ground fixture");
  world
    .drop_item(ActorId::new(2), ItemId::new(99))
    .expect("enemy can drop the ground fixture");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("enemy can become dead");
  world
    .teleport(ActorId::new(1), Position::new(2, 0))
    .expect("player can use the dead actor tile");
  let before = world.clone();
  assert!(!world.legal_commands().iter().any(|command| {
    matches!(command, Command::Pickup { actor, item } if *actor == ActorId::new(1) && *item == ItemId::new(99))
  }));
  assert_eq!(
    world.execute(Command::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(CommandError::InventoryFull(ActorId::new(1)))
  );
  assert_eq!(world, before);
}
