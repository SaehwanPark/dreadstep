//! Seeded procedural-floor content invariants.

use dreadstep_content::{procedural_floor, procedural_floor_definition};
use dreadstep_core::{ActorId, ActorKind, Position, Tile};

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
