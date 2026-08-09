//! Contract tests for typed sprite selectors over the headless render projection.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, SceneRenderEntry,
  SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn sprite_app() -> App {
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
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

#[test]
fn sprite_projection_preserves_ordered_complete_entries_and_typed_keys() {
  let mut app = sprite_app();
  let render_entries = app
    .world_mut()
    .resource::<PresentationRenderProjection>()
    .entries()
    .to_vec();
  let sprite_entries = app
    .world_mut()
    .resource::<PresentationSpriteProjection>()
    .entries()
    .to_vec();
  assert_eq!(sprite_entries.len(), render_entries.len());
  assert_eq!(
    sprite_entries
      .iter()
      .map(|entry| entry.key())
      .collect::<Vec<_>>(),
    vec![
      SceneSpriteKey::Terrain(Tile::Floor),
      SceneSpriteKey::Terrain(Tile::Floor),
      SceneSpriteKey::Terrain(Tile::Floor),
      SceneSpriteKey::Terrain(Tile::Wall),
      SceneSpriteKey::Player,
      SceneSpriteKey::Enemy,
      SceneSpriteKey::GroundItem(ItemDefinitionId::new(101)),
      SceneSpriteKey::InventoryItem(ItemDefinitionId::new(102)),
    ]
  );
  for (sprite, render) in sprite_entries.iter().zip(render_entries.iter()) {
    assert_eq!(sprite.render_entry(), *render);
    assert_eq!(
      sprite.entity(),
      match render {
        SceneRenderEntry::Terrain { entity, .. }
        | SceneRenderEntry::Actor { entity, .. }
        | SceneRenderEntry::GroundItem { entity, .. }
        | SceneRenderEntry::InventoryItem { entity, .. } => *entity,
      }
    );
  }
  assert!(matches!(
    sprite_entries
      .last()
      .expect("inventory entry should exist")
      .render_entry(),
    SceneRenderEntry::InventoryItem { .. }
  ));
}

#[test]
fn missing_sprite_projection_resource_is_a_safe_noop() {
  let mut app = sprite_app();
  app
    .world_mut()
    .remove_resource::<PresentationSpriteProjection>();
  app.update();
  assert!(
    app
      .world()
      .get_resource::<PresentationSpriteProjection>()
      .is_none()
  );
}

#[test]
fn sprite_projection_refreshes_roles_and_preserves_identity_without_runtime_mutation() {
  let mut app = sprite_app();
  let enemy_entity = app
    .world_mut()
    .resource::<PresentationSpriteProjection>()
    .entries()
    .iter()
    .find(|entry| entry.key() == SceneSpriteKey::Enemy)
    .expect("enemy sprite entry should exist")
    .entity();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();
  let dead_entry = app
    .world_mut()
    .resource::<PresentationSpriteProjection>()
    .entries()
    .iter()
    .find(|entry| entry.key() == SceneSpriteKey::DeadActor)
    .copied()
    .expect("dead actor sprite entry should exist");
  assert_eq!(dead_entry.entity(), enemy_entity);

  let before = app
    .world_mut()
    .resource::<PresentationSpriteProjection>()
    .entries()
    .to_vec();
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  assert_eq!(
    app
      .world()
      .resource::<PresentationSpriteProjection>()
      .entries(),
    before.as_slice()
  );
}
