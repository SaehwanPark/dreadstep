//! Headless ECS scene synchronization behavior.

use std::collections::BTreeMap;

use bevy::ecs::world::World;
use dreadstep_bevy::{PresentationState, SceneActor, SceneGroundItem, SceneTile, sync_scene};
use dreadstep_content::starter_floor;
use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldState,
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

fn ground_world() -> WorldState {
  let map = GridMap::from_tiles(
    3,
    2,
    vec![
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
    ],
  )
  .expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 1)),
    ],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("first item should be given");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("second item should be given");
  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("first item should be dropped");
  world
    .drop_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should be dropped");
  world
    .give_item(
      ActorId::new(2),
      Item::new(ItemId::new(3), ItemDefinitionId::new(103)),
    )
    .expect("third item should be given");
  world
    .drop_item(ActorId::new(2), ItemId::new(3))
    .expect("third item should be dropped");
  world
}

#[test]
fn sync_creates_scene_entities_and_preserves_keys_across_updates() {
  let mut state = PresentationState::start_run(7).expect("content should validate");
  let initial = state.snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &initial);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 35);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 4);
  assert!(initial.ground_items().is_empty());
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);
  let player_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should exist");
  let tile_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneTile)>()
    .iter(&scene)
    .find(|(_, tile)| tile.position() == Position::new(0, 0))
    .map(|(entity, _)| entity)
    .expect("origin tile scene entity should exist");

  sync_scene(&mut scene, &initial);
  let repeated_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should remain");
  let repeated_tile = scene
    .query::<(bevy::ecs::entity::Entity, &SceneTile)>()
    .iter(&scene)
    .find(|(_, tile)| tile.position() == Position::new(0, 0))
    .map(|(entity, _)| entity)
    .expect("origin tile scene entity should remain");
  assert_eq!(repeated_entity, player_entity);
  assert_eq!(repeated_tile, tile_entity);

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
  assert_eq!(player.1.ready_at(), ActionTime::new(1));
}

#[test]
fn sync_projects_complete_ground_items_and_preserves_item_identity() {
  let snapshot = PresentationState::new(7, ground_world()).snapshot();
  assert_eq!(snapshot.ground_items().len(), 2);
  assert_eq!(snapshot.ground_items()[0].position(), Position::new(0, 0));
  assert_eq!(snapshot.ground_items()[0].items()[0].id(), ItemId::new(1));
  assert_eq!(
    snapshot.ground_items()[0].items()[0].definition(),
    ItemDefinitionId::new(101)
  );
  assert_eq!(snapshot.ground_items()[0].items()[1].id(), ItemId::new(2));
  assert_eq!(snapshot.ground_items()[1].position(), Position::new(2, 1));

  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);

  let first_entities: BTreeMap<_, _> = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .map(|(entity, item)| (item.id(), (entity, *item)))
    .collect();
  assert_eq!(first_entities.len(), 3);
  assert_eq!(
    first_entities[&ItemId::new(1)].1.position(),
    Position::new(0, 0)
  );
  assert_eq!(
    first_entities[&ItemId::new(2)].1.definition(),
    ItemDefinitionId::new(102)
  );
  assert_eq!(
    first_entities[&ItemId::new(3)].1.position(),
    Position::new(2, 1)
  );

  sync_scene(&mut scene, &snapshot);
  let repeated_entities: BTreeMap<_, _> = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .map(|(entity, item)| (item.id(), entity))
    .collect();
  for (item_id, (entity, _)) in first_entities {
    assert_eq!(repeated_entities[&item_id], entity);
  }
}

#[test]
fn sync_deduplicates_public_mirror_entities_by_stable_key() {
  let snapshot = PresentationState::start_run(7)
    .expect("content should validate")
    .snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &snapshot);
  let tile = *scene
    .query::<&SceneTile>()
    .iter(&scene)
    .next()
    .expect("tile should exist");
  let actor = *scene
    .query::<&SceneActor>()
    .iter(&scene)
    .next()
    .expect("actor should exist");
  let ground_snapshot = PresentationState::new(7, ground_world()).snapshot();
  sync_scene(&mut scene, &ground_snapshot);
  let ground_item = *scene
    .query::<&SceneGroundItem>()
    .iter(&scene)
    .next()
    .expect("ground item should exist");
  scene.spawn(tile);
  scene.spawn(actor);
  scene.spawn(ground_item);
  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 7);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 3);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 4);

  sync_scene(&mut scene, &ground_snapshot);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 6);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 3);
}

#[test]
fn sync_removes_entities_absent_from_a_later_snapshot() {
  let full = combat_state().snapshot();
  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);

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
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 0);
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
fn sync_removes_picked_up_items_while_preserving_other_scene_mirrors() {
  let full = PresentationState::new(7, ground_world()).snapshot();
  let mut reduced_world = ground_world();
  reduced_world
    .pickup_item(ActorId::new(1), ItemId::new(2))
    .expect("second item should be picked up");
  let reduced = PresentationState::new(7, reduced_world).snapshot();

  let mut scene = World::new();
  sync_scene(&mut scene, &full);
  let item_one_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(1))
    .map(|(entity, _)| entity)
    .expect("first ground item should exist");
  let item_three_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
    .iter(&scene)
    .find(|(_, item)| item.id() == ItemId::new(3))
    .map(|(entity, _)| entity)
    .expect("third ground item should exist");
  let player_entity = scene
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(&scene)
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player should exist");

  sync_scene(&mut scene, &reduced);

  assert_eq!(scene.query::<&SceneTile>().iter(&scene).count(), 6);
  assert_eq!(scene.query::<&SceneActor>().iter(&scene).count(), 2);
  assert_eq!(scene.query::<&SceneGroundItem>().iter(&scene).count(), 2);
  assert!(
    scene
      .query::<&SceneGroundItem>()
      .iter(&scene)
      .all(|item| item.id() != ItemId::new(2))
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
      .iter(&scene)
      .find(|(_, item)| item.id() == ItemId::new(1))
      .map(|(entity, _)| entity),
    Some(item_one_entity)
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneGroundItem)>()
      .iter(&scene)
      .find(|(_, item)| item.id() == ItemId::new(3))
      .map(|(entity, _)| entity),
    Some(item_three_entity)
  );
  assert_eq!(
    scene
      .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
      .iter(&scene)
      .find(|(_, actor)| actor.id() == ActorId::new(1))
      .map(|(entity, _)| entity),
    Some(player_entity)
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
