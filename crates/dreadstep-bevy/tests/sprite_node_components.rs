//! Contract tests for attaching headless Bevy `Sprite` values to render-node entities.

use bevy::app::App;
use bevy::asset::Handle;
use bevy::camera::visibility::Visibility;
use bevy::image::Image;
use bevy::math::Vec2;
use bevy::sprite::Sprite;
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationPlugin, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, SceneRenderEntry,
  SceneRenderPlaceholder, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn node_app() -> App {
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
  app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size should validate"));
  app.insert_resource(PresentationRenderProjection::new());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn expected_color(placeholder: SceneRenderPlaceholder) -> bevy::color::Color {
  match placeholder {
    SceneRenderPlaceholder::Terrain => bevy::color::Color::srgb(0.18, 0.18, 0.18),
    SceneRenderPlaceholder::Player => bevy::color::Color::srgb(0.1, 0.8, 0.3),
    SceneRenderPlaceholder::Enemy => bevy::color::Color::srgb(0.8, 0.2, 0.2),
    SceneRenderPlaceholder::DeadActor => bevy::color::Color::srgb(0.35, 0.35, 0.35),
    SceneRenderPlaceholder::GroundItem => bevy::color::Color::srgb(0.8, 0.65, 0.15),
    SceneRenderPlaceholder::InventoryItem => bevy::color::Color::srgb(0.2, 0.5, 0.9),
  }
}

fn assert_sprite(sprite: &Sprite, placeholder: SceneRenderPlaceholder) {
  assert_eq!(sprite.color, expected_color(placeholder));
  assert_eq!(sprite.custom_size, Some(Vec2::new(32.0, 32.0)));
  assert_eq!(sprite.image, Handle::<Image>::default());
  assert!(sprite.texture_atlas.is_none());
  assert!(sprite.rect.is_none());
}

fn assert_same_sprite(actual: &Sprite, expected: &Sprite) {
  assert_eq!(actual.image, expected.image);
  assert_eq!(actual.texture_atlas, expected.texture_atlas);
  assert_eq!(actual.color, expected.color);
  assert_eq!(actual.flip_x, expected.flip_x);
  assert_eq!(actual.flip_y, expected.flip_y);
  assert_eq!(actual.custom_size, expected.custom_size);
  assert_eq!(actual.rect, expected.rect);
  assert_eq!(actual.image_mode, expected.image_mode);
}

fn sprite_for_node(app: &App, entity: bevy::ecs::entity::Entity) -> Sprite {
  app
    .world()
    .get_entity(entity)
    .expect("node entity should exist")
    .get::<Sprite>()
    .cloned()
    .expect("node should carry a Sprite component")
}

#[test]
fn attaches_complete_sprites_to_every_node_without_render_plugin() {
  let app = node_app();
  assert!(!app.is_plugin_added::<bevy::sprite::SpritePlugin>());
  let entries = app
    .world()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  assert!(!entries.is_empty());
  for entry in &entries {
    let entity = entry.node().node_entity();
    let sprite = sprite_for_node(&app, entity);
    assert_same_sprite(&sprite, entry.sprite());
    assert_sprite(&sprite, entry.node().node().placeholder());
    assert_eq!(
      app
        .world()
        .get_entity(entity)
        .expect("node should exist")
        .get::<Transform>()
        .copied(),
      Some(Transform::default())
    );
    assert_eq!(
      app
        .world()
        .get_entity(entity)
        .expect("node should exist")
        .get::<Visibility>()
        .copied(),
      Some(Visibility::Inherited)
    );
  }
  let families = entries
    .iter()
    .map(|entry| entry.node().node().placeholder())
    .collect::<Vec<_>>();
  for family in [
    SceneRenderPlaceholder::Terrain,
    SceneRenderPlaceholder::Player,
    SceneRenderPlaceholder::Enemy,
    SceneRenderPlaceholder::GroundItem,
    SceneRenderPlaceholder::InventoryItem,
  ] {
    assert!(families.contains(&family));
  }
}

#[test]
fn dead_refresh_updates_the_same_node_sprite() {
  let mut app = node_app();
  let source = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(2) => {
        Some(*entity)
      }
      _ => None,
    })
    .expect("target actor mirror should exist");
  let before_node = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().source_entity() == source && entry.node().key() == SceneSpriteKey::Enemy
    })
    .map(|entry| entry.node_entity())
    .expect("enemy node should exist");
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();
  let dead = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().source_entity() == source && entry.node().key() == SceneSpriteKey::DeadActor
    })
    .expect("dead node should exist");
  assert_eq!(dead.node_entity(), before_node);
  assert_sprite(
    &sprite_for_node(&app, before_node),
    SceneRenderPlaceholder::DeadActor,
  );
}

#[test]
fn stale_inventory_despawns_its_sprite_and_retains_other_nodes() {
  let mut app = node_app();
  let before = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let stale = before
    .iter()
    .find(|entry| entry.node().placeholder() == SceneRenderPlaceholder::InventoryItem)
    .copied()
    .expect("inventory node should exist");
  let retained_sprites = before
    .iter()
    .filter(|entry| entry.node_entity() != stale.node_entity())
    .map(|entry| {
      (
        entry.node_entity(),
        sprite_for_node(&app, entry.node_entity()),
      )
    })
    .collect::<Vec<_>>();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect("inventory item should be consumable");
  app.update();
  assert!(app.world().get_entity(stale.node_entity()).is_err());
  assert!(
    !app
      .world()
      .resource::<PresentationRenderNodeProjection>()
      .entries()
      .iter()
      .any(|entry| entry.node_entity() == stale.node_entity())
  );
  for (entity, sprite) in retained_sprites {
    assert_same_sprite(&sprite_for_node(&app, entity), &sprite);
  }
}

#[test]
fn co_located_mirrors_keep_distinct_sprite_components() {
  let mut app = node_app();
  let (tile_entity, tile) = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Terrain { entity, tile, .. } if tile.position() == Position::new(2, 0) => {
        Some((*entity, *tile))
      }
      _ => None,
    })
    .expect("tile mirror should exist");
  let actor_entity = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(3) => {
        Some(*entity)
      }
      _ => None,
    })
    .expect("actor mirror should exist");
  app.world_mut().despawn(tile_entity);
  app.world_mut().entity_mut(actor_entity).insert(tile);
  app.update();
  let co_located = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .filter(|entry| entry.node().source_entity() == actor_entity)
    .copied()
    .collect::<Vec<_>>();
  assert_eq!(co_located.len(), 2);
  let terrain = co_located
    .iter()
    .find(|entry| entry.node().placeholder() == SceneRenderPlaceholder::Terrain)
    .expect("co-located terrain node should exist");
  let actor = co_located
    .iter()
    .find(|entry| entry.node().placeholder() == SceneRenderPlaceholder::Enemy)
    .expect("co-located actor node should exist");
  assert_ne!(terrain.node_entity(), actor.node_entity());
  assert_eq!(terrain.node().order(), 2);
  assert_eq!(actor.node().order(), 6);
  assert_sprite(
    &sprite_for_node(&app, terrain.node_entity()),
    SceneRenderPlaceholder::Terrain,
  );
  assert_sprite(
    &sprite_for_node(&app, actor.node_entity()),
    SceneRenderPlaceholder::Enemy,
  );
}

#[test]
fn missing_runtime_source_projection_and_node_entity_are_safe() {
  let mut missing_runtime = node_app();
  let player = missing_runtime
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::Player)
    .copied()
    .expect("player node should exist");
  let before = sprite_for_node(&missing_runtime, player.node_entity());
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_same_sprite(
    &sprite_for_node(&missing_runtime, player.node_entity()),
    &before,
  );

  let mut missing_source = node_app();
  let player = missing_source
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::Player)
    .copied()
    .expect("player node should exist");
  let before = sprite_for_node(&missing_source, player.node_entity());
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  assert_same_sprite(
    &sprite_for_node(&missing_source, player.node_entity()),
    &before,
  );

  let mut missing_projection = node_app();
  let player = missing_projection
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::Player)
    .copied()
    .expect("player node should exist");
  let before = sprite_for_node(&missing_projection, player.node_entity());
  missing_projection
    .world_mut()
    .remove_resource::<PresentationBevySpriteProjection>();
  missing_projection.update();
  assert_same_sprite(
    &sprite_for_node(&missing_projection, player.node_entity()),
    &before,
  );

  let mut missing_entity = node_app();
  let player = missing_entity
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::Player)
    .copied()
    .expect("player node should exist");
  let before_nodes = missing_entity
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let before_sprites = missing_entity
    .world()
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  missing_entity.world_mut().despawn(player.node_entity());
  missing_entity
    .world_mut()
    .remove_resource::<PresentationRenderCommandPlan>();
  missing_entity.update();
  assert!(
    missing_entity
      .world()
      .get_entity(player.node_entity())
      .is_err()
  );
  assert_eq!(
    missing_entity
      .world()
      .resource::<PresentationRenderNodeProjection>()
      .entries(),
    before_nodes.as_slice()
  );
  assert_eq!(
    missing_entity
      .world()
      .resource::<PresentationBevySpriteProjection>()
      .entries(),
    before_sprites.as_slice()
  );
}

#[test]
fn sprite_component_attachment_does_not_mutate_runtime_or_replay() {
  let mut app = node_app();
  let snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.update();
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), snapshot);
  assert_eq!(runtime.replay_digest(), digest);
}
