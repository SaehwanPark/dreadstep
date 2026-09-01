//! Map projection behavior for the deterministic core.

use dreadstep_core::{GridMap, Tile};

#[test]
fn map_tiles_are_exposed_in_validated_row_major_order() {
  let map = GridMap::from_tiles(
    2,
    2,
    vec![Tile::Floor, Tile::Stairs, Tile::Wall, Tile::Floor],
  )
  .expect("map should validate");

  assert_eq!(
    map.tiles(),
    &[Tile::Floor, Tile::Stairs, Tile::Wall, Tile::Floor]
  );
}

#[test]
fn stairs_are_walkable_and_transparent_to_ranged_lines() {
  assert!(Tile::Stairs.is_walkable());
  assert!(!Tile::Stairs.blocks_ranged_line_of_sight());
}
