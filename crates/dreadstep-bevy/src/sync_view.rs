//! Focus, camera, window, viewport, and visibility synchronization.

use std::collections::{BTreeSet, VecDeque};

use bevy::camera::Camera2d;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::transform::components::Transform;
use bevy::window::{PrimaryWindow, Window, WindowResolution};
use dreadstep_core::{Actor, Direction, Position, Tile};

use crate::{
  PresentationCamera, PresentationFocus, PresentationInput, PresentationRuntime,
  PresentationSnapshot, PresentationTileSize, PresentationViewport, PresentationVisibility,
  PresentationWindow, SceneActor, SceneCamera, SceneCameraState, SceneFocus, ScenePixelPosition,
  SceneViewport, SceneViewportState, SceneWindow, SceneWindowState,
};

pub(crate) fn sync_visibility(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    if let Some(mut visibility) = world.get_resource_mut::<PresentationVisibility>() {
      visibility.active = false;
      visibility.positions.clear();
    }
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    if let Some(mut visibility) = world.get_resource_mut::<PresentationVisibility>() {
      visibility.active = false;
      visibility.positions.clear();
    }
    return;
  };
  let Some(origin) = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position)
  else {
    if let Some(mut visibility) = world.get_resource_mut::<PresentationVisibility>() {
      visibility.active = false;
      visibility.positions.clear();
      visibility.actor = actor;
    }
    return;
  };
  let Some(mut visibility) = world.get_resource_mut::<PresentationVisibility>() else {
    return;
  };
  visibility.actor = actor;
  visibility.positions = visible_positions(&snapshot, origin, visibility.radius);
  visibility.active = true;
}

pub(crate) fn visible_positions(
  snapshot: &PresentationSnapshot,
  origin: Position,
  radius: u32,
) -> Vec<Position> {
  if !snapshot_tile(snapshot, origin).is_some_and(Tile::is_walkable) {
    return Vec::new();
  }
  let mut queue = VecDeque::from([(origin, 0_u32)]);
  let mut visited_walkable = BTreeSet::from([(origin.x(), origin.y())]);
  let mut visible = BTreeSet::new();
  while let Some((position, distance)) = queue.pop_front() {
    visible.insert((position.x(), position.y()));
    for direction in [
      Direction::North,
      Direction::South,
      Direction::West,
      Direction::East,
    ] {
      let neighbor = position.translated(direction);
      match snapshot_tile(snapshot, neighbor) {
        Some(Tile::Wall | Tile::Door | Tile::Breakable) => {
          visible.insert((neighbor.x(), neighbor.y()));
        }
        Some(Tile::Floor | Tile::Cover | Tile::OpenDoor | Tile::Trap | Tile::ChillTrap)
          if distance < radius && visited_walkable.insert((neighbor.x(), neighbor.y())) =>
        {
          queue.push_back((neighbor, distance + 1));
        }
        Some(Tile::Floor | Tile::Cover | Tile::OpenDoor | Tile::Trap | Tile::ChillTrap) | None => {}
      }
    }
  }
  let mut positions = visible
    .into_iter()
    .map(|(x, y)| Position::new(x, y))
    .collect::<Vec<_>>();
  positions.sort_by_key(|position| (position.y(), position.x()));
  positions
}

pub(crate) fn snapshot_tile(snapshot: &PresentationSnapshot, position: Position) -> Option<Tile> {
  let x = usize::try_from(position.x()).ok()?;
  let y = usize::try_from(position.y()).ok()?;
  let width = usize::try_from(snapshot.width()).ok()?;
  let height = usize::try_from(snapshot.height()).ok()?;
  if x >= width || y >= height {
    return None;
  }
  snapshot
    .tiles()
    .get(y.checked_mul(width)?.checked_add(x)?)
    .copied()
}

pub(crate) fn sync_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let position = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut focus) = world.get_resource_mut::<PresentationFocus>() else {
    return;
  };
  focus.actor = actor;
  focus.position = position;
}

pub(crate) fn sync_scene_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  if world.get_resource::<PresentationFocus>().is_none() {
    return;
  }
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let actor_is_known = snapshot.actors().iter().any(|record| record.id() == actor);
  let (marked_entities, target_entity) = {
    let mut query = world.query::<(Entity, Option<&SceneActor>, Option<&SceneFocus>)>();
    query.iter(world).fold(
      (Vec::new(), None),
      |(mut marked_entities, target_entity), (entity, scene_actor, marker)| {
        if marker.is_some() {
          marked_entities.push(entity);
        }
        let target_entity = target_entity.or_else(|| {
          actor_is_known
            .then(|| {
              scene_actor
                .filter(|record| record.id() == actor)
                .map(|_| entity)
            })
            .flatten()
        });
        (marked_entities, target_entity)
      },
    )
  };
  for entity in marked_entities {
    if Some(entity) != target_entity {
      world.entity_mut(entity).remove::<SceneFocus>();
    }
  }
  if let Some(entity) = target_entity {
    world.entity_mut(entity).insert(SceneFocus);
  }
}

pub(crate) fn sync_camera(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let center = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut camera) = world.get_resource_mut::<PresentationCamera>() else {
    return;
  };
  camera.actor = actor;
  camera.center = center;
}

pub(crate) fn sync_scene_camera(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
  {
    return;
  }
  let Some(camera) = world.get_resource::<PresentationCamera>() else {
    return;
  };
  let center = camera.center();
  let Some(center) = center else {
    let mut query = world.query::<(Entity, &SceneCamera)>();
    let stale_entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    for entity in stale_entities {
      let _ = world.despawn(entity);
    }
    world.insert_resource(SceneCameraState::default());
    return;
  };
  let mut query = world.query::<(Entity, &SceneCamera)>();
  let mut existing = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  existing.sort_unstable();
  let retained = world
    .get_resource::<SceneCameraState>()
    .and_then(|state| state.entity)
    .filter(|entity| existing.contains(entity))
    .or_else(|| existing.first().copied());
  if let Some(entity) = retained {
    world.entity_mut(entity).insert(SceneCamera::new(center));
    for stale in existing
      .into_iter()
      .filter(|candidate| *candidate != entity)
    {
      let _ = world.despawn(stale);
    }
    world.insert_resource(SceneCameraState {
      entity: Some(entity),
    });
  } else {
    let entity = world.spawn(SceneCamera::new(center)).id();
    world.insert_resource(SceneCameraState {
      entity: Some(entity),
    });
  }
}

pub(crate) fn sync_scene_camera_components(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
    || world.get_resource::<PresentationCamera>().is_none()
  {
    return;
  }
  let mut query = world.query::<(Entity, &SceneCamera)>();
  let tile_size = world.get_resource::<PresentationTileSize>().copied();
  let entities = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  for entity in entities {
    let Ok(mut entity) = world.get_entity_mut(entity) else {
      continue;
    };
    entity.insert(Camera2d);
    let Some(tile_size) = tile_size else {
      continue;
    };
    let Some(center) = entity.get::<SceneCamera>().map(|camera| camera.center()) else {
      continue;
    };
    let Some(pixel_position) = tile_size.pixel_position(center) else {
      continue;
    };
    entity.insert(camera_transform_from_pixel_position(
      pixel_position,
      tile_size,
    ));
  }
}

pub(crate) fn sync_scene_window_components(world: &mut World) {
  let Some(request) = world.get_resource::<PresentationWindow>().copied() else {
    return;
  };
  let primary = {
    let mut query = world.query::<(Entity, &PrimaryWindow)>();
    let mut entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    entities.sort_unstable();
    entities.first().copied()
  };
  let existing = {
    let mut query = world.query::<(Entity, &SceneWindow)>();
    let mut entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    entities.sort_unstable();
    entities
  };
  let state_entity = world
    .get_resource::<SceneWindowState>()
    .and_then(|state| state.entity)
    .filter(|entity| world.get::<SceneWindow>(*entity).is_some());
  let retained = primary
    .or(state_entity)
    .or_else(|| existing.first().copied());
  let retained = retained.unwrap_or_else(|| world.spawn_empty().id());
  for entity in existing {
    if entity != retained {
      world.despawn(entity);
    }
  }
  let window = Window {
    resolution: WindowResolution::new(request.physical_width(), request.physical_height())
      .with_scale_factor_override(window_scale_factor(request.pixel_scale())),
    ..Default::default()
  };
  world
    .entity_mut(retained)
    .insert((SceneWindow::new(request), window));
  world.insert_resource(SceneWindowState {
    entity: Some(retained),
  });
}

// Bevy's WindowResolution API accepts an f32 scale; SceneWindow retains the exact integer request.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn window_scale_factor(pixel_scale: u32) -> f32 {
  pixel_scale as f32
}

pub(crate) fn camera_transform_from_pixel_position(
  position: ScenePixelPosition,
  tile_size: PresentationTileSize,
) -> Transform {
  // Bevy's Transform API stores f32 values; the checked integer origin remains available in
  // ScenePixelPosition, and tile half-extents intentionally use the existing presentation adapter.
  #[allow(clippy::cast_precision_loss)]
  {
    Transform::from_xyz(
      position.x() as f32 + tile_size.width() as f32 / 2.0,
      position.y() as f32 + tile_size.height() as f32 / 2.0,
      0.0,
    )
  }
}

pub(crate) fn sync_viewport(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none() {
    return;
  }
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let Some(camera) = world.get_resource::<PresentationCamera>() else {
    return;
  };
  let Some(center) = camera.center() else {
    let Some(mut viewport) = world.get_resource_mut::<PresentationViewport>() else {
      return;
    };
    viewport.origin = None;
    viewport.effective_width = 0;
    viewport.effective_height = 0;
    return;
  };
  let Some((requested_width, requested_height)) = world
    .get_resource::<PresentationViewport>()
    .map(|viewport| (viewport.width(), viewport.height()))
  else {
    return;
  };
  let effective_width = requested_width.min(snapshot.width());
  let effective_height = requested_height.min(snapshot.height());
  let origin = Position::new(
    clamped_viewport_axis(center.x(), effective_width, snapshot.width()),
    clamped_viewport_axis(center.y(), effective_height, snapshot.height()),
  );
  let Some(mut viewport) = world.get_resource_mut::<PresentationViewport>() else {
    return;
  };
  viewport.origin = Some(origin);
  viewport.effective_width = effective_width;
  viewport.effective_height = effective_height;
}

pub(crate) fn clamped_viewport_axis(center: i32, extent: u32, map_extent: u32) -> i32 {
  let half_extent = i64::from(extent / 2);
  let desired = i64::from(center) - half_extent;
  let maximum = i64::from(map_extent.saturating_sub(extent));
  i32::try_from(desired.clamp(0, maximum)).unwrap_or_default()
}

pub(crate) fn sync_scene_viewport(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
  {
    return;
  }
  let Some(viewport) = world.get_resource::<PresentationViewport>() else {
    return;
  };
  let Some(origin) = viewport.origin() else {
    let mut query = world.query::<(Entity, &SceneViewport)>();
    let stale_entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    for entity in stale_entities {
      let _ = world.despawn(entity);
    }
    world.insert_resource(SceneViewportState::default());
    return;
  };
  let dimensions = (viewport.effective_width(), viewport.effective_height());
  let mut query = world.query::<(Entity, &SceneViewport)>();
  let mut existing = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  existing.sort_unstable();
  let retained = world
    .get_resource::<SceneViewportState>()
    .and_then(|state| state.entity)
    .filter(|entity| existing.contains(entity))
    .or_else(|| existing.first().copied());
  let scene_viewport = SceneViewport::new(origin, dimensions.0, dimensions.1);
  if let Some(entity) = retained {
    world.entity_mut(entity).insert(scene_viewport);
    for stale in existing
      .into_iter()
      .filter(|candidate| *candidate != entity)
    {
      let _ = world.despawn(stale);
    }
    world.insert_resource(SceneViewportState {
      entity: Some(entity),
    });
  } else {
    let entity = world.spawn(scene_viewport).id();
    world.insert_resource(SceneViewportState {
      entity: Some(entity),
    });
  }
}
