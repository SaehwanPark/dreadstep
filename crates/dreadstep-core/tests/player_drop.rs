//! Behavioral evidence for the scheduled player-facing item drop contract.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap, HitPoints, Item,
  ItemDefinitionId, ItemId, Position, Tile, WorldState,
};

fn world_with_items() -> WorldState {
  let mut world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(10),
      ),
    ],
  )
  .expect("world should be valid");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("first item should be owned");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("second item should be owned");
  world
}

#[test]
fn drop_removes_inventory_item_appends_ground_item_and_consumes_one_action() {
  let mut world = world_with_items();
  let result = world
    .execute(Command::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("owned unequipped item should drop");

  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("player should remain")
      .inventory(),
    &[Item::new(ItemId::new(2), ItemDefinitionId::new(102))]
  );
  assert_eq!(
    world
      .ground_items()
      .iter()
      .flat_map(dreadstep_core::GroundItemStack::items)
      .copied()
      .collect::<Vec<_>>(),
    vec![Item::new(ItemId::new(1), ItemDefinitionId::new(101))]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("player")
      .ready_at()
      .value(),
    1
  );
  assert_eq!(
    result.events(),
    &[Event::ItemDropped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }]
  );
}

#[test]
fn legal_drop_order_excludes_equipped_items_and_rejection_is_atomic() {
  let mut world = world_with_items();
  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect("second item should equip");
  world
    .execute(Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy should take its scheduled turn");
  let legal = world.legal_commands();
  assert!(legal.contains(&Command::Drop {
    actor: ActorId::new(1),
    item: ItemId::new(1),
  }));
  assert!(!legal.contains(&Command::Drop {
    actor: ActorId::new(1),
    item: ItemId::new(2),
  }));

  let before = world.clone();
  let error = world
    .execute(Command::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect_err("equipped item should remain protected");
  assert_eq!(
    error,
    CommandError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    }
  );
  assert_eq!(world, before);
}

#[test]
fn drop_rejects_wrong_role_and_unscheduled_actor_atomically() {
  let mut enemy_world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![Actor::with_hit_points(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(0, 0),
      HitPoints::new(10),
    )],
  )
  .expect("enemy world should be valid");
  enemy_world
    .give_item(
      ActorId::new(2),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("enemy item should be owned");
  let before = enemy_world.clone();
  let error = enemy_world
    .execute(Command::Drop {
      actor: ActorId::new(2),
      item: ItemId::new(1),
    })
    .expect_err("enemy drop should reject");
  assert_eq!(error, CommandError::DropRequiresPlayer(ActorId::new(2)));
  assert_eq!(enemy_world, before);

  let mut world = world_with_items();
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should wait");
  let before = world.clone();
  let error = world
    .execute(Command::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect_err("unscheduled player should reject first");
  assert_eq!(
    error,
    CommandError::ActorNotScheduled {
      requested: ActorId::new(1),
      scheduled: ActorId::new(2),
    }
  );
  assert_eq!(world, before);
}

#[test]
fn drop_rejects_an_unowned_item_atomically() {
  let mut world = world_with_items();
  let before = world.clone();
  let error = world
    .execute(Command::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    })
    .expect_err("unowned item should reject");
  assert_eq!(
    error,
    CommandError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }
  );
  assert_eq!(world, before);
}
