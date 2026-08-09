//! Contract tests for deterministic placeholder render-node reconciliation.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRenderCommandPlan, PresentationRenderNodeProjection,
  PresentationRenderProjection, PresentationRuntime, PresentationSpriteProjection,
  PresentationState, PresentationTileSize, SceneRenderLayer, SceneRenderPlaceholder,
  SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn bootstrap_app() -> App {
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
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

#[test]
fn bootstrap_projects_placeholder_nodes_in_command_order() {
  let mut app = bootstrap_app();
  let entries = app
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  assert_eq!(entries.len(), 8);
  assert_eq!(
    entries
      .iter()
      .map(|entry| entry.node().layer())
      .collect::<Vec<_>>(),
    vec![
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::GroundItem,
      SceneRenderLayer::Actor,
      SceneRenderLayer::Actor,
      SceneRenderLayer::InventoryItem,
    ]
  );
  assert_eq!(
    entries
      .iter()
      .map(|entry| entry.node().placeholder())
      .collect::<Vec<_>>(),
    vec![
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::GroundItem,
      SceneRenderPlaceholder::Player,
      SceneRenderPlaceholder::Enemy,
      SceneRenderPlaceholder::InventoryItem,
    ]
  );
  assert!(
    entries
      .iter()
      .all(|entry| entry.node_entity() != entry.node().source_entity())
  );
  assert_eq!(
    entries
      .last()
      .expect("inventory node should exist")
      .node()
      .pixel_position(),
    None
  );
}

#[test]
fn bootstrap_refreshes_dead_node_and_retains_source_identity() {
  let mut app = bootstrap_app();
  let before = app
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let enemy = before
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::Enemy)
    .copied()
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
  let entries = app
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let dead = entries
    .iter()
    .find(|entry| entry.node().key() == SceneSpriteKey::DeadActor)
    .copied()
    .expect("dead node should exist");
  assert_eq!(dead.node().source_entity(), enemy.node().source_entity());
  assert_eq!(dead.node_entity(), enemy.node_entity());
  assert!(
    !entries
      .iter()
      .any(|entry| entry.node().key() == SceneSpriteKey::Enemy)
  );
}

#[test]
fn missing_source_and_runtime_preserve_nodes_independently() {
  let mut source_absent = bootstrap_app();
  let source_before = source_absent
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  source_absent
    .world_mut()
    .remove_resource::<PresentationRenderCommandPlan>();
  source_absent.update();
  assert_eq!(
    source_absent
      .world()
      .resource::<PresentationRenderNodeProjection>()
      .entries(),
    source_before.as_slice()
  );

  let mut runtime_absent = bootstrap_app();
  let runtime_before = runtime_absent
    .world_mut()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  runtime_absent
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  runtime_absent.update();
  assert_eq!(
    runtime_absent
      .world()
      .resource::<PresentationRenderNodeProjection>()
      .entries(),
    runtime_before.as_slice()
  );
}

#[test]
fn missing_node_projection_resource_is_a_safe_noop() {
  let mut app = bootstrap_app();
  app
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  app.update();
  assert!(
    app
      .world()
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
  );
}
