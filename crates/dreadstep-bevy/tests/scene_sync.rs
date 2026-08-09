//! Headless ECS scene synchronization behavior.

use bevy::ecs::world::World;
use dreadstep_bevy::{PresentationState, SceneActor, SceneTile, sync_scene};
use dreadstep_content::starter_floor;
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Position, Tile, WorldState,
};

fn combat_state() -> PresentationState {
  let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Floor]).expect("map should validate");
  let world = WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .expect("world should validate");
  PresentationState::new(7, world)
}

#[test]
fn sync_creates_scene_entities_and_preserves_keys_across_updates() {
  let mut state = PresentationState::start_run(7).expect("content should validate");
  let initial = state.snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &initial);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 35);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 4);
  let player_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should exist");

  sync_scene(&mut scene, &initial);
  let repeated_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should remain");
  assert_eq!(repeated_entity, player_entity);

  let output = state
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .expect("player should move");
  sync_scene(&mut scene, output.snapshot());
  let player = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .expect("updated player scene entity should exist");
  assert_eq!(player.0, player_entity);
  assert_eq!(player.1.position(), Position::new(2, 1));
}

#[test]
fn sync_removes_entities_absent_from_a_later_snapshot() {
  let full = combat_state().snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);

  let map = GridMap::filled(1, 1, Tile::Floor).expect("map should validate");
  let reduced_world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("reduced world should validate");
  let reduced = PresentationState::new(7, reduced_world).snapshot();
  sync_scene(&mut scene, &reduced);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 1);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 1);
  assert_eq!(
    scene
      .query::<&SceneActor>()
      .iter(&scene)
      .next()
      .expect("player should remain")
      .id(),
    ActorId::new(1)
  );
}

#[test]
fn sync_retains_dead_actor_records_for_presentation() {
  let mut state = combat_state();
  let mut scene = World::new();
  sync_scene(&mut scene, &state.snapshot());
  let output = state
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  sync_scene(&mut scene, output.snapshot());

  let enemy = scene
    .query::<&SceneActor>()
    .iter(&scene)
    .find(|actor| actor.id() == ActorId::new(2))
    .expect("dead actor should remain represented");
  assert!(!enemy.is_alive());
  assert_eq!(enemy.hit_points(), HitPoints::new(0));
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
}

#[test]
fn starter_snapshot_uses_typed_scene_values() {
  let snapshot =
    PresentationState::new(1, starter_floor().expect("content should validate")).snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);

  let tile = scene
    .query::<&SceneTile>()
    .iter(&scene)
    .next()
    .expect("tile should exist");
  assert_eq!(tile.position(), Position::new(0, 0));
  assert_eq!(tile.terrain(), Tile::Wall);
}
