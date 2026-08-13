//! Content contract tests for the authored chill-trap floor.

use dreadstep_content::chill_trap_floor;
use dreadstep_core::{Position, Tile};

#[test]
fn authored_chill_floor_has_one_walkable_chill_trap() {
  let world = chill_trap_floor().expect("authored chill floor should validate");
  assert_eq!(
    world.map().tile_at(Position::new(2, 1)),
    Some(Tile::ChillTrap)
  );
  assert!(Tile::ChillTrap.is_walkable());
}
