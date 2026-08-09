//! Typed reversible render-boundary projection behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRenderProjection, PresentationRuntime, PresentationState,
  PresentationTileSize, SceneActor, SceneGroundItem, SceneInventoryItem, ScenePixelPosition,
  SceneRenderEntry, SceneSpriteRole, SceneTile,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn world_state(with_items: bool) -> WorldState {
  let map = GridMap::from_tiles(
    3,
    2,
    vec![
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
    ],
  )
  .expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 1)),
    ],
  )
  .expect("world should validate");
  if with_items {
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(1), ItemDefinitionId::new(101)),
      )
      .expect("ground item should be given");
    world
      .drop_item(ActorId::new(1), ItemId::new(1))
      .expect("ground item should be dropped");
    world
      .give_item(
        ActorId::new(1),
        Item::new(ItemId::new(2), ItemDefinitionId::new(102)),
      )
      .expect("inventory item should be given");
  }
  world
}

fn item_state() -> PresentationState {
  PresentationState::new(7, world_state(true))
}

fn empty_state() -> PresentationState {
  PresentationState::new(7, world_state(false))
}

fn app_with_projection() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size should validate"));
  app.insert_resource(PresentationRenderProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

fn entries(app: &mut App) -> Vec<SceneRenderEntry> {
  app
    .world_mut()
    .resource::<PresentationRenderProjection>()
    .entries()
    .to_vec()
}

type RenderKey = (u8, i32, i32, u32);

fn projection_map(entries: &[SceneRenderEntry]) -> BTreeMap<RenderKey, SceneRenderEntry> {
  entries
    .iter()
    .copied()
    .map(|entry| {
      let key = match entry {
        SceneRenderEntry::Terrain { tile, .. } => (0, tile.position().x(), tile.position().y(), 0),
        SceneRenderEntry::Actor { actor, .. } => (1, 0, 0, actor.id().value()),
        SceneRenderEntry::GroundItem { item, .. } => (2, 0, 0, item.id().value()),
        SceneRenderEntry::InventoryItem { item, .. } => (3, 0, 0, item.id().value()),
      };
      (key, entry)
    })
    .collect()
}

fn scene_projection_map(app: &mut App) -> BTreeMap<RenderKey, SceneRenderEntry> {
  let world = app.world_mut();
  let mut expected = BTreeMap::new();
  {
    let mut query = world.query::<(Entity, &SceneTile, Option<&ScenePixelPosition>)>();
    for (entity, tile, pixel_position) in query.iter(world) {
      expected.insert(
        (0, tile.position().x(), tile.position().y(), 0),
        SceneRenderEntry::Terrain {
          entity,
          tile: *tile,
          role: SceneSpriteRole::Terrain,
          pixel_position: pixel_position.copied(),
        },
      );
    }
  }
  {
    let mut query = world.query::<(Entity, &SceneActor, Option<&ScenePixelPosition>)>();
    for (entity, actor, pixel_position) in query.iter(world) {
      let role = if actor.is_alive() {
        match actor.kind() {
          ActorKind::Player => SceneSpriteRole::Player,
          ActorKind::Enemy => SceneSpriteRole::Enemy,
        }
      } else {
        SceneSpriteRole::DeadActor
      };
      expected.insert(
        (1, 0, 0, actor.id().value()),
        SceneRenderEntry::Actor {
          entity,
          actor: *actor,
          role,
          pixel_position: pixel_position.copied(),
        },
      );
    }
  }
  {
    let mut query = world.query::<(Entity, &SceneGroundItem, Option<&ScenePixelPosition>)>();
    for (entity, item, pixel_position) in query.iter(world) {
      expected.insert(
        (2, 0, 0, item.id().value()),
        SceneRenderEntry::GroundItem {
          entity,
          item: *item,
          role: SceneSpriteRole::GroundItem,
          pixel_position: pixel_position.copied(),
        },
      );
    }
  }
  {
    let mut query = world.query::<(Entity, &SceneInventoryItem)>();
    for (entity, item) in query.iter(world) {
      expected.insert(
        (3, 0, 0, item.id().value()),
        SceneRenderEntry::InventoryItem {
          entity,
          item: *item,
          role: SceneSpriteRole::InventoryItem,
        },
      );
    }
  }
  expected
}

fn expected_32_pixel(position: Position) -> (u32, u32) {
  (
    u32::try_from(position.x()).expect("fixture positions should be non-negative") * 32,
    u32::try_from(position.y()).expect("fixture positions should be non-negative") * 32,
  )
}

#[test]
fn projection_contains_complete_typed_scene_and_inventory_exclusion() {
  let mut app = app_with_projection();
  let entries = entries(&mut app);
  assert_eq!(entries.len(), 10);
  assert_eq!(projection_map(&entries), scene_projection_map(&mut app));
  for entry in &entries {
    match entry {
      SceneRenderEntry::Terrain {
        tile,
        pixel_position,
        ..
      } => assert_eq!(
        pixel_position.map(|pixel| (pixel.x(), pixel.y())),
        Some(expected_32_pixel(tile.position()))
      ),
      SceneRenderEntry::Actor {
        actor,
        pixel_position,
        ..
      } => assert_eq!(
        pixel_position.map(|pixel| (pixel.x(), pixel.y())),
        Some(expected_32_pixel(actor.position()))
      ),
      SceneRenderEntry::GroundItem {
        item,
        pixel_position,
        ..
      } => assert_eq!(
        pixel_position.map(|pixel| (pixel.x(), pixel.y())),
        Some(expected_32_pixel(item.position()))
      ),
      SceneRenderEntry::InventoryItem { .. } => {}
    }
  }

  let inventory = entries
    .iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::InventoryItem { item, .. } => Some(*item),
      _ => None,
    })
    .expect("inventory item should remain represented");
  assert_eq!(inventory.id(), ItemId::new(2));
  assert_eq!(inventory.inventory_index(), 0);
}

#[test]
fn accepted_move_refreshes_projection_without_replacing_actor_entity() {
  let mut app = app_with_projection();
  let before = entries(&mut app)
    .into_iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(1) => {
        Some((entity, actor))
      }
      _ => None,
    })
    .expect("player entry should exist");
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("player should move");
  app.update();
  let after = entries(&mut app)
    .into_iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor {
        entity,
        actor,
        pixel_position,
        ..
      } if actor.id() == ActorId::new(1) => Some((entity, actor, pixel_position)),
      _ => None,
    })
    .expect("updated player entry should exist");
  assert_eq!(after.0, before.0);
  assert_eq!(after.1.position(), Position::new(1, 0));
  assert_eq!(after.2.map(|pixel| (pixel.x(), pixel.y())), Some((32, 0)));
}

#[test]
fn missing_runtime_preserves_projection_and_missing_resource_is_safe() {
  let mut app = app_with_projection();
  let before = entries(&mut app);
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  assert_eq!(entries(&mut app), before);

  let mut no_sink = App::new();
  no_sink.insert_resource(PresentationRuntime::new(item_state()));
  no_sink.insert_resource(PresentationTileSize::new(32, 32).expect("tile size should validate"));
  no_sink.add_plugins(PresentationPlugin);
  no_sink.update();
  assert!(
    no_sink
      .world_mut()
      .get_resource::<PresentationRenderProjection>()
      .is_none()
  );
}

#[test]
fn projection_refresh_is_read_only_for_runtime_snapshot_and_digest() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size should validate"));
  app.insert_resource(PresentationRenderProjection::new());
  let before = {
    let runtime = app.world().resource::<PresentationRuntime>();
    (runtime.snapshot(), runtime.replay_digest())
  };
  app.add_plugins(PresentationPlugin);
  app.update();
  let after = {
    let runtime = app.world().resource::<PresentationRuntime>();
    (runtime.snapshot(), runtime.replay_digest())
  };
  assert_eq!(after, before);
}

#[test]
fn missing_tile_size_preserves_existing_pixel_projection() {
  let mut app = app_with_projection();
  let before = entries(&mut app);
  app.world_mut().remove_resource::<PresentationTileSize>();
  app.update();
  assert_eq!(entries(&mut app), before);
}

#[test]
fn missing_tile_size_starts_with_unplaced_map_entries() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(item_state()));
  app.insert_resource(PresentationRenderProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();

  let entries = entries(&mut app);
  assert_eq!(entries.len(), 10);
  for entry in entries {
    match entry {
      SceneRenderEntry::Terrain { pixel_position, .. }
      | SceneRenderEntry::Actor { pixel_position, .. }
      | SceneRenderEntry::GroundItem { pixel_position, .. } => assert_eq!(pixel_position, None),
      SceneRenderEntry::InventoryItem { .. } => {}
    }
  }
}

#[test]
fn stale_scene_keys_are_removed_from_projection() {
  let mut app = app_with_projection();
  assert_eq!(entries(&mut app).len(), 10);
  app
    .world_mut()
    .insert_resource(PresentationRuntime::new(empty_state()));
  app.update();

  let entries = entries(&mut app);
  assert_eq!(entries.len(), 8);
  assert!(entries.iter().all(|entry| !matches!(
    entry,
    SceneRenderEntry::GroundItem { .. } | SceneRenderEntry::InventoryItem { .. }
  )));
  assert_eq!(projection_map(&entries), scene_projection_map(&mut app));
}

#[test]
fn co_located_lower_tile_entity_retains_actor_projection() {
  let mut app = app_with_projection();
  let (tile, actor, actor_entity) = {
    let world = app.world_mut();
    let tile = world
      .query::<&SceneTile>()
      .iter(world)
      .find(|tile| tile.position() == Position::new(0, 0))
      .copied()
      .expect("origin tile should exist");
    let (actor_entity, actor) = world
      .query::<(Entity, &SceneActor)>()
      .iter(world)
      .find(|(_, actor)| actor.id() == ActorId::new(2))
      .map(|(entity, actor)| (entity, *actor))
      .expect("enemy mirror should exist");
    (tile, actor, actor_entity)
  };
  let tile_entity = app.world_mut().spawn(tile).id();
  app.world_mut().entity_mut(tile_entity).insert(actor);
  assert!(tile_entity < actor_entity);
  app.update();

  let projection = projection_map(&entries(&mut app));
  let tile = projection
    .get(&(0, 0, 0, 0))
    .expect("co-located tile should remain represented");
  let projected_actor = projection
    .get(&(1, 0, 0, ActorId::new(2).value()))
    .expect("co-located actor should remain represented");
  assert!(matches!(
    tile,
    SceneRenderEntry::Terrain {
      entity,
      role: SceneSpriteRole::Terrain,
      pixel_position,
      ..
    } if *entity == tile_entity
      && pixel_position.map(|pixel| (pixel.x(), pixel.y())) == Some((0, 0))
  ));
  assert!(matches!(
    projected_actor,
    SceneRenderEntry::Actor {
      entity,
      actor: projected,
      role: SceneSpriteRole::Enemy,
      pixel_position,
    } if *entity == tile_entity
      && *projected == actor
      && pixel_position.map(|pixel| (pixel.x(), pixel.y())) == Some((64, 32))
  ));
}

#[test]
fn duplicate_actor_mirror_retains_lowest_entity_in_projection() {
  let mut app = app_with_projection();
  let (original_entity, actor, role, pixel_position) = {
    let world = app.world_mut();
    let (entity, actor, role, pixel) = world
      .query::<(
        Entity,
        &SceneActor,
        &SceneSpriteRole,
        Option<&ScenePixelPosition>,
      )>()
      .iter(world)
      .find(|(_, actor, _, _)| actor.id() == ActorId::new(1))
      .expect("player mirror should exist");
    (entity, *actor, *role, pixel.copied())
  };
  let duplicate = app
    .world_mut()
    .spawn((
      actor,
      role,
      pixel_position.expect("actor should have a pixel position"),
    ))
    .id();
  let expected = original_entity.min(duplicate);
  app.update();
  let actor_count = {
    let world = app.world_mut();
    world.query::<&SceneActor>().iter(world).count()
  };
  assert_eq!(actor_count, 2);
  let retained = entries(&mut app)
    .into_iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(1) => {
        Some(entity)
      }
      _ => None,
    })
    .expect("player entry should exist");
  assert_eq!(retained, expected);
}

#[test]
fn recycled_lower_index_does_not_replace_the_retained_actor_projection() {
  let mut app = app_with_projection();
  let stable = entries(&mut app)
    .into_iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(1) => {
        Some(entity)
      }
      _ => None,
    })
    .expect("player entry should exist");
  let (tile_to_recycle, actor, role, pixel_position) = {
    let world = app.world_mut();
    let mut query = world.query::<(Entity, &SceneTile)>();
    let tile = query
      .iter(world)
      .find(|(entity, _)| entity.index() < stable.index())
      .map(|(entity, _)| entity)
      .expect("starter scene should have a lower-index tile to recycle");
    let (actor, role, pixel_position) = world
      .query::<(&SceneActor, &SceneSpriteRole, &ScenePixelPosition)>()
      .iter(world)
      .find(|(actor, _, _)| actor.id() == ActorId::new(1))
      .map(|(actor, role, pixel)| (*actor, *role, *pixel))
      .expect("player mirror should have complete render metadata");
    (tile, actor, role, pixel_position)
  };
  app.world_mut().despawn(tile_to_recycle);
  let recycled_entity = Entity::from_index_and_generation(
    tile_to_recycle.index(),
    tile_to_recycle.generation().after_versions(1),
  );
  let duplicate = app
    .world_mut()
    .spawn_at(recycled_entity, (actor, role, pixel_position))
    .expect("despawned index should accept its next generation")
    .id();
  assert!(duplicate.index() < stable.index());
  app.update();

  let retained = entries(&mut app)
    .into_iter()
    .find_map(|entry| match entry {
      SceneRenderEntry::Actor { entity, actor, .. } if actor.id() == ActorId::new(1) => {
        Some(entity)
      }
      _ => None,
    })
    .expect("player entry should exist");
  assert_eq!(retained, stable);
}
