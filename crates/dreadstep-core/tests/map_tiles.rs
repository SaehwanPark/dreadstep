//! Map projection behavior for the deterministic core.

use dreadstep_core::{GridMap, Tile};

#[test]
fn map_tiles_are_exposed_in_validated_row_major_order() {
  let map = GridMap::from_tiles(2, 2, vec![Tile::Floor, Tile::Wall, Tile::Wall, Tile::Floor])
    .expect("map should validate");

  assert_eq!(
    map.tiles(),
    &[Tile::Floor, Tile::Wall, Tile::Wall, Tile::Floor]
  );
}
