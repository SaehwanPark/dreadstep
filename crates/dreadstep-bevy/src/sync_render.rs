//! Sprite, placeholder, transform, and asset synchronization.

use std::collections::BTreeMap;

use bevy::camera::visibility::Visibility;
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Or, With};
use bevy::ecs::world::World;
use bevy::sprite::Sprite;
use bevy::transform::components::Transform;
use dreadstep_core::Position;

use crate::{
  PresentationAssetManifest, PresentationBevySpriteProjection,
  PresentationBevySpriteTransformProjection, PresentationKeyboardMode,
  PresentationRenderAssetProjection, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationTileSize, PresentationVisibility, SceneActor,
  SceneBevySpriteEntry, SceneBevySpriteTransformEntry, SceneGroundItem, SceneInventoryItem,
  ScenePixelPosition, SceneRenderAssetEntry, SceneRenderCommand, SceneRenderEntry,
  SceneRenderLayer, SceneRenderNode, SceneRenderNodeEntry, SceneRenderPlaceholder,
  SceneSpriteEntry, SceneSpriteRole, SceneTile,
};

pub(crate) fn sync_scene_pixel_positions(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let Some(tile_size) = world.get_resource::<PresentationTileSize>().copied() else {
    return;
  };
  let placements = {
    let mut query = world.query_filtered::<(
      Entity,
      Option<&SceneTile>,
      Option<&SceneActor>,
      Option<&SceneGroundItem>,
    ), Or<(With<SceneTile>, With<SceneActor>, With<SceneGroundItem>)>>();
    query
      .iter(world)
      .map(|(entity, tile, actor, ground_item)| {
        let position = tile
          .map(|tile| tile.position())
          .or_else(|| actor.map(|actor| actor.position()))
          .or_else(|| ground_item.map(|item| item.position()));
        (
          entity,
          position.and_then(|position| tile_size.pixel_position(position)),
        )
      })
      .collect::<Vec<_>>()
  };
  for (entity, pixel_position) in placements {
    let mut entity = world.entity_mut(entity);
    if let Some(pixel_position) = pixel_position {
      entity.insert(pixel_position);
    } else {
      entity.remove::<ScenePixelPosition>();
    }
  }
}

pub(crate) fn sync_render_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let entries = {
    let tile_size = world.get_resource::<PresentationTileSize>().copied();
    let mut query = world.query::<(
      Entity,
      Option<&SceneTile>,
      Option<&SceneActor>,
      Option<&SceneGroundItem>,
      Option<&SceneInventoryItem>,
      Option<&ScenePixelPosition>,
    )>();
    let mut keyed: BTreeMap<_, (Entity, SceneRenderEntry)> = BTreeMap::new();
    for (entity, tile, actor, ground_item, inventory_item, pixel_position) in query.iter(world) {
      for (key, entry) in render_entries(
        entity,
        tile.copied(),
        actor.copied(),
        ground_item.copied(),
        inventory_item.copied(),
        pixel_position.copied(),
        tile_size,
      ) {
        match keyed.entry(key) {
          std::collections::btree_map::Entry::Occupied(mut retained) => {
            if entity < retained.get().0 {
              retained.insert((entity, entry));
            }
          }
          std::collections::btree_map::Entry::Vacant(slot) => {
            slot.insert((entity, entry));
          }
        }
      }
    }
    keyed
      .into_values()
      .map(|(_, entry)| entry)
      .collect::<Vec<_>>()
  };
  let Some(mut projection) = world.get_resource_mut::<PresentationRenderProjection>() else {
    return;
  };
  projection.entries = entries;
}

pub(crate) fn sync_sprite_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let entries = world
    .get_resource::<PresentationRenderProjection>()
    .map(|projection| {
      projection
        .entries()
        .iter()
        .copied()
        .map(SceneSpriteEntry::from_render_entry)
        .collect::<Vec<_>>()
    });
  let Some(entries) = entries else {
    return;
  };
  let Some(mut projection) = world.get_resource_mut::<PresentationSpriteProjection>() else {
    return;
  };
  projection.entries = entries;
}

pub(crate) fn sync_render_command_plan(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let commands = world
    .get_resource::<PresentationSpriteProjection>()
    .map(|projection| {
      let mut commands = projection
        .entries()
        .iter()
        .copied()
        .enumerate()
        .map(|(order, entry)| SceneRenderCommand::from_sprite_entry(entry, order))
        .collect::<Vec<_>>();
      commands.sort_by_key(|command| (command.layer(), command.order()));
      commands
    });
  let Some(commands) = commands else {
    return;
  };
  let Some(mut plan) = world.get_resource_mut::<PresentationRenderCommandPlan>() else {
    return;
  };
  plan.commands = commands;
}

pub(crate) fn sync_render_nodes(world: &mut World) {
  if world
    .get_resource::<PresentationRenderCommandPlan>()
    .is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
  {
    return;
  }
  let commands = world
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .to_vec();
  let visibility = world.get_resource::<PresentationVisibility>().cloned();
  let existing = {
    let mut query = world.query::<(Entity, &SceneRenderNode)>();
    query
      .iter(world)
      .map(|(entity, node)| (entity, *node))
      .collect::<Vec<_>>()
  };
  let mut retained = Vec::new();
  let mut entries = Vec::with_capacity(commands.len());
  for command in commands {
    let visible = visibility.as_ref().is_none_or(|visibility| {
      render_entry_position(command.sprite_entry().render_entry())
        .is_none_or(|position| visibility.is_visible(position))
    });
    let node = SceneRenderNode::from_command(command, visible);
    let retained_entity = existing
      .iter()
      .find(|(entity, existing_node)| {
        !retained.contains(entity)
          && existing_node.source_entity() == node.source_entity()
          && existing_node.layer() == node.layer()
      })
      .map(|(entity, _)| *entity);
    let node_entity = retained_entity.unwrap_or_else(|| world.spawn(node).id());
    world.entity_mut(node_entity).insert(node);
    retained.push(node_entity);
    entries.push(SceneRenderNodeEntry { node_entity, node });
  }
  for (entity, _) in existing {
    if !retained.contains(&entity) {
      let _ = world.despawn(entity);
    }
  }
  world
    .resource_mut::<PresentationRenderNodeProjection>()
    .entries = entries;
}

pub(crate) fn render_entry_position(entry: SceneRenderEntry) -> Option<Position> {
  match entry {
    SceneRenderEntry::Terrain { tile, .. } => Some(tile.position()),
    SceneRenderEntry::Actor { actor, .. } => Some(actor.position()),
    SceneRenderEntry::GroundItem { item, .. } => Some(item.position()),
    SceneRenderEntry::InventoryItem { .. } => None,
  }
}

pub(crate) fn sync_bevy_sprite_projection(world: &mut World) {
  if world
    .get_resource::<PresentationRenderNodeProjection>()
    .is_none()
    || world
      .get_resource::<PresentationBevySpriteProjection>()
      .is_none()
  {
    return;
  }
  let tile_size = world.get_resource::<PresentationTileSize>().copied();
  let entries = world
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .copied()
    .map(|node| SceneBevySpriteEntry {
      sprite: placeholder_sprite(node.node().placeholder(), tile_size),
      node,
    })
    .collect::<Vec<_>>();
  world
    .resource_mut::<PresentationBevySpriteProjection>()
    .entries = entries;
}

pub(crate) fn sync_bevy_sprite_transform_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
    || world
      .get_resource::<PresentationBevySpriteTransformProjection>()
      .is_none()
  {
    return;
  }
  let entries = world
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .copied()
    .map(|node| SceneBevySpriteTransformEntry {
      translation: node.node().pixel_position(),
      node,
    })
    .collect::<Vec<_>>();
  world
    .resource_mut::<PresentationBevySpriteTransformProjection>()
    .entries = entries;
}

pub(crate) fn sync_sprite_node_components(world: &mut World) {
  if world
    .get_resource::<PresentationRenderNodeProjection>()
    .is_none()
    || world
      .get_resource::<PresentationBevySpriteProjection>()
      .is_none()
  {
    return;
  }
  let entries = world
    .resource::<PresentationBevySpriteProjection>()
    .entries()
    .to_vec();
  let external_desktop = world
    .get_resource::<PresentationKeyboardMode>()
    .is_some_and(|mode| *mode == PresentationKeyboardMode::External);
  for entry in entries {
    let Ok(mut entity) = world.get_entity_mut(entry.node().node_entity()) else {
      continue;
    };
    entity.insert(entry.sprite().clone());
    if (external_desktop
      && entry.node().node().placeholder() == SceneRenderPlaceholder::InventoryItem)
      || !entry.node().node().is_visible()
    {
      entity.insert(Visibility::Hidden);
    } else {
      entity.insert(Visibility::Inherited);
    }
  }
}

pub(crate) fn sync_sprite_transform_components(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
    || world
      .get_resource::<PresentationBevySpriteTransformProjection>()
      .is_none()
  {
    return;
  }
  let tile_size = world.get_resource::<PresentationTileSize>().copied();
  let entries = world
    .resource::<PresentationBevySpriteTransformProjection>()
    .entries()
    .to_vec();
  for entry in entries {
    let Ok(mut entity) = world.get_entity_mut(entry.node().node_entity()) else {
      continue;
    };
    let Some(position) = entry.translation() else {
      entity.insert(Transform::default());
      continue;
    };
    let Some(tile_size) = tile_size else {
      // A later tile-size removal retains the last checked ECS placement. Fresh unplaced entries
      // have no translation and take the default path above.
      continue;
    };
    entity.insert(transform_from_pixel_position(
      position,
      entry.node().node().layer(),
      tile_size,
    ));
  }
}

pub(crate) fn transform_from_pixel_position(
  position: ScenePixelPosition,
  layer: SceneRenderLayer,
  tile_size: PresentationTileSize,
) -> Transform {
  // Bevy's Transform API stores f32 values; the checked integer pixel origin remains available in
  // ScenePixelPosition and the half-extents are a deliberate adapter conversion at the ECS
  // boundary. Odd dimensions intentionally produce deterministic half-pixel centers.
  #[allow(clippy::cast_precision_loss)]
  {
    Transform::from_xyz(
      position.x() as f32 + tile_size.width() as f32 / 2.0,
      position.y() as f32 + tile_size.height() as f32 / 2.0,
      layer_depth(layer),
    )
  }
}

const fn layer_depth(layer: SceneRenderLayer) -> f32 {
  match layer {
    SceneRenderLayer::GroundItem => 1.0,
    SceneRenderLayer::Actor => 2.0,
    SceneRenderLayer::Terrain | SceneRenderLayer::InventoryItem => 0.0,
  }
}

pub(crate) fn placeholder_sprite(
  placeholder: SceneRenderPlaceholder,
  tile_size: Option<PresentationTileSize>,
) -> Sprite {
  let color = match placeholder {
    SceneRenderPlaceholder::Terrain => Color::srgb(0.18, 0.18, 0.18),
    SceneRenderPlaceholder::Player => Color::srgb(0.1, 0.8, 0.3),
    SceneRenderPlaceholder::Enemy => Color::srgb(0.8, 0.2, 0.2),
    SceneRenderPlaceholder::DeadActor => Color::srgb(0.35, 0.35, 0.35),
    SceneRenderPlaceholder::GroundItem => Color::srgb(0.8, 0.65, 0.15),
    SceneRenderPlaceholder::InventoryItem => Color::srgb(0.2, 0.5, 0.9),
  };
  Sprite {
    color,
    custom_size: tile_size.map(PresentationTileSize::sprite_size),
    ..Default::default()
  }
}

pub(crate) fn sync_render_asset_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
    || world.get_resource::<PresentationAssetManifest>().is_none()
  {
    return;
  }
  let nodes = world
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let manifest = world.resource::<PresentationAssetManifest>();
  let entries = nodes
    .into_iter()
    .map(|node| SceneRenderAssetEntry {
      reference: manifest.reference(node.node().placeholder()).clone(),
      node,
    })
    .collect::<Vec<_>>();
  let Some(mut projection) = world.get_resource_mut::<PresentationRenderAssetProjection>() else {
    return;
  };
  projection.entries = entries;
}

pub(crate) fn render_entries(
  entity: Entity,
  tile: Option<SceneTile>,
  actor: Option<SceneActor>,
  ground_item: Option<SceneGroundItem>,
  inventory_item: Option<SceneInventoryItem>,
  pixel_position: Option<ScenePixelPosition>,
  tile_size: Option<PresentationTileSize>,
) -> Vec<((u8, i32, i32, u32), SceneRenderEntry)> {
  let pixel_position_for = |position: Position| {
    tile_size.map_or(pixel_position, |tile_size| {
      tile_size.pixel_position(position)
    })
  };
  let mut entries = Vec::new();
  if let Some(tile) = tile {
    entries.push((
      (0, tile.position().x(), tile.position().y(), 0),
      SceneRenderEntry::Terrain {
        entity,
        tile,
        role: SceneSpriteRole::Terrain,
        pixel_position: pixel_position_for(tile.position()),
      },
    ));
  }
  if let Some(actor) = actor {
    entries.push((
      (1, 0, 0, actor.id().value()),
      SceneRenderEntry::Actor {
        entity,
        actor,
        role: SceneSpriteRole::for_scene_actor(actor),
        pixel_position: pixel_position_for(actor.position()),
      },
    ));
  }
  if let Some(item) = ground_item {
    entries.push((
      (2, 0, 0, item.id().value()),
      SceneRenderEntry::GroundItem {
        entity,
        item,
        role: SceneSpriteRole::GroundItem,
        pixel_position: pixel_position_for(item.position()),
      },
    ));
  }
  if let Some(item) = inventory_item {
    entries.push((
      (3, 0, 0, item.id().value()),
      SceneRenderEntry::InventoryItem {
        entity,
        item,
        role: SceneSpriteRole::InventoryItem,
      },
    ));
  }
  entries
}
