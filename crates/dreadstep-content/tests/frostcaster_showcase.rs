//! Frostcaster identity in the presentation-only authored item showcase.

use dreadstep_content::{starter_item_floor, starter_item_showcase_floor};
use dreadstep_core::{ActorId, EnemyBehavior, Position, Tile};

#[test]
fn showcase_authors_actor_three_as_frostcaster_without_changing_item_fixture() {
  let showcase = starter_item_showcase_floor().expect("showcase should validate");
  let item_fixture = starter_item_floor().expect("item fixture should validate");

  assert_eq!(
    showcase
      .actor(ActorId::new(3))
      .expect("showcase actor 3 should exist")
      .enemy_behavior(),
    EnemyBehavior::Frostcaster
  );
  assert_eq!(
    item_fixture
      .actor(ActorId::new(3))
      .expect("item-fixture actor 3 should exist")
      .enemy_behavior(),
    EnemyBehavior::Pursuer
  );
  assert_eq!(
    showcase.map().tile_at(Position::new(2, 1)),
    Some(Tile::Door)
  );
  assert_eq!(
    showcase
      .actor(ActorId::new(4))
      .expect("showcase Blocker should exist")
      .enemy_behavior(),
    EnemyBehavior::Blocker
  );
}
