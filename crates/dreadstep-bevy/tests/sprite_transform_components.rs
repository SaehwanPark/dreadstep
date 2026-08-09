//! Contract tests for ECS attachment of checked Sprite-transform translations.

use bevy::app::App;
use bevy::math::Vec3;
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationPlugin,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  SceneRenderEntry, SceneRenderPlaceholder, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, HitPoints, Item, ItemDefinitionId,
  ItemId, Position, Tile, WorldState,
};

fn attachment_app(tile_size: Option<PresentationTileSize>, with_sprite_projection: bool) -> App {
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
  if let Some(tile_size) = tile_size {
    app.insert_resource(tile_size);
  }
  app.insert_resource(PresentationRenderProjection::new());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  if with_sprite_projection {
    app.insert_resource(PresentationBevySpriteProjection::new());
  }
  app.insert_resource(PresentationBevySpriteTransformProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn sized_app(with_sprite_projection: bool) -> App {
  attachment_app(
    Some(PresentationTileSize::new(32, 32).expect("tile size should validate")),
    with_sprite_projection,
  )
}

fn nonzero_y_app() -> App {
  let map = GridMap::from_tiles(2, 2, vec![Tile::Floor; 4]).expect("map should validate");
  let world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 1)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 1),
        HitPoints::new(1),
      ),
      Actor::with_hit_points(
        ActorId::new(3),
        ActorKind::Enemy,
        Position::new(0, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should validate");
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationTileSize::new(32, 24).expect("tile size should validate"));
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

#[allow(clippy::cast_precision_loss)]
fn transform(entry: dreadstep_bevy::SceneBevySpriteTransformEntry) -> Option<Vec3> {
  let depth = match entry.node().node().placeholder() {
    SceneRenderPlaceholder::GroundItem => 1.0,
    SceneRenderPlaceholder::Player
    | SceneRenderPlaceholder::Enemy
    | SceneRenderPlaceholder::DeadActor => 2.0,
    SceneRenderPlaceholder::Terrain | SceneRenderPlaceholder::InventoryItem => 0.0,
  };
  entry
    .translation()
    .map(|position| Vec3::new(position.x() as f32, position.y() as f32, depth))
}

fn node_transform(app: &App, entity: bevy::ecs::entity::Entity) -> Option<Transform> {
  app
    .world()
    .get_entity(entity)
    .ok()?
    .get::<Transform>()
    .copied()
}

#[test]
fn attaches_complete_map_transforms_and_leaves_inventory_unplaced() {
  let app = sized_app(true);
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  assert_eq!(entries.len(), 9);
  for entry in entries {
    let expected = transform(entry).map(Transform::from_translation);
    let actual = node_transform(&app, entry.node().node_entity());
    if entry.node().node().placeholder() == SceneRenderPlaceholder::InventoryItem {
      assert_eq!(actual, Some(Transform::default()));
    } else {
      assert_eq!(actual, expected);
    }
  }
}

#[test]
fn attaches_nonzero_y_transform_in_logical_pixels() {
  let app = nonzero_y_app();
  let (tile_source, actor_source) = app
    .world()
    .resource::<PresentationRenderProjection>()
    .entries()
    .iter()
    .fold(
      (None, None),
      |(tile_source, actor_source), entry| match entry {
        SceneRenderEntry::Terrain { entity, tile, .. }
          if tile.position() == Position::new(0, 1) =>
        {
          (Some(*entity), actor_source)
        }
        SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(2) => {
          (tile_source, Some(*entity))
        }
        _ => (tile_source, actor_source),
      },
    );
  let tile_source = tile_source.expect("nonzero-y tile mirror should exist");
  let actor_source = actor_source.expect("nonzero-y actor mirror should exist");
  let nodes = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let tile_node = nodes
    .iter()
    .find(|entry| {
      entry.node().source_entity() == tile_source
        && entry.node().placeholder() == SceneRenderPlaceholder::Terrain
    })
    .expect("nonzero-y tile node should exist");
  let actor_node = nodes
    .iter()
    .find(|entry| {
      entry.node().source_entity() == actor_source
        && entry.node().placeholder() == SceneRenderPlaceholder::Enemy
    })
    .expect("nonzero-y actor node should exist");
  assert_eq!(
    node_transform(&app, tile_node.node_entity()),
    Some(Transform::from_xyz(0.0, 24.0, 0.0))
  );
  assert_eq!(
    node_transform(&app, actor_node.node_entity()),
    Some(Transform::from_xyz(32.0, 24.0, 2.0))
  );
}

#[test]
fn attachment_works_without_sprite_projection() {
  let app = sized_app(false);
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  assert!(entries.iter().any(|entry| {
    entry.translation().is_some()
      && node_transform(&app, entry.node().node_entity())
        == transform(*entry).map(Transform::from_translation)
  }));
  assert!(
    entries
      .iter()
      .filter(|entry| entry.translation().is_none())
      .all(|entry| {
        node_transform(&app, entry.node().node_entity()) == Some(Transform::default())
      })
  );
}

#[test]
#[allow(clippy::too_many_lines)]
fn refreshes_same_node_transform_on_dead_and_accepted_movement() {
  let mut app = sized_app(true);
  let dead_source = app
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
    .expect("dead actor mirror should exist");
  let dead_before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().node().source_entity() == dead_source
        && entry.node().node().key() == SceneSpriteKey::Enemy
    })
    .copied()
    .expect("dead actor transform entry should exist");
  let actor_source = app
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
    .expect("moving actor mirror should exist");
  let before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().node().source_entity() == actor_source
        && entry.node().node().key() == SceneSpriteKey::Enemy
    })
    .copied()
    .expect("moving actor transform entry should exist");
  assert_eq!(
    node_transform(&app, before.node().node_entity()),
    Some(Transform::from_xyz(64.0, 0.0, 2.0))
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
    .find(|entry| {
      entry.node().node().source_entity() == dead_source
        && entry.node().node().key() == SceneSpriteKey::DeadActor
    })
    .copied()
    .expect("dead transform entry should exist");
  assert_eq!(dead.node().node_entity(), dead_before.node().node_entity());
  assert_eq!(
    node_transform(&app, dead.node().node_entity()),
    Some(Transform::from_xyz(32.0, 0.0, 2.0))
  );
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(3),
      direction: Direction::West,
    })
    .expect("enemy move should succeed");
  app.update();
  let moved = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().node().source_entity() == actor_source
        && entry.node().node().key() == SceneSpriteKey::Enemy
    })
    .copied()
    .expect("moving actor transform entry should remain");
  assert_eq!(moved.node().node_entity(), before.node().node_entity());
  assert_eq!(
    node_transform(&app, moved.node().node_entity()),
    Some(Transform::from_xyz(32.0, 0.0, 2.0))
  );
}

#[test]
fn stale_inventory_removal_retains_complete_map_transforms() {
  let mut app = sized_app(true);
  let before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  let stale = before
    .iter()
    .find(|entry| {
      entry.node().node().key() == SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102))
    })
    .copied()
    .expect("inventory transform entry should exist");
  let retained = before
    .iter()
    .filter_map(|entry| {
      if entry.node().node().key() == SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102)) {
        None
      } else {
        Some((
          entry.node().node_entity(),
          node_transform(&app, entry.node().node_entity()),
        ))
      }
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
  assert!(app.world().get_entity(stale.node().node_entity()).is_err());
  let after = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&app, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, retained);
}

#[test]
fn missing_tile_size_is_freshly_unplaced_but_later_removal_retains_transforms() {
  let fresh = attachment_app(None, true);
  let fresh_entries = fresh
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  assert!(fresh_entries.iter().all(|entry| {
    entry.translation().is_none()
      && node_transform(&fresh, entry.node().node_entity()) == Some(Transform::default())
  }));

  let mut removed = sized_app(true);
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
fn colocated_nodes_receive_independent_transform_components() {
  let mut app = sized_app(true);
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
  for entry in entries {
    let expected_z = match entry.node().node().placeholder() {
      SceneRenderPlaceholder::Player
      | SceneRenderPlaceholder::Enemy
      | SceneRenderPlaceholder::DeadActor => 2.0,
      SceneRenderPlaceholder::Terrain
      | SceneRenderPlaceholder::GroundItem
      | SceneRenderPlaceholder::InventoryItem => 0.0,
    };
    assert_eq!(
      node_transform(&app, entry.node().node_entity()),
      Some(Transform::from_xyz(64.0, 0.0, expected_z))
    );
  }
}

#[test]
#[allow(clippy::too_many_lines)]
fn missing_resources_and_entities_preserve_transform_components() {
  let mut app = sized_app(true);
  let before = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&app, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  let after = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&app, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, before);

  let mut missing_source = sized_app(true);
  let before = missing_source
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&missing_source, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  let after = missing_source
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&missing_source, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  assert_eq!(after, before);

  let mut missing_projection = sized_app(true);
  let before = missing_projection
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&missing_projection, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  missing_projection
    .world_mut()
    .remove_resource::<PresentationBevySpriteTransformProjection>();
  missing_projection.update();
  for (entity, expected) in before {
    assert_eq!(node_transform(&missing_projection, entity), expected);
  }

  let mut missing_entity = sized_app(true);
  let player = missing_entity
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| entry.node().node().key() == SceneSpriteKey::Player)
    .copied()
    .expect("player transform entry should exist");
  let before = missing_entity
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .map(|entry| {
      (
        entry.node().node_entity(),
        node_transform(&missing_entity, entry.node().node_entity()),
      )
    })
    .collect::<Vec<_>>();
  missing_entity
    .world_mut()
    .despawn(player.node().node_entity());
  missing_entity
    .world_mut()
    .remove_resource::<PresentationRenderCommandPlan>();
  missing_entity.update();
  assert!(
    missing_entity
      .world()
      .get_entity(player.node().node_entity())
      .is_err()
  );
  for (entity, expected) in before {
    if entity != player.node().node_entity() {
      assert_eq!(node_transform(&missing_entity, entity), expected);
    }
  }
}

#[test]
fn transform_attachment_does_not_mutate_runtime_or_replay() {
  let mut app = sized_app(true);
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
