//! Bevy projection evidence for the typed actor melee reach.

use bevy::ecs::world::World;
use dreadstep_bevy::{PresentationState, SceneActor, sync_scene};
use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, MeleeReach, Position, Tile, WorldState,
};

#[test]
fn scene_actor_mirrors_explicit_melee_reach_without_changing_identity() {
  let world = WorldState::new(
    GridMap::filled(3, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::with_melee_reach(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
        MeleeReach::new(2).expect("two is a valid reach"),
      ),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 0)),
    ],
  )
  .expect("world should be valid");
  let state = PresentationState::new(7, world);
  let snapshot = state.snapshot();
  let mut scene = World::new();

  sync_scene(&mut scene, &snapshot);

  let mut query = scene.query::<&SceneActor>();
  let player = query
    .iter(&scene)
    .find(|actor| actor.id() == ActorId::new(1))
    .copied()
    .expect("player mirror should exist");
  assert_eq!(
    player.melee_reach(),
    MeleeReach::new(2).expect("two is valid")
  );
}
