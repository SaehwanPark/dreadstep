//! Core equipment contract tests.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Event, GridMap, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn equipment_world() -> WorldState {
  let mut world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should validate"),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("item should be owned");
  world
}

#[test]
fn equip_and_unequip_preserve_inventory_and_emit_typed_events() {
  let mut world = equipment_world();
  let equipped = world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("owned item should equip");
  assert_eq!(
    equipped.events(),
    &[Event::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().equipped_item(),
    Some(ItemId::new(1))
  );
  assert_eq!(world.actor(ActorId::new(1)).unwrap().inventory().len(), 1);

  let unequipped = world
    .execute(Command::Unequip {
      actor: ActorId::new(1),
    })
    .expect("equipped item should unequip");
  assert_eq!(
    unequipped.events(),
    &[Event::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }]
  );
  assert_eq!(world.actor(ActorId::new(1)).unwrap().equipped_item(), None);
  assert_eq!(world.actor(ActorId::new(1)).unwrap().inventory().len(), 1);
}

#[test]
fn replacement_is_ordered_and_rejected_commands_are_atomic() {
  let mut world = equipment_world();
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("second item should be owned");
  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("first item should equip");

  let replaced = world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect("second owned item should replace the first");
  assert_eq!(
    replaced.events(),
    &[
      Event::ItemUnequipped {
        actor: ActorId::new(1),
        item: ItemId::new(1),
      },
      Event::ItemEquipped {
        actor: ActorId::new(1),
        item: ItemId::new(2),
      },
    ]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().equipped_item(),
    Some(ItemId::new(2))
  );

  let before = world.clone();
  let before_digest = world.digest();
  assert_eq!(
    world.execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    }),
    Err(dreadstep_core::CommandError::ItemAlreadyEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
  );
  assert_eq!(world, before);
  assert_eq!(world.digest(), before_digest);

  assert_eq!(
    world.execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(dreadstep_core::CommandError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    })
  );
  assert_eq!(world, before);
}

#[test]
fn equipped_items_cannot_be_moved_by_tester_mutations() {
  let mut world = equipment_world();
  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("item should equip");
  assert_eq!(
    world.drop_item(ActorId::new(1), ItemId::new(1)),
    Err(dreadstep_core::WorldError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
  );
  assert_eq!(
    world.transfer_item(ActorId::new(1), ActorId::new(1), ItemId::new(1)),
    Err(dreadstep_core::WorldError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
  );
}

#[test]
fn legal_commands_and_replay_digest_include_equipment_in_order() {
  let mut world = equipment_world();
  let before = world.digest();
  let commands = world.legal_commands();
  assert_eq!(
    commands,
    vec![
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::North,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::South,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::West,
      },
      Command::Move {
        actor: ActorId::new(1),
        direction: dreadstep_core::Direction::East,
      },
      Command::Wait {
        actor: ActorId::new(1),
      },
      Command::Equip {
        actor: ActorId::new(1),
        item: ItemId::new(1),
      },
    ]
  );

  let mut trace = dreadstep_core::ReplayTrace::new(7);
  trace.record(Command::Equip {
    actor: ActorId::new(1),
    item: ItemId::new(1),
  });
  let initial_trace = dreadstep_core::ReplayTrace::new(7).digest();
  assert_ne!(trace.digest(), initial_trace);

  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("owned item should equip");
  assert_ne!(world.digest(), before);
  assert_eq!(
    world.legal_commands().last().copied(),
    Some(Command::Unequip {
      actor: ActorId::new(1),
    })
  );
}

#[test]
fn empty_and_dead_equipment_commands_are_rejected_atomically() {
  let mut world = equipment_world();
  let before = world.clone();
  let before_digest = world.digest();
  assert_eq!(
    world.execute(Command::Unequip {
      actor: ActorId::new(1),
    }),
    Err(dreadstep_core::CommandError::NothingEquipped(ActorId::new(
      1
    )))
  );
  assert_eq!(world, before);
  assert_eq!(world.digest(), before_digest);

  world
    .set_hit_points(ActorId::new(1), dreadstep_core::HitPoints::new(0))
    .expect("tester should be able to retain a dead actor");
  let dead_before = world.clone();
  assert_eq!(
    world.execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }),
    Err(dreadstep_core::CommandError::ActorDead(ActorId::new(1)))
  );
  assert_eq!(world, dead_before);
}
