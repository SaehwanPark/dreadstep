//! Authored starter-floor behavior.

use dreadstep_content::{ContentError, StarterFloorDefinition, starter_floor};
use dreadstep_core::{Actor, ActorId, ActorKind, HitPoints, Position, Tile};

#[test]
fn starter_floor_has_stable_shape_and_digest() {
  let world = starter_floor().expect("authored starter floor should validate");
  let actors: Vec<_> = world.actors().collect();

  assert_eq!(world.map().width(), 7);
  assert_eq!(world.map().height(), 5);
  assert_eq!(actors.len(), 4);
  assert_eq!(actors[0].id(), ActorId::new(1));
  assert_eq!(actors[0].kind(), ActorKind::Player);
  assert!(
    actors[1..]
      .iter()
      .all(|actor| actor.kind() == ActorKind::Enemy)
  );
  assert!(actors.iter().all(|actor| actor.is_alive()));
  assert_eq!(
    actors
      .iter()
      .map(|actor| actor.position())
      .collect::<Vec<_>>(),
    vec![
      Position::new(1, 1),
      Position::new(5, 1),
      Position::new(1, 3),
      Position::new(5, 3),
    ]
  );
  assert_eq!(
    world.map().tiles(),
    &[
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ]
  );
  assert_eq!(
    world.digest(),
    starter_floor().expect("same content").digest()
  );
}

#[test]
fn invalid_authored_floor_is_reported_before_world_creation() {
  let definition = StarterFloorDefinition::new(
    0,
    5,
    Vec::new(),
    vec![Actor::with_hit_points(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
      HitPoints::new(0),
    )],
  );

  assert!(matches!(definition.build(), Err(ContentError::Map(_))));

  let invalid_actor = StarterFloorDefinition::new(
    1,
    1,
    vec![Tile::Floor],
    vec![Actor::with_hit_points(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
      HitPoints::new(0),
    )],
  );
  assert!(matches!(invalid_actor.build(), Err(ContentError::World(_))));
}
