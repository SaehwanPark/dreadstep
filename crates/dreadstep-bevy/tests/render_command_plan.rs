//! Contract tests for the deterministic render-command plan.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationPlugin, PresentationRenderCommandPlan, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  SceneRenderLayer, SceneSpriteKey,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  Tile, WorldState,
};

fn plan_app() -> App {
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
  app.add_plugins(PresentationPlugin);
  app.update();
  app
}

#[test]
fn render_plan_preserves_complete_entries_and_assigns_deterministic_layers() {
  let mut app = plan_app();
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
  let commands = app
    .world_mut()
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .to_vec();
  assert_eq!(commands.len(), 8);
  assert_eq!(commands.len(), sprite_entries.len());
  assert_eq!(commands.len(), render_entries.len());
  assert_eq!(
    commands
      .iter()
      .map(|command| command.layer())
      .collect::<Vec<_>>(),
    vec![
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Terrain,
      SceneRenderLayer::Actor,
      SceneRenderLayer::Actor,
      SceneRenderLayer::GroundItem,
      SceneRenderLayer::InventoryItem,
    ]
  );
  for (order, ((command, sprite), render)) in commands
    .iter()
    .zip(sprite_entries.iter())
    .zip(render_entries.iter())
    .enumerate()
  {
    assert_eq!(command.order(), order);
    assert_eq!(command.sprite_entry(), *sprite);
    assert_eq!(command.sprite_entry().render_entry(), *render);
  }
  assert_eq!(
    commands
      .iter()
      .map(|command| command
        .pixel_position()
        .map(|position| (position.x(), position.y())))
      .collect::<Vec<_>>(),
    vec![
      Some((0, 0)),
      Some((32, 0)),
      Some((64, 0)),
      Some((96, 0)),
      Some((0, 0)),
      Some((32, 0)),
      Some((0, 0)),
      None,
    ]
  );
}

#[test]
fn render_plan_refreshes_dead_role_and_retains_actor_identity() {
  let mut app = plan_app();
  let enemy_entity = app
    .world_mut()
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .iter()
    .find(|command| command.sprite_entry().key() == SceneSpriteKey::Enemy)
    .expect("enemy command should exist")
    .sprite_entry()
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
  let dead = app
    .world_mut()
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .iter()
    .find(|command| command.sprite_entry().key() == SceneSpriteKey::DeadActor)
    .expect("dead command should exist");
  assert_eq!(dead.sprite_entry().entity(), enemy_entity);
  assert_eq!(dead.layer(), SceneRenderLayer::Actor);
}

#[test]
fn missing_source_and_runtime_preserve_existing_render_plan() {
  let mut app = plan_app();
  let before = app
    .world_mut()
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .to_vec();
  app
    .world_mut()
    .remove_resource::<PresentationSpriteProjection>();
  app.update();
  assert_eq!(
    app
      .world()
      .resource::<PresentationRenderCommandPlan>()
      .commands(),
    before.as_slice()
  );
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.update();
  assert_eq!(
    app
      .world()
      .resource::<PresentationRenderCommandPlan>()
      .commands(),
    before.as_slice()
  );
}

#[test]
fn missing_plan_resource_is_a_safe_noop() {
  let mut app = plan_app();
  app
    .world_mut()
    .remove_resource::<PresentationRenderCommandPlan>();
  app.update();
  assert!(
    app
      .world()
      .get_resource::<PresentationRenderCommandPlan>()
      .is_none()
  );
}
