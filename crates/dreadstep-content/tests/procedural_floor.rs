//! Seeded procedural-floor content invariants.

use dreadstep_content::{procedural_floor, procedural_floor_definition};
use dreadstep_core::{ActorId, ActorKind, EnemyBehavior, ItemRarity, Position, Tile};

#[test]
fn identical_seed_and_depth_produce_identical_valid_worlds() {
  let first = procedural_floor(7, 1).expect("generated floor should validate");
  let second = procedural_floor(7, 1).expect("generated floor should validate");

  assert_eq!(first.digest(), second.digest());
  assert_eq!(first.map(), second.map());
  assert_eq!(
    first
      .actors()
      .map(dreadstep_core::Actor::position)
      .collect::<Vec<_>>(),
    second
      .actors()
      .map(dreadstep_core::Actor::position)
      .collect::<Vec<_>>()
  );
}

#[test]
fn procedural_floor_places_two_equipment_items_and_one_consumable_in_player_inventory() {
  let world = procedural_floor(7, 1).expect("generated floor should validate");
  let player = world
    .actor(ActorId::new(1))
    .expect("generated player should exist");
  let [first, second, consumable] = player.inventory() else {
    panic!("procedural floor should provide two equipment items and one consumable");
  };

  for item in [first, second] {
    assert!(item.id().value() & 0x8000_0000 != 0);
    assert!(item.equipment_slot().is_some());
    assert!(item.equipment_effect().is_some());
    assert!(item.affix().is_some());
  }
  assert_ne!(first.id(), second.id());
  assert!(consumable.equipment_effect().is_none());
  assert!(consumable.equipment_slot().is_none());
  assert!(matches!(
    consumable.effect(),
    dreadstep_core::ItemEffect::Heal { .. } | dreadstep_core::ItemEffect::RestoreAmmunition { .. }
  ));
  assert!(consumable.affix().is_none());
}

#[test]
fn procedural_loadout_choices_have_distinct_equipment_effects() {
  for seed in 0..32 {
    for depth in [0, 1, 2, 3, 6] {
      let world = procedural_floor(seed, depth).expect("generated floor should validate");
      let player = world
        .actor(ActorId::new(1))
        .expect("generated player should exist");
      let [first, second, ..] = player.inventory() else {
        panic!("procedural floor should provide two equipment choices");
      };

      assert_ne!(
        first.equipment_effect(),
        second.equipment_effect(),
        "seed {seed} depth {depth} should provide distinct equipment effects"
      );
    }
  }
}

#[test]
fn procedural_floor_places_two_seeded_equipment_items_on_the_ground() {
  let world = procedural_floor(7, 1).expect("generated floor should validate");
  let [first_stack, second_stack] = world.ground_items() else {
    panic!("procedural floor should provide exactly two ground stacks");
  };
  let [first_item] = first_stack.items() else {
    panic!("first procedural ground stack should provide exactly one item");
  };
  let [second_item] = second_stack.items() else {
    panic!("second procedural ground stack should provide exactly one item");
  };

  assert_eq!(first_stack.position(), Position::new(11, 1));
  assert_eq!(second_stack.position(), Position::new(11, 7));
  assert_ne!(first_item.id(), second_item.id());
  for item in [first_item, second_item] {
    assert!(item.id().value() & 0x8000_0000 != 0);
    assert!(item.equipment_slot().is_some());
    assert!(item.affix().is_some());
    assert!(
      !world
        .actor(ActorId::new(1))
        .expect("generated player should exist")
        .inventory()
        .iter()
        .any(|owned| owned.id() == item.id())
    );
  }
}

#[test]
fn procedural_item_identity_changes_with_seed_or_depth() {
  let first = procedural_floor(7, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[0]
    .id();
  let different_seed = procedural_floor(8, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[0]
    .id();
  let deeper = procedural_floor(7, 4)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[0]
    .id();

  assert_ne!(first, different_seed);
  assert_ne!(first, deeper);
}

#[test]
fn generated_affix_amount_is_seeded_and_bounded() {
  let first = procedural_floor(7, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[0]
    .affix()
    .expect("generated equipment should carry an affix")
    .amount()
    .value();
  let different_seed = procedural_floor(8, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[0]
    .affix()
    .expect("generated equipment should carry an affix")
    .amount()
    .value();

  assert!((1..=2).contains(&first));
  assert!((1..=2).contains(&different_seed));
  assert_ne!(first, different_seed);
}

#[test]
fn procedural_affix_magnitude_respects_depth_floor() {
  let shallow = procedural_floor(7, 1).expect("generated floor should validate");
  assert_eq!(
    shallow
      .actor(ActorId::new(1))
      .expect("generated player should exist")
      .inventory()[0]
      .affix()
      .expect("generated equipment should carry an affix")
      .amount()
      .value(),
    1
  );

  let deep = procedural_floor(7, 3).expect("generated floor should validate");
  let player = deep
    .actor(ActorId::new(1))
    .expect("generated player should exist");
  assert!(player.inventory()[..2].iter().all(|item| {
    item
      .affix()
      .expect("generated equipment should carry an affix")
      .amount()
      .value()
      == 2
  }));
  assert!(
    deep
      .ground_items()
      .iter()
      .flat_map(dreadstep_core::GroundItemStack::items)
      .all(|item| {
        item
          .affix()
          .expect("generated ground equipment should carry an affix")
          .amount()
          .value()
          == 2
      })
  );
}

#[test]
fn generated_consumable_potency_is_seeded_and_bounded() {
  let ammunition_first = procedural_floor(0, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();
  let ammunition_different_seed = procedural_floor(1, 1)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();
  let healing_first = procedural_floor(0, 2)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();
  let healing_different_seed = procedural_floor(1, 2)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();

  let potency = |effect| match effect {
    dreadstep_core::ItemEffect::Heal { amount } => ("healing", amount.value()),
    dreadstep_core::ItemEffect::RestoreAmmunition { amount } => ("ammunition", amount.value()),
    other @ dreadstep_core::ItemEffect::None => {
      panic!("expected a generated consumable effect, got {other:?}")
    }
  };
  let (ammunition_kind, ammunition_first_potency) = potency(ammunition_first);
  let (ammunition_different_kind, ammunition_different_seed_potency) =
    potency(ammunition_different_seed);
  let (healing_kind, healing_first_potency) = potency(healing_first);
  let (healing_different_kind, healing_different_seed_potency) = potency(healing_different_seed);

  assert_eq!(ammunition_kind, ammunition_different_kind);
  assert_eq!(healing_kind, healing_different_kind);
  assert!((1..=2).contains(&ammunition_first_potency));
  assert!((1..=2).contains(&ammunition_different_seed_potency));
  assert!((1..=2).contains(&healing_first_potency));
  assert!((1..=2).contains(&healing_different_seed_potency));
  assert_ne!(ammunition_first_potency, ammunition_different_seed_potency);
  assert_ne!(healing_first_potency, healing_different_seed_potency);

  let deep_ammunition = procedural_floor(0, 3)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();
  let deep_healing = procedural_floor(0, 4)
    .expect("generated floor should validate")
    .actor(ActorId::new(1))
    .expect("generated player should exist")
    .inventory()[2]
    .effect();
  let (deep_ammunition_kind, deep_ammunition_potency) = potency(deep_ammunition);
  let (deep_healing_kind, deep_healing_potency) = potency(deep_healing);
  assert_eq!(deep_ammunition_kind, "ammunition");
  assert_eq!(deep_healing_kind, "healing");
  assert_eq!(deep_ammunition_potency, 2);
  assert_eq!(deep_healing_potency, 2);
}

#[test]
fn procedural_rarity_respects_depth_floor() {
  let shallow = procedural_floor(7, 1).expect("generated floor should validate");
  assert_eq!(
    shallow.ground_items()[0].items()[0].rarity(),
    ItemRarity::Common,
    "shallow floors should retain the common rarity mix"
  );

  let deep = procedural_floor(7, 3).expect("generated floor should validate");
  let player = deep
    .actor(ActorId::new(1))
    .expect("generated player should exist");
  assert!(
    player
      .inventory()
      .iter()
      .all(|item| matches!(item.rarity(), ItemRarity::Magic | ItemRarity::Rare))
  );
  assert!(
    deep
      .ground_items()
      .iter()
      .flat_map(dreadstep_core::GroundItemStack::items)
      .all(|item| matches!(item.rarity(), ItemRarity::Magic | ItemRarity::Rare))
  );
}

#[test]
fn seed_and_depth_change_the_generated_floor_without_invalidating_it() {
  let first = procedural_floor(7, 1).expect("generated floor should validate");
  let different_seed = procedural_floor(8, 1).expect("generated floor should validate");
  let deeper = procedural_floor(7, 4).expect("generated floor should validate");

  assert_ne!(first.map().tiles(), different_seed.map().tiles());
  assert_ne!(
    first
      .actors()
      .map(dreadstep_core::Actor::hit_points)
      .collect::<Vec<_>>(),
    deeper
      .actors()
      .map(dreadstep_core::Actor::hit_points)
      .collect::<Vec<_>>()
  );
  assert!(deeper.actors().all(dreadstep_core::Actor::is_alive));
}

#[test]
fn generated_layout_has_perimeter_and_three_single_gap_partitions() {
  let world = procedural_floor(19, 2).expect("generated floor should validate");

  assert_eq!(world.map().width(), 13);
  assert_eq!(world.map().height(), 9);
  for y in 0..9 {
    for x in 0..13 {
      let position = Position::new(x, y);
      let tile = world
        .map()
        .tile_at(position)
        .expect("position is in bounds");
      if x == 0 || x == 12 || y == 0 || y == 8 {
        assert_eq!(tile, Tile::Wall, "perimeter at ({x}, {y})");
      }
    }
  }

  for x in [3, 6, 9] {
    let gaps = (1..=7)
      .filter(|y| world.map().tile_at(Position::new(x, *y)) == Some(Tile::Floor))
      .count();
    assert_eq!(gaps, 1, "partition x={x} should have one gap");
  }
}

#[test]
fn generated_actor_roster_is_stable_and_walkable() {
  let world = procedural_floor_definition(7, 1)
    .build()
    .expect("generated definition should validate");
  let actors: Vec<_> = world.actors().collect();

  assert_eq!(
    actors.iter().map(|actor| actor.id()).collect::<Vec<_>>(),
    vec![
      ActorId::new(1),
      ActorId::new(2),
      ActorId::new(3),
      ActorId::new(4),
    ]
  );
  assert_eq!(actors[0].kind(), ActorKind::Player);
  assert!(
    actors[1..]
      .iter()
      .all(|actor| actor.kind() == ActorKind::Enemy)
  );
  assert!(
    actors
      .iter()
      .all(|actor| world.map().is_walkable(actor.position()))
  );
  assert_eq!(actors[3].enemy_behavior(), EnemyBehavior::Kiter);
  assert_eq!(
    actors
      .iter()
      .map(|actor| actor.position())
      .collect::<Vec<_>>(),
    vec![
      Position::new(1, 1),
      Position::new(11, 1),
      Position::new(11, 7),
      Position::new(1, 7),
    ]
  );
}

#[test]
fn every_generated_floor_tile_is_reachable_from_the_player() {
  for seed in 0..32 {
    for depth in [0, 1, 3, 6] {
      let world = procedural_floor(seed, depth).expect("generated floor should validate");
      let start = world
        .actor(ActorId::new(1))
        .expect("generated player should exist")
        .position();
      let mut visited = vec![start];
      let mut index = 0;
      while let Some(position) = visited.get(index).copied() {
        index += 1;
        for neighbor in [
          Position::new(position.x() + 1, position.y()),
          Position::new(position.x() - 1, position.y()),
          Position::new(position.x(), position.y() + 1),
          Position::new(position.x(), position.y() - 1),
        ] {
          if world.map().is_walkable(neighbor) && !visited.contains(&neighbor) {
            visited.push(neighbor);
          }
        }
      }
      let floor_count = world
        .map()
        .tiles()
        .iter()
        .filter(|tile| tile.is_walkable())
        .count();
      assert_eq!(visited.len(), floor_count, "seed={seed}, depth={depth}");
    }
  }
}
