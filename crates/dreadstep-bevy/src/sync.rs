//! Scene-mirror reconciliation from an authoritative presentation snapshot.

use std::collections::{BTreeMap, BTreeSet};

use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use dreadstep_core::Actor;

use crate::{
  PresentationRuntime, PresentationSnapshot, SceneActor, SceneGroundItem, SceneInventoryItem,
  SceneSpriteRole, SceneTile,
};

pub(crate) fn sync_runtime_scene(world: &mut World) {
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  sync_scene(world, &snapshot);
}

pub(crate) fn tile_key(position: dreadstep_core::Position) -> (i32, i32) {
  (position.x(), position.y())
}

pub(crate) fn scene_position(index: usize, width: usize) -> Option<dreadstep_core::Position> {
  if width == 0 {
    return None;
  }
  Some(dreadstep_core::Position::new(
    i32::try_from(index % width).ok()?,
    i32::try_from(index / width).ok()?,
  ))
}

/// Synchronizes a complete core projection into disposable Bevy scene entities.
///
/// Tile entities are keyed by position, actor entities by [`dreadstep_core::ActorId`], and ground or inventory-item
/// entities by globally unique [`dreadstep_core::ItemId`]. Existing entities keep their Bevy identity when their key
/// remains in the snapshot; stale entities are despawned before new keys are spawned. The ECS world
/// is only a presentation mirror and cannot change core state.
pub fn sync_scene(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_tiles: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneTile)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, tile)| {
        entities
          .entry(tile_key(tile.position()))
          .or_default()
          .push(entity);
        entities
      })
  };
  for entities in existing_tiles.values_mut() {
    entities.sort_unstable();
  }
  let Ok(width) = usize::try_from(snapshot.width()) else {
    return;
  };
  let Some(positions) = snapshot
    .tiles()
    .iter()
    .enumerate()
    .map(|(index, _)| scene_position(index, width))
    .collect::<Option<Vec<_>>>()
  else {
    return;
  };
  let expected_tiles: BTreeSet<_> = positions.iter().copied().map(tile_key).collect();
  for (key, entities) in &existing_tiles {
    if expected_tiles.contains(key) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for (position, terrain) in positions
    .iter()
    .copied()
    .zip(snapshot.tiles().iter().copied())
  {
    if let Some(entity) = existing_tiles
      .get(&tile_key(position))
      .and_then(|entities| entities.first())
    {
      scene
        .entity_mut(*entity)
        .insert((SceneTile::new(position, terrain), SceneSpriteRole::Terrain));
    } else {
      scene.spawn((SceneTile::new(position, terrain), SceneSpriteRole::Terrain));
    }
  }

  let mut existing_actors: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneActor)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, actor)| {
        entities.entry(actor.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_actors.values_mut() {
    entities.sort_unstable();
  }
  let expected_actors: BTreeSet<_> = snapshot.actors().iter().map(Actor::id).collect();
  for (actor_id, entities) in &existing_actors {
    if expected_actors.contains(actor_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for actor in snapshot.actors() {
    let scene_actor = SceneActor::from_core(actor);
    let sprite_role = SceneSpriteRole::for_actor(actor);
    if let Some(entity) = existing_actors
      .get(&actor.id())
      .and_then(|entities| entities.first())
    {
      scene.entity_mut(*entity).insert((scene_actor, sprite_role));
    } else {
      scene.spawn((scene_actor, sprite_role));
    }
  }

  sync_ground_items(scene, snapshot);
  sync_inventory_items(scene, snapshot);
}

pub(crate) fn sync_ground_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_ground_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneGroundItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_ground_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_ground_items: BTreeSet<_> = snapshot
    .ground_items()
    .iter()
    .flat_map(|stack| stack.items().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_ground_items {
    if expected_ground_items.contains(item_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for stack in snapshot.ground_items() {
    for (stack_index, item) in stack.items().iter().enumerate() {
      let scene_item = SceneGroundItem::from_core(stack.position(), stack_index, *item);
      if let Some(entity) = existing_ground_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene
          .entity_mut(*entity)
          .insert((scene_item, SceneSpriteRole::GroundItem));
      } else {
        scene.spawn((scene_item, SceneSpriteRole::GroundItem));
      }
    }
  }
}

pub(crate) fn sync_inventory_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_inventory_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneInventoryItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_inventory_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_inventory_items: BTreeSet<_> = snapshot
    .actors()
    .iter()
    .flat_map(|actor| actor.inventory().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_inventory_items {
    if expected_inventory_items.contains(item_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for actor in snapshot.actors() {
    for (inventory_index, item) in actor.inventory().iter().enumerate() {
      let scene_item = SceneInventoryItem::from_core(actor.id(), inventory_index, *item);
      if let Some(entity) = existing_inventory_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene
          .entity_mut(*entity)
          .insert((scene_item, SceneSpriteRole::InventoryItem));
      } else {
        scene.spawn((scene_item, SceneSpriteRole::InventoryItem));
      }
    }
  }
}
