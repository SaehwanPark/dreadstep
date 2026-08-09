//! Contract tests for the headless Bevy `Sprite` API projection.

use bevy::app::App;
use bevy::asset::Handle;
use bevy::color::Color;
use bevy::image::Image;
use bevy::math::Vec2;
use bevy::sprite::Sprite;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationPlugin, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, SceneRenderEntry,
  SceneRenderLayer, SceneRenderPlaceholder, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn sprite_app(with_tile_size: bool) -> App {
  let map = GridMap::from_tiles(
    4,
    1,
    vec![Tile::Floor, Tile::Floor, Tile::Floor, Tile::Wall],
  )
  .expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
      Actor::with_hit_points(
        ActorId::new(3),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should validate");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
    )
    .expect("ground item should be accepted");
  world
    .drop_item(ActorId::new(1), ItemId::new(1))
    .expect("ground item should be dropped");
  world
    .give_item(
      ActorId::new(1),
      Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
    )
    .expect("inventory item should be accepted");

  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  if with_tile_size {
    app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size should validate"));
  }
  app.insert_resource(PresentationRenderProjection::new());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn assert_sprite_shape(sprite: &Sprite, size: Option<Vec2>) {
  assert_eq!(sprite.custom_size, size);
  assert_eq!(sprite.image, Handle::<Image>::default());
  assert!(sprite.texture_atlas.is_none());
  assert!(sprite.rect.is_none());
}

fn expected_color(placeholder: SceneRenderPlaceholder) -> Color {
  match placeholder {
    SceneRenderPlaceholder::Terrain => Color::srgb(0.18, 0.18, 0.18),
    SceneRenderPlaceholder::Player => Color::srgb(0.1, 0.8, 0.3),
    SceneRenderPlaceholder::Enemy => Color::srgb(0.8, 0.2, 0.2),
    SceneRenderPlaceholder::DeadActor => Color::srgb(0.35, 0.35, 0.35),
    SceneRenderPlaceholder::GroundItem => Color::srgb(0.8, 0.65, 0.15),
    SceneRenderPlaceholder::InventoryItem => Color::srgb(0.2, 0.5, 0.9),
  }
}

#[test]
fn sprite_projection_covers_all_families_and_complete_node_metadata() {
  let mut app = sprite_app(true);
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();

  let nodes = app
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let sprites = app
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  assert_eq!(sprites.len(), nodes.len());
  assert!(sprites.iter().all(|entry| {
    assert_sprite_shape(entry.sprite(), Some(Vec2::new(32.0, 32.0)));
    assert_eq!(
      entry.sprite().color,
      expected_color(entry.node().node().placeholder())
    );
    entry.node().node_entity() != entry.node().node().source_entity()
  }));
  for (entry, node) in sprites.iter().zip(nodes.iter()) {
    assert_eq!(entry.node(), *node);
    assert_eq!(entry.node().node().order(), node.node().order());
    assert_eq!(entry.node().node().layer(), node.node().layer());
    assert_eq!(
      entry.node().node().pixel_position(),
      node.node().pixel_position()
    );
  }
  let families = sprites
    .iter()
    .map(|entry| entry.node().node().placeholder())
    .collect::<Vec<_>>();
  assert!(families.contains(&SceneRenderPlaceholder::Terrain));
  assert!(families.contains(&SceneRenderPlaceholder::Player));
  assert!(families.contains(&SceneRenderPlaceholder::DeadActor));
  assert!(families.contains(&SceneRenderPlaceholder::GroundItem));
  assert!(families.contains(&SceneRenderPlaceholder::InventoryItem));
  assert!(sprites.iter().any(|entry| {
    matches!(entry.node().node().key(), SceneSpriteKey::DeadActor)
      && entry.node().node().layer() == SceneRenderLayer::Actor
  }));
  assert!(sprites.iter().any(|entry| {
    matches!(entry.node().node().key(), SceneSpriteKey::InventoryItem(_))
      && entry.node().node().pixel_position().is_none()
  }));
}

#[test]
fn sprite_projection_refreshes_dead_node_and_retains_identity() {
  let mut app = sprite_app(true);
  let enemy_source = app
    .world_mut()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(2) => {
        Some(*entity)
      }
      _ => None,
    })
    .expect("target enemy mirror should exist");
  let before = app
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().node().source_entity() == enemy_source
        && entry.node().node().key() == SceneSpriteKey::Enemy
    })
    .map(|entry| entry.node().node_entity())
    .expect("living enemy sprite should exist");
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();
  let entries = app
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  let dead = entries
    .iter()
    .find(|entry| entry.node().node().key() == SceneSpriteKey::DeadActor)
    .expect("dead actor sprite should exist");
  assert_eq!(dead.node().node_entity(), before);
  assert!(entries.iter().any(|entry| {
    entry.node().node().source_entity() != enemy_source
      && entry.node().node().key() == SceneSpriteKey::Enemy
  }));
  assert_sprite_shape(dead.sprite(), Some(Vec2::new(32.0, 32.0)));
}

#[test]
fn sprite_projection_without_tile_size_is_unsized_metadata() {
  let mut app = sprite_app(false);
  let entries = app
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries();
  assert!(!entries.is_empty());
  assert!(entries.iter().all(|entry| {
    assert_sprite_shape(entry.sprite(), None);
    entry.node().node().pixel_position().is_none()
  }));
}

#[test]
fn sprite_projection_preserves_runtime_and_replay_state() {
  let mut app = sprite_app(true);
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
}

#[test]
fn sprite_projection_has_independent_runtime_source_and_destination_guards() {
  let mut missing_runtime = sprite_app(true);
  let before = missing_runtime
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(
    missing_runtime
      .world_mut()
      .resource::<PresentationBevySpriteProjection>()
      .entries(),
    before.as_slice()
  );

  let mut missing_source = sprite_app(true);
  let before = missing_source
    .world_mut()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  assert_eq!(
    missing_source
      .world_mut()
      .resource::<PresentationBevySpriteProjection>()
      .entries(),
    before.as_slice()
  );

  let mut missing_destination = sprite_app(true);
  missing_destination
    .world_mut()
    .remove_resource::<PresentationBevySpriteProjection>();
  missing_destination.update();
}
