//! Typed headless presentation-focus behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationFocus, PresentationInput, PresentationPlugin, PresentationRuntime, SceneActor,
  SceneTile,
};
use dreadstep_core::{ActorId, Position};

type TileProjection = BTreeMap<(i32, i32), (Entity, SceneTile)>;
type ActorProjection = BTreeMap<ActorId, (Entity, SceneActor)>;

fn focus_app(actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationFocus::new(actor));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn scene_projection(app: &mut App) -> (TileProjection, ActorProjection) {
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
  (tiles, actors)
}

#[test]
fn startup_focus_mirrors_selected_actor_position() {
  let mut app = focus_app(ActorId::new(1));

  app.update();

  let focus = app.world().resource::<PresentationFocus>();
  assert_eq!(focus.actor(), ActorId::new(1));
  assert_eq!(focus.position(), Some(Position::new(1, 1)));
}

#[test]
fn accepted_keyboard_move_updates_focus_in_same_app_update() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    app.world().resource::<PresentationFocus>().position(),
    Some(Position::new(2, 1))
  );
}

#[test]
fn changing_controlled_actor_updates_focus_identity_and_position() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  app.insert_resource(PresentationInput::new(ActorId::new(2)));

  app.update();

  let focus = app.world().resource::<PresentationFocus>();
  assert_eq!(focus.actor(), ActorId::new(2));
  assert_eq!(focus.position(), Some(Position::new(5, 1)));
}

#[test]
fn missing_input_resource_leaves_existing_focus_unchanged() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before = *app.world().resource::<PresentationFocus>();
  app.world_mut().remove_resource::<PresentationInput>();

  app.update();

  assert_eq!(*app.world().resource::<PresentationFocus>(), before);
}

#[test]
fn missing_focus_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);

  app.update();

  let world = app.world_mut();
  assert_eq!(world.query::<&SceneTile>().iter(world).count(), 35);
  assert_eq!(world.query::<&SceneActor>().iter(world).count(), 4);
}

#[test]
fn missing_runtime_leaves_existing_focus_and_scene_unchanged() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before_focus = *app.world().resource::<PresentationFocus>();
  let before_scene = scene_projection(&mut app);
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert_eq!(*app.world().resource::<PresentationFocus>(), before_focus);
  assert_eq!(scene_projection(&mut app), before_scene);
}

#[test]
fn unknown_focus_actor_has_no_position_and_does_not_change_scene() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  let focus = app.world().resource::<PresentationFocus>();
  assert_eq!(focus.actor(), ActorId::new(99));
  assert_eq!(focus.position(), None);
  assert_eq!(
    app.world().resource::<PresentationRuntime>().snapshot(),
    before
  );
  assert_eq!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .replay_digest(),
    before_digest
  );
  assert_eq!(scene_projection(&mut app), before_scene);
}
