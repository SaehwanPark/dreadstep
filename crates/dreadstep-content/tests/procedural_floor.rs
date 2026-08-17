//! Seeded procedural-floor content invariants.

use dreadstep_content::{procedural_floor, procedural_floor_definition};
use dreadstep_core::{ActorId, ActorKind, EnemyBehavior, Position, Tile};

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
fn procedural_floor_places_one_seeded_equipment_item_on_the_ground() {
  let world = procedural_floor(7, 1).expect("generated floor should validate");
  let [stack] = world.ground_items() else {
    panic!("procedural floor should provide exactly one ground stack");
  };
  let [item] = stack.items() else {
    panic!("procedural ground stack should provide exactly one item");
  };

  assert_eq!(stack.position(), Position::new(11, 1));
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
