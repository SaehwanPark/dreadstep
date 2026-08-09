//! Headless Bevy application-shell behavior.

use bevy::app::App;
use dreadstep_bevy::{PresentationPlugin, PresentationRuntime, SceneActor, SceneTile};
use dreadstep_core::{ActorId, Command, Direction, Position};

fn app_with_runtime() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.add_plugins(PresentationPlugin);
  app
}

#[test]
fn plugin_startup_projects_runtime_into_headless_scene() {
  let mut app = app_with_runtime();

  app.update();

  let world = app.world_mut();
  assert_eq!(world.query::<&SceneTile>().iter(world).count(), 35);
  assert_eq!(world.query::<&SceneActor>().iter(world).count(), 4);
  let runtime = world.resource::<PresentationRuntime>();
  assert_eq!(runtime.seed(), 7);
  assert_eq!(runtime.snapshot().width(), 7);
}

#[test]
fn accepted_runtime_command_is_projected_after_next_update() {
  let mut app = app_with_runtime();
  app.update();
  let player_entity = app
    .world_mut()
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .map(|(entity, _)| entity)
    .expect("player scene entity should exist");

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("player should move");
  app.update();

  let player = app
    .world_mut()
    .query::<(bevy::ecs::entity::Entity, &SceneActor)>()
    .iter(app.world())
    .find(|(_, actor)| actor.id() == ActorId::new(1))
    .expect("updated player scene entity should exist");
  assert_eq!(player.0, player_entity);
  assert_eq!(player.1.position(), Position::new(2, 1));
}

#[test]
fn rejected_runtime_command_is_atomic_for_state_and_scene() {
  let mut app = app_with_runtime();
  app.update();
  let before = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  let before_entities = {
    let world = app.world_mut();
    world.query::<&SceneActor>().iter(world).count()
  };

  let error = app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(99),
    })
    .expect_err("unknown actor should be rejected");
  assert!(error.to_string().contains("actor"));
  app.update();

  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before);
  assert_eq!(runtime.replay_digest(), before_digest);
  let after_entities = {
    let world = app.world_mut();
    world.query::<&SceneActor>().iter(world).count()
  };
  assert_eq!(after_entities, before_entities);
}
