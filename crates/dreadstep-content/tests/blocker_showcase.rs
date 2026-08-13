//! Stationary Blocker identity in the authored desktop showcase.

use dreadstep_content::{starter_item_floor, starter_item_showcase_floor};
use dreadstep_core::{ActorId, EnemyBehavior, Position};

#[test]
fn showcase_authors_actor_four_as_blocker_at_the_lower_chokepoint() {
  let showcase = starter_item_showcase_floor().expect("showcase should validate");
  let item_fixture = starter_item_floor().expect("item fixture should validate");

  let blocker = showcase
    .actor(ActorId::new(4))
    .expect("showcase Blocker should exist");
  assert_eq!(blocker.enemy_behavior(), EnemyBehavior::Blocker);
  assert_eq!(blocker.position(), Position::new(3, 3));
  assert_eq!(
    item_fixture
      .actor(ActorId::new(4))
      .expect("item-fixture actor 4 should exist")
      .enemy_behavior(),
    EnemyBehavior::Brute
  );
}
