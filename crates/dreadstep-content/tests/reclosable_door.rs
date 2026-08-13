//! Authored reclosable-door content fixture behavior.

use dreadstep_content::{reclosable_door_floor, starter_item_showcase_floor};
use dreadstep_core::{Position, Tile};

#[test]
fn authored_fixture_places_a_closed_door_next_to_the_player() {
  let world = reclosable_door_floor().expect("authored door floor should validate");

  assert_eq!(world.map().tile_at(Position::new(2, 1)), Some(Tile::Door));
}

#[test]
fn item_showcase_exposes_the_same_reachable_door_with_items() {
  let world = starter_item_showcase_floor().expect("item showcase should validate");

  assert_eq!(world.map().tile_at(Position::new(2, 1)), Some(Tile::Door));
  assert_eq!(
    world
      .actors()
      .next()
      .expect("player should exist")
      .inventory()
      .len(),
    4
  );
}
