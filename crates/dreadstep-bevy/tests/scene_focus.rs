//! Headless scene-focus marker behavior.

use std::collections::BTreeMap;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationFocus, PresentationInput, PresentationPlugin, PresentationRuntime, SceneActor,
  SceneFocus, SceneTile,
};
use dreadstep_core::ActorId;

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

fn focused_actors(app: &mut App) -> BTreeMap<ActorId, Entity> {
  let world = app.world_mut();
  let mut query = world.query::<(Entity, &SceneActor, Option<&SceneFocus>)>();
  query
    .iter(world)
    .filter_map(|(entity, actor, marker)| marker.map(|_| (actor.id(), entity)))
    .collect()
}

#[test]
fn startup_marks_the_selected_actor_entity() {
  let mut app = focus_app(ActorId::new(1));

  app.update();

  let focused = focused_actors(&mut app);
  let actor_entity = scene_projection(&mut app)
    .1
    .get(&ActorId::new(1))
    .map(|(entity, _)| *entity)
    .expect("selected actor should be projected");
  assert_eq!(focused, BTreeMap::from([(ActorId::new(1), actor_entity)]));
}

#[test]
fn accepted_move_keeps_marker_on_the_stable_actor_entity() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let actor_entity = focused_actors(&mut app)
    .remove(&ActorId::new(1))
    .expect("selected actor should be focused");
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    focused_actors(&mut app),
    BTreeMap::from([(ActorId::new(1), actor_entity)])
  );
  assert_eq!(
    scene_projection(&mut app)
      .1
      .get(&ActorId::new(1))
      .expect("selected actor should remain projected")
      .1
      .position(),
    dreadstep_core::Position::new(2, 1)
  );
}

#[test]
fn changing_controlled_actor_moves_one_marker_without_changing_entity_identity() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before_scene = scene_projection(&mut app);
  let actor_two_entity = before_scene
    .1
    .get(&ActorId::new(2))
    .map(|(entity, _)| *entity)
    .expect("second actor should be projected");
  app
    .world_mut()
    .entity_mut(actor_two_entity)
    .insert(SceneFocus);
  app.insert_resource(PresentationInput::new(ActorId::new(2)));

  app.update();

  assert_eq!(
    focused_actors(&mut app),
    BTreeMap::from([(ActorId::new(2), actor_two_entity)])
  );
  assert_eq!(focused_actors(&mut app).len(), 1);
}

#[test]
fn unknown_actor_clears_marker_without_mutating_runtime_or_scene() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_scene = scene_projection(&mut app);
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert!(focused_actors(&mut app).is_empty());
  assert_eq!(
    app.world().resource::<PresentationRuntime>().snapshot(),
    before_snapshot
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

#[test]
fn missing_input_leaves_existing_marker_unchanged() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let actor_two_entity = scene_projection(&mut app)
    .1
    .get(&ActorId::new(2))
    .map(|(entity, _)| *entity)
    .expect("second actor should be projected");
  app
    .world_mut()
    .entity_mut(actor_two_entity)
    .insert(SceneFocus);
  let before_with_duplicate = focused_actors(&mut app);
  app.world_mut().remove_resource::<PresentationInput>();

  app.update();

  assert_eq!(focused_actors(&mut app), before_with_duplicate);
}

#[test]
fn missing_focus_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);

  app.update();

  assert!(focused_actors(&mut app).is_empty());
  let world = app.world_mut();
  assert_eq!(world.query::<&SceneTile>().iter(world).count(), 35);
  assert_eq!(world.query::<&SceneActor>().iter(world).count(), 4);
}

#[test]
fn missing_runtime_leaves_existing_marker_unchanged() {
  let mut app = focus_app(ActorId::new(1));
  app.update();
  let before = focused_actors(&mut app);
  app.world_mut().remove_resource::<PresentationRuntime>();
  app.insert_resource(PresentationInput::new(ActorId::new(99)));

  app.update();

  assert_eq!(focused_actors(&mut app), before);
}
