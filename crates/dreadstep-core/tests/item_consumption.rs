//! Core single-item consumption contract tests.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap, Item, ItemDefinitionId, ItemId,
  Position, ReplayTrace, Tile, WorldState,
};

fn consumption_world() -> WorldState {
  let map = GridMap::filled(2, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  for (item, definition) in [(1, 101), (2, 102), (3, 103)] {
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(item), ItemDefinitionId::new(definition)),
      )
      .expect("item should be accepted");
  }
  world
}

#[test]
fn use_item_consumes_one_owned_instance_and_emits_typed_event() {
  let mut world = consumption_world();
  let before = world.digest();
  let result = world
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect("owned item should be consumed");

  assert_eq!(
    result.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor should remain")
      .inventory()
      .iter()
      .map(|item| item.id())
      .collect::<Vec<_>>(),
    vec![ItemId::new(1), ItemId::new(3)]
  );
  assert_ne!(world.digest(), before);
}

#[test]
fn use_item_rejects_unknown_and_equipped_instances_atomically() {
  let mut world = consumption_world();
  let before = world.clone();
  let before_digest = world.digest();
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(CommandError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    })
  );
  assert_eq!(world, before);
  assert_eq!(world.digest(), before_digest);

  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("item should equip");
  let equipped_before = world.clone();
  let equipped_digest = world.digest();
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }),
    Err(CommandError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
  );
  assert_eq!(world, equipped_before);
  assert_eq!(world.digest(), equipped_digest);
}

#[test]
fn legal_commands_and_replay_digest_include_each_consumable_identity() {
  let mut world = consumption_world();
  let legal = world.legal_commands();
  assert_eq!(
    legal
      .iter()
      .filter_map(|command| match command {
        Command::UseItem { item, .. } => Some(*item),
        _ => None,
      })
      .collect::<Vec<_>>(),
    vec![ItemId::new(1), ItemId::new(2), ItemId::new(3)]
  );

  world
    .execute(Command::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    })
    .expect("item should equip");
  assert!(!world.legal_commands().contains(&Command::UseItem {
    actor: ActorId::new(1),
    item: ItemId::new(1),
  }));

  let mut first = ReplayTrace::new(7);
  first.record(Command::UseItem {
    actor: ActorId::new(1),
    item: ItemId::new(2),
  });
  let mut second = ReplayTrace::new(7);
  second.record(Command::UseItem {
    actor: ActorId::new(1),
    item: ItemId::new(3),
  });
  assert_ne!(first.digest(), second.digest());
}
