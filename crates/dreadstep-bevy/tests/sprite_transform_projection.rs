//! Contract tests for the headless Sprite-transform projection.

use bevy::app::App;
use bevy::camera::visibility::Visibility;
use bevy::sprite::Sprite;
use bevy::transform::components::Transform;
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationPlugin,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  SceneRenderEntry, SceneRenderPlaceholder, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn node_app(tile_size: Option<PresentationTileSize>, with_sprite_projection: bool) -> App {
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

fn sized_app() -> App {
  node_app(
    Some(PresentationTileSize::new(32, 32).expect("tile size should validate")),
    true,
  )
}

fn transform_only_app() -> App {
  node_app(
    Some(PresentationTileSize::new(32, 32).expect("tile size should validate")),
    false,
  )
}

fn translation(entry: dreadstep_bevy::SceneBevySpriteTransformEntry) -> Option<(u32, u32)> {
  entry
    .translation()
    .map(|position| (position.x(), position.y()))
}

#[test]
fn projects_complete_map_translations_and_inventory_exclusion() {
  let app = sized_app();
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries();
  assert_eq!(entries.len(), 9);
  assert_eq!(
    entries
      .iter()
      .map(|entry| entry.node().node().placeholder())
      .collect::<Vec<_>>(),
    vec![
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::Terrain,
      SceneRenderPlaceholder::GroundItem,
      SceneRenderPlaceholder::Player,
      SceneRenderPlaceholder::Enemy,
      SceneRenderPlaceholder::Enemy,
      SceneRenderPlaceholder::InventoryItem,
    ]
  );
  let expected = [
    Some((0, 0)),
    Some((32, 0)),
    Some((64, 0)),
    Some((96, 0)),
    Some((0, 0)),
    Some((0, 0)),
    Some((32, 0)),
    Some((64, 0)),
    None,
  ];
  assert_eq!(
    entries.iter().copied().map(translation).collect::<Vec<_>>(),
    expected
  );
  assert_eq!(
    entries
      .last()
      .expect("inventory entry should exist")
      .node()
      .node()
      .key(),
    SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102))
  );
  assert_eq!(
    entries.iter().map(|entry| entry.node()).collect::<Vec<_>>(),
    app
      .world()
      .resource::<PresentationRenderNodeProjection>()
      .entries()
      .to_vec()
  );
}

#[test]
fn refreshes_dead_and_stale_translations_without_replacing_nodes() {
  let mut app = sized_app();
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
  let before_dead = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .find(|entry| {
      entry.node().node().source_entity() == source
        && entry.node().node().key() == SceneSpriteKey::Enemy
    })
    .copied()
    .expect("enemy transform entry should exist");
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
      entry.node().node().source_entity() == source
        && entry.node().node().key() == SceneSpriteKey::DeadActor
    })
    .expect("dead transform entry should exist");
  assert_eq!(dead.node().node_entity(), before_dead.node().node_entity());
  assert_eq!(translation(*dead), Some((32, 0)));

  let mut stale_app = sized_app();
  let before_stale = stale_app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  let stale_entry = before_stale
    .iter()
    .find(|entry| {
      entry.node().node().key() == SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102))
    })
    .copied()
    .expect("inventory transform entry should exist");
  let retained_stale = before_stale
    .iter()
    .filter(|entry| {
      entry.node().node().key() != SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102))
    })
    .copied()
    .collect::<Vec<_>>();
  stale_app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(2),
    })
    .expect("inventory item should be consumable");
  stale_app.update();
  assert_eq!(
    stale_app
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries(),
    retained_stale.as_slice()
  );
  assert!(
    stale_app
      .world()
      .get_entity(stale_entry.node().node_entity())
      .is_err()
  );
}

#[test]
fn colocated_nodes_keep_distinct_translations() {
  let mut app = sized_app();
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
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .iter()
    .filter(|entry| entry.node().node().source_entity() == actor_entity)
    .copied()
    .collect::<Vec<_>>();
  assert_eq!(co_located.len(), 2);
  assert_ne!(
    co_located[0].node().node_entity(),
    co_located[1].node().node_entity()
  );
  assert_eq!(translation(co_located[0]), Some((64, 0)));
  assert_eq!(translation(co_located[1]), Some((64, 0)));
}

#[test]
fn missing_tile_size_leaves_translations_unset() {
  let app = node_app(None, true);
  assert!(
    app
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries()
      .iter()
      .all(|entry| entry.translation().is_none())
  );

  let mut removed_later = sized_app();
  let before = removed_later
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  removed_later
    .world_mut()
    .remove_resource::<PresentationTileSize>();
  removed_later.update();
  assert_eq!(
    removed_later
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries(),
    before.as_slice()
  );
}

#[test]
fn accepted_movement_refreshes_the_same_node_translation() {
  let mut app = sized_app();
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
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(3),
      direction: dreadstep_core::Direction::West,
    })
    .expect("enemy move into the dead actor tile should succeed");
  app.update();
  let after = app
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
  assert_eq!(after.node().node_entity(), before.node().node_entity());
  assert_eq!(translation(after), Some((32, 0)));
}

#[test]
fn plugin_attaches_transform_without_sprite_projection() {
  let app = transform_only_app();
  let entries = app
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  assert!(!entries.is_empty());
  for entry in entries {
    let entity = app
      .world()
      .get_entity(entry.node().node_entity())
      .expect("node entity should exist");
    assert!(entity.get::<Sprite>().is_none());
    assert!(entity.get::<Visibility>().is_none());
    let actual = entity
      .get::<Transform>()
      .expect("plugin should attach a transform");
    if let Some(position) = entry.translation() {
      #[allow(clippy::cast_precision_loss)]
      {
        assert_eq!(
          *actual,
          Transform::from_xyz(
            position.x() as f32 + 16.0,
            position.y() as f32 + 16.0,
            actual.translation.z
          )
        );
      }
    } else {
      assert_eq!(*actual, Transform::default());
    }
  }
}

#[test]
fn missing_resources_and_entities_preserve_transform_projection() {
  let mut missing_runtime = sized_app();
  let before = missing_runtime
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(
    missing_runtime
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries(),
    before.as_slice()
  );

  let mut missing_source = sized_app();
  let before = missing_source
    .world()
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  missing_source
    .world_mut()
    .remove_resource::<PresentationRenderNodeProjection>();
  missing_source.update();
  assert_eq!(
    missing_source
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries(),
    before.as_slice()
  );

  let mut missing_projection = sized_app();
  missing_projection
    .world_mut()
    .remove_resource::<PresentationBevySpriteTransformProjection>();
  missing_projection.update();

  let mut missing_entity = sized_app();
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
    .to_vec();
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
  assert_eq!(
    missing_entity
      .world()
      .resource::<PresentationBevySpriteTransformProjection>()
      .entries(),
    before.as_slice()
  );
}

#[test]
fn transform_projection_does_not_mutate_runtime_or_replay() {
  let mut app = sized_app();
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
