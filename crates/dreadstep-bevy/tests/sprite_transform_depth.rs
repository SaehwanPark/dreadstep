//! Contract tests for deterministic ECS Sprite z-layer attachment.

use bevy::app::App;
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationPlugin,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  SceneRenderEntry, SceneRenderLayer,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn depth_app() -> App {
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
  app.insert_resource(PresentationBevySpriteTransformProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn node_transform(app: &App, entity: bevy::ecs::entity::Entity) -> Transform {
  *app
    .world()
    .get_entity(entity)
    .expect("node entity should exist")
    .get::<Transform>()
    .expect("node should have a transform")
}

fn expected_depth(layer: SceneRenderLayer) -> f32 {
  match layer {
    SceneRenderLayer::GroundItem => 1.0,
    SceneRenderLayer::Actor => 2.0,
    SceneRenderLayer::Terrain | SceneRenderLayer::InventoryItem => 0.0,
  }
}

#[test]
fn complete_nodes_receive_layer_depth_without_changing_xy_or_order() {
  let app = depth_app();
  let nodes = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  assert_eq!(nodes.len(), 9);
  assert_eq!(
    nodes
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
      SceneRenderLayer::Actor,
      SceneRenderLayer::InventoryItem,
    ]
  );
  let expected_xy = [
    Some((0.0, 0.0)),
    Some((32.0, 0.0)),
    Some((64.0, 0.0)),
    Some((96.0, 0.0)),
    Some((0.0, 0.0)),
    Some((0.0, 0.0)),
    Some((32.0, 0.0)),
    Some((64.0, 0.0)),
    None,
  ];
  for (entry, expected) in nodes.iter().zip(expected_xy) {
    let actual = node_transform(&app, entry.node_entity());
    match expected {
      Some((x, y)) => assert_eq!(
        actual,
        Transform::from_xyz(x, y, expected_depth(entry.node().layer()))
      ),
      None => assert_eq!(actual, Transform::default()),
    }
  }
}

#[test]
fn dead_refresh_keeps_actor_depth_and_node_identity() {
  let mut app = depth_app();
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
    .expect("dead actor source should exist");
  let before = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().source_entity() == source)
    .copied()
    .expect("dead actor node should exist");
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
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().source_entity() == source)
    .copied()
    .expect("dead actor node should remain");
  assert_eq!(after.node_entity(), before.node_entity());
  assert_eq!(after.node().layer(), SceneRenderLayer::Actor);
  assert_eq!(
    node_transform(&app, after.node_entity()),
    Transform::from_xyz(32.0, 0.0, 2.0)
  );
}

#[test]
fn stale_inventory_despawn_preserves_other_layer_depths() {
  let mut app = depth_app();
  let before = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let stale = before
    .iter()
    .find(|entry| entry.node().layer() == SceneRenderLayer::InventoryItem)
    .copied()
    .expect("inventory node should exist");
  let retained = before
    .iter()
    .filter(|entry| entry.node_entity() != stale.node_entity())
    .map(|entry| {
      (
        entry.node_entity(),
        node_transform(&app, entry.node_entity()),
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
  let after = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node_entity(),
        node_transform(&app, entry.node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, retained);
}

#[test]
fn colocated_tile_and_actor_keep_distinct_layer_depths() {
  let mut app = depth_app();
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
  let nodes = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .filter(|entry| entry.node().source_entity() == actor_entity)
    .copied()
    .collect::<Vec<_>>();
  assert_eq!(nodes.len(), 2);
  assert_eq!(nodes[0].node().layer(), SceneRenderLayer::Terrain);
  assert_eq!(nodes[1].node().layer(), SceneRenderLayer::Actor);
  assert_eq!(
    node_transform(&app, nodes[0].node_entity()),
    Transform::from_xyz(64.0, 0.0, 0.0)
  );
  assert_eq!(
    node_transform(&app, nodes[1].node_entity()),
    Transform::from_xyz(64.0, 0.0, 2.0)
  );
}

#[test]
fn missing_resources_preserve_depth_components_and_authority() {
  let mut app = depth_app();
  let before = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node_entity(),
        node_transform(&app, entry.node_entity()),
      )
    })
    .collect::<Vec<_>>();
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  let after = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node_entity(),
        node_transform(&app, entry.node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, before);
  assert!(app.world().get_resource::<PresentationRuntime>().is_none());
}
