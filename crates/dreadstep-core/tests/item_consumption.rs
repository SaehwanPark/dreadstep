//! Core single-item consumption contract tests.

use dreadstep_core::{
  ActionCost, Actor, ActorId, ActorKind, AmmunitionAmount, AmmunitionResult, Command, CommandError,
  Event, GridMap, HealingAmount, HealingResult, HitPoints, Item, ItemDefinitionId, ItemEffect,
  ItemId, Position, ReplayTrace, Tile, WorldState,
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
  let before_ready = world
    .actor(ActorId::new(1))
    .expect("actor should remain")
    .ready_at();
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
      healing: None,
      ammunition: None,
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
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor should remain")
      .ready_at()
      .value(),
    before_ready.value() + ActionCost::STANDARD.value()
  );
  assert_eq!(
    result.current_time().value(),
    before_ready.value() + ActionCost::STANDARD.value()
  );
  assert_ne!(world.digest(), before);
  let before_reuse = world.digest();
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(202)),
    )
    .expect("a consumed identity should be reusable by a tester");
  let actor = world.actor(ActorId::new(1)).expect("actor should remain");
  assert_eq!(
    actor
      .inventory()
      .iter()
      .map(|item| item.id())
      .collect::<Vec<_>>(),
    vec![ItemId::new(1), ItemId::new(3), ItemId::new(2)]
  );
  assert_eq!(
    actor.inventory()[2].definition(),
    ItemDefinitionId::new(202)
  );
  assert_ne!(world.digest(), before_reuse);
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
fn use_item_rejects_unscheduled_and_dead_actors_atomically() {
  let mut world = consumption_world();
  world
    .spawn(Actor::new(
      ActorId::new(2),
      ActorKind::Enemy,
      Position::new(1, 0),
    ))
    .expect("second actor should spawn");
  let before_unscheduled = world.clone();
  let before_unscheduled_digest = world.digest();
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(2),
      item: ItemId::new(1),
    }),
    Err(CommandError::ActorNotScheduled {
      requested: ActorId::new(2),
      scheduled: ActorId::new(1),
    })
  );
  assert_eq!(world, before_unscheduled);
  assert_eq!(world.digest(), before_unscheduled_digest);

  world
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("scheduled actor should become a retained dead record");
  let before_dead = world.clone();
  let before_dead_digest = world.digest();
  assert_eq!(
    world.execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    }),
    Err(CommandError::ActorDead(ActorId::new(1)))
  );
  assert_eq!(world, before_dead);
  assert_eq!(world.digest(), before_dead_digest);
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

#[test]
fn healing_consumable_is_capped_and_reports_actual_recovery() {
  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  let amount = HealingAmount::new(5).expect("positive healing should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::with_effect(
        ItemId::new(9),
        ItemDefinitionId::new(909),
        ItemEffect::Heal { amount },
      ),
    )
    .expect("healing item should be accepted");
  world
    .set_hit_points(ActorId::new(1), HitPoints::new(8))
    .expect("tester damage should be accepted");

  let result = world
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(9),
    })
    .expect("healing item should be consumed");

  assert_eq!(
    result.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(9),
      healing: Some(HealingResult::new(2, HitPoints::new(10))),
      ammunition: None,
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor should remain")
      .hit_points(),
    HitPoints::new(10)
  );

  world
    .give_item(
      ActorId::new(1),
      Item::with_effect(
        ItemId::new(10),
        ItemDefinitionId::new(910),
        ItemEffect::Heal { amount },
      ),
    )
    .expect("a second healing item should be accepted");
  let full_health = world
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(10),
    })
    .expect("full-health healing item should still be consumed");
  assert_eq!(
    full_health.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(10),
      healing: Some(HealingResult::new(0, HitPoints::new(10))),
      ammunition: None,
    }]
  );
}

#[test]
fn ammunition_consumable_is_capped_and_reports_actual_recovery() {
  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::with_ranged_ammo(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
      HitPoints::new(10),
      1,
    )],
  )
  .expect("world should validate");
  let amount = AmmunitionAmount::new(4).expect("positive ammunition should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::with_effect(
        ItemId::new(11),
        ItemDefinitionId::new(911),
        ItemEffect::RestoreAmmunition { amount },
      ),
    )
    .expect("ammunition item should be accepted");

  let result = world
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(11),
    })
    .expect("ammunition item should be consumed");

  assert_eq!(
    result.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(11),
      healing: None,
      ammunition: Some(AmmunitionResult::new(2, 3)),
    }]
  );
  assert_eq!(
    world
      .actor(ActorId::new(1))
      .expect("actor should remain")
      .ranged_ammo(),
    3
  );

  world
    .give_item(
      ActorId::new(1),
      Item::with_effect(
        ItemId::new(12),
        ItemDefinitionId::new(912),
        ItemEffect::RestoreAmmunition { amount },
      ),
    )
    .expect("a second ammunition item should be accepted");
  let full_ammunition = world
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(12),
    })
    .expect("full-ammunition item should still be consumed");
  assert_eq!(
    full_ammunition.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(12),
      healing: None,
      ammunition: Some(AmmunitionResult::new(0, 3)),
    }]
  );
}
