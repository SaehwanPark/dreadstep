//! Deterministic headless scene sprite-role metadata behavior.

use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use dreadstep_bevy::{
  PresentationState, SceneActor, SceneGroundItem, SceneInventoryItem, SceneSpriteRole, SceneTile,
  sync_scene,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn custom_state(actors: Vec<Actor>) -> PresentationState {
  let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Floor]).expect("map should validate");
  PresentationState::new(
    7,
    WorldState::new(map, actors).expect("world should validate"),
  )
}

fn state_with_items() -> PresentationState {
  let map = GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Floor]).expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("item should be owned");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("item should be owned");
  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("item should drop");
  PresentationState::new(7, world)
}

fn scene_actor_entity(world: &mut World, actor: ActorId) -> Entity {
  let mut query = world.query::<(Entity, &SceneActor)>();
  query
    .iter(world)
    .find_map(|(entity, scene_actor)| (scene_actor.id() == actor).then_some(entity))
    .expect("actor scene entity should exist")
}

#[test]
fn sync_assigns_terrain_and_living_actor_roles() {
  let state = custom_state(vec![
    Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
    Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(1, 0)),
  ]);
  let snapshot = state.snapshot();
  let mut world = World::new();

  sync_scene(&mut world, &snapshot);

  let mut tile_query = world.query::<(&SceneTile, &SceneSpriteRole)>();
  assert_eq!(tile_query.iter(&world).count(), 2);
  assert!(
    tile_query
      .iter(&world)
      .all(|(_, role)| *role == SceneSpriteRole::Terrain)
  );
  let mut actor_query = world.query::<(&SceneActor, &SceneSpriteRole)>();
  let roles = actor_query
    .iter(&world)
    .map(|(actor, role)| (actor.id(), *role))
    .collect::<Vec<_>>();
  assert_eq!(
    roles,
    vec![
      (ActorId::new(1), SceneSpriteRole::Player),
      (ActorId::new(2), SceneSpriteRole::Enemy),
    ]
  );
}

#[test]
fn death_refreshes_role_without_replacing_actor_entity() {
  let mut state = custom_state(vec![
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
  ]);
  let mut world = World::new();
  sync_scene(&mut world, &state.snapshot());
  let enemy_entity = scene_actor_entity(&mut world, ActorId::new(2));

  state
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  sync_scene(&mut world, &state.snapshot());

  assert_eq!(
    scene_actor_entity(&mut world, ActorId::new(2)),
    enemy_entity
  );
  let role = world
    .entity(enemy_entity)
    .get::<SceneSpriteRole>()
    .copied()
    .expect("dead actor should retain a role");
  assert_eq!(role, SceneSpriteRole::DeadActor);
}

#[test]
fn ground_and_inventory_items_receive_item_roles() {
  let state = state_with_items();
  let snapshot = state.snapshot();
  let mut world = World::new();

  sync_scene(&mut world, &snapshot);

  let mut ground_query = world.query::<(&SceneGroundItem, &SceneSpriteRole)>();
  assert_eq!(
    ground_query
      .iter(&world)
      .map(|(_, role)| *role)
      .collect::<Vec<_>>(),
    vec![SceneSpriteRole::GroundItem]
  );
  let mut inventory_query = world.query::<(&SceneInventoryItem, &SceneSpriteRole)>();
  assert_eq!(
    inventory_query
      .iter(&world)
      .map(|(_, role)| *role)
      .collect::<Vec<_>>(),
    vec![SceneSpriteRole::InventoryItem]
  );
}

#[test]
fn stale_item_entities_and_roles_are_removed() {
  let state_with_item = state_with_items();
  let mut world = World::new();
  sync_scene(&mut world, &state_with_item.snapshot());
  let state_without_item = custom_state(vec![Actor::new(
    ActorId::new(1),
    ActorKind::Player,
    Position::new(0, 0),
  )]);
  sync_scene(&mut world, &state_without_item.snapshot());

  let mut role_query = world.query::<&SceneSpriteRole>();
  assert!(role_query.iter(&world).all(|role| {
    matches!(
      role,
      SceneSpriteRole::Terrain | SceneSpriteRole::Player | SceneSpriteRole::Enemy
    )
  }));
}

#[test]
fn unsynchronized_scene_does_not_invent_sprite_roles() {
  let mut world = World::new();

  let mut query = world.query::<&SceneSpriteRole>();
  assert_eq!(query.iter(&world).count(), 0);
}
