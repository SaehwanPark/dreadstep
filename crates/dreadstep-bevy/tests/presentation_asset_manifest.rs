//! Contract tests for local-only presentation asset metadata.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationAssetManifest, PresentationAssetReference, PresentationPlugin,
  PresentationRenderAssetProjection, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, SceneRenderPlaceholder,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position, Tile,
  WorldState,
};

fn reference(path: &str) -> PresentationAssetReference {
  PresentationAssetReference::new(path).expect("fixture path should validate")
}

fn manifest(suffix: &str) -> PresentationAssetManifest {
  PresentationAssetManifest::new(vec![
    (
      SceneRenderPlaceholder::Terrain,
      reference(&format!("art/terrain-{suffix}.png")),
    ),
    (
      SceneRenderPlaceholder::Player,
      reference(&format!("art/player-{suffix}.png")),
    ),
    (
      SceneRenderPlaceholder::Enemy,
      reference(&format!("art/enemy-{suffix}.png")),
    ),
    (
      SceneRenderPlaceholder::DeadActor,
      reference(&format!("art/dead-{suffix}.png")),
    ),
    (
      SceneRenderPlaceholder::GroundItem,
      reference(&format!("art/ground-{suffix}.png")),
    ),
    (
      SceneRenderPlaceholder::InventoryItem,
      reference(&format!("art/inventory-{suffix}.png")),
    ),
  ])
  .expect("manifest should contain each family once")
}

fn asset_app() -> App {
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
  app.insert_resource(manifest("one"));
  app.insert_resource(PresentationRenderAssetProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

#[test]
fn asset_projection_joins_every_node_without_loading_files() {
  let mut app = asset_app();
  let nodes = app
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let assets = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  assert_eq!(assets.len(), nodes.len());
  for (asset, node) in assets.iter().zip(nodes.iter()) {
    assert_eq!(asset.node(), *node);
    let family = match node.node().placeholder() {
      SceneRenderPlaceholder::Terrain => "terrain",
      SceneRenderPlaceholder::Player => "player",
      SceneRenderPlaceholder::Enemy => "enemy",
      SceneRenderPlaceholder::DeadActor => "dead",
      SceneRenderPlaceholder::GroundItem => "ground",
      SceneRenderPlaceholder::InventoryItem => "inventory",
    };
    let expected = format!("art/{family}-one.png");
    assert_eq!(
      asset
        .reference()
        .expect("manifest entry should exist")
        .path(),
      expected
    );
  }
  assert!(assets.iter().all(|entry| entry.reference().is_some()));
}

#[test]
fn asset_reference_validation_rejects_unsafe_paths_and_manifest_shape() {
  for path in [
    "",
    "/absolute.png",
    "../parent.png",
    "art/../parent.png",
    "C:/asset.png",
    "art\\asset.png",
  ] {
    assert!(
      PresentationAssetReference::new(path).is_none(),
      "{path} should be rejected"
    );
  }
  assert!(PresentationAssetReference::new("art/valid.png").is_some());
  let one = reference("art/one.png");
  assert!(
    PresentationAssetManifest::new(vec![(SceneRenderPlaceholder::Terrain, one.clone())]).is_none()
  );
  assert!(
    PresentationAssetManifest::new(vec![
      (SceneRenderPlaceholder::Terrain, one.clone()),
      (SceneRenderPlaceholder::Terrain, one.clone()),
      (SceneRenderPlaceholder::Player, one.clone()),
      (SceneRenderPlaceholder::Enemy, one.clone()),
      (SceneRenderPlaceholder::DeadActor, one.clone()),
      (SceneRenderPlaceholder::GroundItem, one),
    ])
    .is_none()
  );
}

#[test]
fn manifest_refresh_preserves_nodes_and_updates_references() {
  let mut app = asset_app();
  let before = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  app.world_mut().insert_resource(manifest("two"));
  app.update();
  let after = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  assert_eq!(before.len(), after.len());
  for (old, new) in before.iter().zip(after.iter()) {
    assert_eq!(old.node().node_entity(), new.node().node_entity());
    assert_ne!(old.reference(), new.reference());
    assert!(
      new
        .reference()
        .expect("updated reference")
        .path()
        .ends_with("-two.png")
    );
  }
}

#[test]
fn missing_manifest_source_runtime_and_destination_preserve_safely() {
  let mut missing_manifest = asset_app();
  let before = missing_manifest
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  missing_manifest
    .world_mut()
    .remove_resource::<PresentationAssetManifest>();
  missing_manifest.update();
  assert_eq!(
    missing_manifest
      .world()
      .resource::<PresentationRenderAssetProjection>()
      .entries(),
    before.as_slice()
  );

  let mut missing_source = asset_app();
  let before_source = missing_source
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  assert_eq!(
    missing_source
      .world()
      .resource::<PresentationRenderAssetProjection>()
      .entries(),
    before_source.as_slice()
  );

  let mut missing_runtime = asset_app();
  let before_runtime = missing_runtime
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(
    missing_runtime
      .world()
      .resource::<PresentationRenderAssetProjection>()
      .entries(),
    before_runtime.as_slice()
  );

  let mut missing_destination = asset_app();
  missing_destination
    .world_mut()
    .remove_resource::<PresentationRenderAssetProjection>();
  missing_destination.update();
  assert!(
    missing_destination
      .world()
      .get_resource::<PresentationRenderAssetProjection>()
      .is_none()
  );
}
