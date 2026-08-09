//! Contract tests for local-only presentation asset metadata.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationAssetManifest, PresentationAssetReference, PresentationPlugin,
  PresentationRenderAssetProjection, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, SceneRenderPlaceholder,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
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
    assert_eq!(asset.reference().path(), expected);
  }
  assert!(
    assets
      .iter()
      .all(|entry| !entry.reference().path().is_empty())
  );
}

#[test]
fn asset_reference_validation_rejects_unsafe_paths_and_manifest_shape() {
  for path in [
    "",
    "/absolute.png",
    "../parent.png",
    "art/../parent.png",
    ".",
    "art/./asset.png",
    "art//asset.png",
    "art/asset\0.png",
    "C:/asset.png",
    "art\\asset.png",
    "README.md",
    "screenshots/example.png",
    "docs/presentation/asset.png",
    "crates/dreadstep-bevy/src/asset.png",
  ] {
    assert!(
      PresentationAssetReference::new(path).is_none(),
      "{path} should be rejected"
    );
  }
  for path in [
    "assets/valid.wav",
    "art/valid.png",
    "audio/valid.ogg",
    "crates/dreadstep-bevy/assets/valid.wav",
    "crates/dreadstep-bevy/art/valid.png",
    "crates/dreadstep-bevy/audio/valid.ogg",
  ] {
    assert!(
      PresentationAssetReference::new(path).is_some(),
      "{path} should validate"
    );
  }
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
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.world_mut().insert_resource(manifest("two"));
  app.update();
  let after = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  assert_eq!(before.len(), after.len());
  for (old, new) in before.iter().zip(after.iter()) {
    assert_eq!(old.node(), new.node());
    assert_ne!(old.reference(), new.reference());
    assert!(new.reference().path().ends_with("-two.png"));
  }
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
}

#[test]
fn dead_actor_refresh_updates_family_and_retains_node_identity() {
  let mut app = asset_app();
  let before = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  let enemy = before
    .iter()
    .find(|entry| entry.node().node().placeholder() == SceneRenderPlaceholder::Enemy)
    .expect("enemy asset entry should exist")
    .clone();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();
  let after = app
    .world_mut()
    .resource::<PresentationRenderAssetProjection>()
    .entries()
    .to_vec();
  let dead = after
    .iter()
    .find(|entry| entry.node().node().placeholder() == SceneRenderPlaceholder::DeadActor)
    .expect("dead asset entry should exist");
  assert_eq!(dead.node().node_entity(), enemy.node().node_entity());
  assert_eq!(
    dead.node().node().source_entity(),
    enemy.node().node().source_entity()
  );
  assert_eq!(dead.node().node().layer(), enemy.node().node().layer());
  assert_eq!(dead.node().node().order(), enemy.node().node().order());
  assert_eq!(
    dead.node().node().pixel_position(),
    enemy.node().node().pixel_position()
  );
  assert_eq!(dead.reference().path(), "art/dead-one.png");
  assert!(
    !after
      .iter()
      .any(|entry| { entry.node().node().placeholder() == SceneRenderPlaceholder::Enemy })
  );
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
