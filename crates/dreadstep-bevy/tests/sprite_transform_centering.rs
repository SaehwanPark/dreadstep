//! Contract tests for centered ECS Sprite-transform placement.

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationPlugin,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  SceneRenderEntry, SceneRenderLayer, SceneRenderPlaceholder, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldState,
};

fn centered_app(tile_size: Option<PresentationTileSize>) -> App {
  let map = GridMap::from_tiles(2, 2, vec![Tile::Floor; 4]).expect("map should validate");
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
        Position::new(1, 1),
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
  if let Some(tile_size) = tile_size {
    app.insert_resource(tile_size);
  }
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

fn node_transform(app: &App, entity: Entity) -> Option<Transform> {
  app
    .world()
    .get_entity(entity)
    .ok()?
    .get::<Transform>()
    .copied()
}

fn node_transforms(app: &App) -> Vec<(Entity, Option<Transform>)> {
  app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(app, entry.node().node_entity()),
      )
    })
    .collect()
}

fn depth(layer: SceneRenderLayer) -> f32 {
  match layer {
    SceneRenderLayer::GroundItem => 1.0,
    SceneRenderLayer::Actor => 2.0,
    SceneRenderLayer::Terrain | SceneRenderLayer::InventoryItem => 0.0,
  }
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn centers_rectangular_nodes_and_preserves_integer_origins() {
  let app = centered_app(PresentationTileSize::new(32, 24));
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  assert_eq!(entries.len(), 9);
  for entry in entries {
    let actual = node_transform(&app, entry.node().node_entity()).expect("node transform exists");
    if let Some(position) = entry.translation() {
      assert_eq!(
        actual,
        Transform::from_xyz(
          position.x() as f32 + 16.0,
          position.y() as f32 + 12.0,
          depth(entry.node().node().layer()),
        )
      );
    } else {
      assert_eq!(actual, Transform::default());
      assert_eq!(entry.node().node().layer(), SceneRenderLayer::InventoryItem);
    }
  }
  let origins = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .filter_map(|entry| match entry {
      SceneRenderEntry::Terrain {
        tile,
        pixel_position,
        ..
      } => Some((tile.position(), *pixel_position)),
      _ => None,
    })
    .collect::<Vec<_>>();
  assert_eq!(origins[0].1.expect("origin is projected").x(), 0);
  assert_eq!(origins[0].1.expect("origin is projected").y(), 0);
  assert_eq!(origins[3].1.expect("origin is projected").x(), 32);
  assert_eq!(origins[3].1.expect("origin is projected").y(), 24);
}

#[test]
#[allow(clippy::cast_precision_loss)]
fn centers_odd_tile_dimensions_with_deterministic_half_pixels() {
  let app = centered_app(PresentationTileSize::new(31, 25));
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries();
  let terrain = entries
    .iter()
    .find(|entry| entry.node().node().placeholder() == SceneRenderPlaceholder::Terrain)
    .expect("terrain entry exists");
  assert_eq!(
    node_transform(&app, terrain.node().node_entity()),
    Some(Transform::from_xyz(15.5, 12.5, 0.0))
  );
  let actor = entries
    .iter()
    .find(|entry| entry.node().node().key() == SceneSpriteKey::Enemy)
    .expect("enemy entry exists");
  assert_eq!(
    node_transform(&app, actor.node().node_entity()),
    Some(Transform::from_xyz(46.5, 12.5, 2.0))
  );
}

#[test]
fn refreshes_centered_transforms_on_death_and_movement() {
  let mut app = centered_app(PresentationTileSize::new(32, 24));
  let actor_three_source = app
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
    .expect("actor three mirror exists");
  let actor_two_source = app
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
    .expect("actor two mirror exists");
  let before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().node().source_entity() == actor_three_source)
    .copied()
    .expect("actor three transform exists");
  let dead_before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().node().source_entity() == actor_two_source)
    .copied()
    .expect("actor two transform exists");
  assert_eq!(
    node_transform(&app, before.node().node_entity()),
    Some(Transform::from_xyz(48.0, 36.0, 2.0))
  );

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
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().node().key() == SceneSpriteKey::DeadActor)
    .copied()
    .expect("dead transform exists");
  assert_eq!(dead.node().node_entity(), dead_before.node().node_entity());
  assert_eq!(
    node_transform(&app, dead.node().node_entity()),
    Some(Transform::from_xyz(48.0, 12.0, 2.0))
  );

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(3),
      direction: Direction::West,
    })
    .expect("actor three move should succeed");
  app.update();
  let moved = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().node().source_entity() == actor_three_source)
    .copied()
    .expect("moved transform exists");
  assert_eq!(moved.node().node_entity(), before.node().node_entity());
  assert_eq!(
    node_transform(&app, moved.node().node_entity()),
    Some(Transform::from_xyz(16.0, 36.0, 2.0))
  );
}

#[test]
fn fresh_missing_size_is_default_and_later_removal_preserves_centered_transforms() {
  let fresh = centered_app(None);
  let fresh_entries = fresh
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries();
  assert!(fresh_entries.iter().all(|entry| {
    node_transform(&fresh, entry.node().node_entity()) == Some(Transform::default())
  }));

  let mut removed = centered_app(PresentationTileSize::new(32, 24));
  let before = removed
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&removed, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  removed
    .world_mut()
    .remove_resource::<PresentationTileSize>();
  removed.update();
  let after = removed
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&removed, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, before);
}

#[test]
fn colocated_tile_and_actor_keep_independent_centered_nodes() {
  let mut app = centered_app(PresentationTileSize::new(32, 24));
  let (tile_entity, tile) = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Terrain { entity, tile, .. } if tile.position() == Position::new(1, 1) => {
        Some((*entity, *tile))
      }
      _ => None,
    })
    .expect("co-located tile exists");
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
    .expect("co-located actor exists");
  app.world_mut().despawn(tile_entity);
  app.world_mut().entity_mut(actor_entity).insert(tile);
  app.update();
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .filter(|entry| entry.node().node().source_entity() == actor_entity)
    .copied()
    .collect::<Vec<_>>();
  assert_eq!(entries.len(), 2);
  assert_ne!(
    entries[0].node().node_entity(),
    entries[1].node().node_entity()
  );
  assert!(entries.iter().all(|entry| {
    node_transform(&app, entry.node().node_entity())
      == Some(Transform::from_xyz(
        48.0,
        36.0,
        depth(entry.node().node().layer()),
      ))
  }));
}

#[test]
fn centered_attachment_preserves_runtime_authority() {
  let mut app = centered_app(PresentationTileSize::new(32, 24));
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

#[test]
fn missing_runtime_source_and_destination_preserve_centered_components() {
  let mut missing_runtime = centered_app(PresentationTileSize::new(32, 24));
  let before = node_transforms(&missing_runtime);
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(node_transforms(&missing_runtime), before);

  let mut missing_source = centered_app(PresentationTileSize::new(32, 24));
  let before = node_transforms(&missing_source);
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  assert_eq!(node_transforms(&missing_source), before);

  let mut missing_destination = centered_app(PresentationTileSize::new(32, 24));
  let before = node_transforms(&missing_destination);
  missing_destination
    .world_mut()
    .remove_resource::<PresentationBevySpriteTransformProjection>();
  missing_destination.update();
  let after = before
    .iter()
    .map(|(entity, _)| (*entity, node_transform(&missing_destination, *entity)))
    .collect::<Vec<_>>();
  assert_eq!(after, before);
}
