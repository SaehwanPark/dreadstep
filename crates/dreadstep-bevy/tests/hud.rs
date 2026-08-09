//! Deterministic headless HUD status projection behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationHud, PresentationInput, PresentationPlugin, PresentationRuntime, SceneActor,
  SceneGroundItem, SceneInventoryItem, SceneTile,
};
use dreadstep_core::{ActorId, ActorKind, ItemId, Position};

type TileProjection = BTreeMap<(i32, i32), (Entity, SceneTile)>;
type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;
type GroundProjection = BTreeMap<ItemId, (Entity, SceneGroundItem)>;
type InventoryProjection = BTreeMap<ItemId, (Entity, SceneInventoryItem)>;

fn hud_app(actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_item_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationHud::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn complete_scene_projection(
  app: &mut App,
) -> (
  TileProjection,
  ActorProjection,
  GroundProjection,
  InventoryProjection,
) {
  let world = app.world_mut();
  let tiles = {
    let mut query = world.query::<(Entity, &SceneTile)>();
    query
      .iter(world)
      .map(|(entity, tile)| ((tile.position().x(), tile.position().y()), (entity, *tile)))
      .collect()
  };
  let actors = {
    let mut query = world.query::<(Entity, &SceneActor)>();
    query
      .iter(world)
      .map(|(entity, actor)| (actor.id(), (entity, *actor)))
      .collect()
  };
  let ground = {
    let mut query = world.query::<(Entity, &SceneGroundItem)>();
    query
      .iter(world)
      .map(|(entity, item)| (item.id(), (entity, *item)))
      .collect()
  };
  let inventory = {
    let mut query = world.query::<(Entity, &SceneInventoryItem)>();
    query
      .iter(world)
      .map(|(entity, item)| (item.id(), (entity, *item)))
      .collect()
  };
  (tiles, actors, ground, inventory)
}

#[test]
fn startup_projects_controlled_actor_status() {
  let mut app = hud_app(ActorId::new(1));
  app.update();

  let hud = *app.world().resource::<PresentationHud>();
  assert_eq!(hud.actor(), ActorId::new(1));
  assert_eq!(hud.kind(), Some(ActorKind::Player));
  assert_eq!(hud.position(), Some(Position::new(1, 1)));
  assert_eq!(
    hud.hit_points().map(dreadstep_core::HitPoints::value),
    Some(10)
  );
  assert_eq!(
    hud.ready_at().map(dreadstep_core::ActionTime::value),
    Some(0)
  );
}

#[test]
fn accepted_move_updates_hud_in_the_same_app_update() {
  let mut app = hud_app(ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  let hud = *app.world().resource::<PresentationHud>();
  assert_eq!(hud.position(), Some(Position::new(2, 1)));
  assert_eq!(hud.kind(), Some(ActorKind::Player));
  assert_eq!(
    hud.hit_points().map(dreadstep_core::HitPoints::value),
    Some(10)
  );
  assert_eq!(
    hud.ready_at().map(dreadstep_core::ActionTime::value),
    Some(1)
  );
}

#[test]
fn changing_controlled_actor_updates_hud_without_mutating_authoritative_or_scene_state() {
  let mut app = hud_app(ActorId::new(1));
  app.update();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = complete_scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(2)));

  app.update();

  let hud = *app.world().resource::<PresentationHud>();
  assert_eq!(hud.actor(), ActorId::new(2));
  assert_eq!(hud.kind(), Some(ActorKind::Enemy));
  assert_eq!(hud.position(), Some(Position::new(5, 1)));
  assert_eq!(
    hud.hit_points().map(dreadstep_core::HitPoints::value),
    Some(3)
  );
  assert_eq!(
    hud.ready_at().map(dreadstep_core::ActionTime::value),
    Some(0)
  );
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
  assert_eq!(complete_scene_projection(&mut app), before_scene);
}

#[test]
fn unknown_actor_clears_hud_without_mutating_authoritative_or_scene_state() {
  let mut app = hud_app(ActorId::new(1));
  app.update();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = complete_scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(999)));

  app.update();

  let hud = *app.world().resource::<PresentationHud>();
  assert_eq!(hud.actor(), ActorId::new(999));
  assert_eq!(hud.kind(), None);
  assert_eq!(hud.position(), None);
  assert_eq!(hud.hit_points(), None);
  assert_eq!(hud.ready_at(), None);
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
  assert_eq!(complete_scene_projection(&mut app), before_scene);
}

#[test]
fn missing_hud_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_item_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);

  app.update();

  assert!(app.world().get_resource::<PresentationHud>().is_none());
}

#[test]
fn missing_runtime_preserves_existing_hud() {
  let mut app = hud_app(ActorId::new(1));
  app.update();
  let before = *app.world().resource::<PresentationHud>();
  app.world_mut().remove_resource::<PresentationRuntime>();

  app.update();

  assert_eq!(*app.world().resource::<PresentationHud>(), before);
}

#[test]
fn missing_input_preserves_existing_hud() {
  let mut app = hud_app(ActorId::new(1));
  app.update();
  let before = *app.world().resource::<PresentationHud>();
  app.world_mut().remove_resource::<PresentationInput>();

  app.update();

  assert_eq!(*app.world().resource::<PresentationHud>(), before);
}
