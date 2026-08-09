//! Deterministic headless scene pixel-placement behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRuntime, PresentationState, PresentationTileSize, SceneActor,
  SceneGroundItem, ScenePixelPosition, SceneTile,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn item_state() -> PresentationState {
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
    .expect("item should be given");
  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("item should be dropped");
  PresentationState::new(7, world)
}

#[test]
fn tile_size_requires_nonzero_dimensions() {
  assert!(PresentationTileSize::new(0, 24).is_none());
  assert!(PresentationTileSize::new(24, 0).is_none());

  let size = PresentationTileSize::new(24, 32).expect("nonzero tile size should validate");
  assert_eq!(size.width(), 24);
  assert_eq!(size.height(), 32);
}

#[test]
fn pixel_positions_are_checked_and_reject_negative_coordinates() {
  let size = PresentationTileSize::new(24, 32).expect("tile size should validate");

  let position = size
    .pixel_position(Position::new(3, 2))
    .expect("valid coordinates should map");
  assert_eq!((position.x(), position.y()), (72, 64));
  assert!(size.pixel_position(Position::new(-1, 0)).is_none());
  assert!(
    PresentationTileSize::new(u32::MAX, 1)
      .expect("tile size should validate")
      .pixel_position(Position::new(2, 0))
      .is_none()
  );
}

#[test]
fn plugin_projects_terrain_actor_and_ground_item_origins() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.insert_resource(PresentationTileSize::new(24, 32).expect("tile size should validate"));
  app.add_plugins(PresentationPlugin);
  app.update();

  let world = app.world_mut();
  let tiles: BTreeMap<_, _> = world
    .query::<(&SceneTile, &ScenePixelPosition)>()
    .iter(world)
    .map(|(tile, pixel)| {
      (
        (tile.position().x(), tile.position().y()),
        (pixel.x(), pixel.y()),
      )
    })
    .collect();
  assert_eq!(tiles.get(&(2, 1)), Some(&(48, 32)));

  let actors: BTreeMap<_, _> = world
    .query::<(&SceneActor, &ScenePixelPosition)>()
    .iter(world)
    .map(|(actor, pixel)| (actor.id(), (pixel.x(), pixel.y())))
    .collect();
  assert_eq!(actors.get(&ActorId::new(1)), Some(&(0, 0)));
  assert_eq!(actors.get(&ActorId::new(2)), Some(&(48, 32)));

  let ground: BTreeMap<_, _> = world
    .query::<(&SceneGroundItem, &ScenePixelPosition)>()
    .iter(world)
    .map(|(item, pixel)| (item.id(), (pixel.x(), pixel.y())))
    .collect();
  assert_eq!(ground.get(&ItemId::new(1)), Some(&(0, 0)));
}

#[test]
fn accepted_movement_refreshes_retained_actor_origin() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.insert_resource(PresentationTileSize::new(24, 24).expect("tile size should validate"));
  app.add_plugins(PresentationPlugin);
  app.update();

  let player_entity = app
    .world_mut()
    .query::<(Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player mirror should exist");
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("player should move");
  app.update();

  let player = app
    .world_mut()
    .query::<(Entity, &SceneActor, &ScenePixelPosition)>()
    .iter(app.world())
    .find(|(_, actor, _)| actor.id() == ActorId::new(1))
    .expect("updated player mirror should exist");
  assert_eq!(player.0, player_entity);
  assert_eq!(player.1.position(), Position::new(1, 0));
  assert_eq!((player.2.x(), player.2.y()), (24, 0));
}

#[test]
fn missing_tile_size_leaves_scene_without_pixel_metadata() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.add_plugins(PresentationPlugin);
  app.update();

  assert_eq!(
    app
      .world_mut()
      .query::<&ScenePixelPosition>()
      .iter(app.world())
      .count(),
    0
  );
}

#[test]
fn missing_runtime_preserves_existing_pixel_metadata() {
  let mut app = App::new();
  let position = PresentationTileSize::new(24, 24)
    .expect("tile size should validate")
    .pixel_position(Position::new(2, 1))
    .expect("position should validate");
  app.world_mut().spawn(position);
  app.insert_resource(PresentationTileSize::new(24, 24).expect("tile size should validate"));
  app.add_plugins(PresentationPlugin);
  app.update();

  let pixel = app
    .world_mut()
    .query::<&ScenePixelPosition>()
    .single(app.world())
    .expect("existing metadata should remain");
  assert_eq!((pixel.x(), pixel.y()), (48, 24));
}
