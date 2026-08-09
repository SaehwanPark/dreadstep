//! Deterministic headless audio-cue placeholder behavior.

use bevy::app::App;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use dreadstep_bevy::{
  PresentationAudioCue, PresentationAudioCues, PresentationInput, PresentationPlugin,
  PresentationRuntime, PresentationState,
};
use dreadstep_core::{
  Actor, ActorId, ActorKind, BlockReason, Command, GridMap, HitPoints, Position, Tile, WorldState,
};

fn cue_app(runtime: PresentationRuntime, actor: ActorId) -> App {
  let mut app = App::new();
  app.insert_resource(runtime);
  app.insert_resource(PresentationInput::new(actor));
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);
  app
}

fn custom_runtime(width: u32, tiles: Vec<Tile>, actors: Vec<Actor>) -> PresentationRuntime {
  let map = GridMap::from_tiles(width, 1, tiles).expect("map should validate");
  let world = WorldState::new(map, actors).expect("world should validate");
  PresentationRuntime::new(PresentationState::new(7, world))
}

fn cues(app: &App) -> Vec<PresentationAudioCue> {
  app
    .world()
    .resource::<PresentationAudioCues>()
    .cues()
    .to_vec()
}

#[test]
fn fresh_runtime_has_no_audio_cues() {
  let mut app = cue_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();

  assert!(cues(&app).is_empty());
}

#[test]
fn movement_and_blocking_map_to_typed_cues() {
  let mut moved = cue_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  moved.update();
  moved
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  moved.update();
  assert_eq!(
    cues(&moved),
    vec![PresentationAudioCue::Moved {
      actor: ActorId::new(1),
    }]
  );

  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Wall],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut blocked = cue_app(runtime, ActorId::new(1));
  blocked.update();
  blocked
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  blocked.update();
  assert_eq!(
    cues(&blocked),
    vec![PresentationAudioCue::MovementBlocked {
      actor: ActorId::new(1),
      reason: BlockReason::Terrain,
    }]
  );
}

#[test]
fn wait_maps_to_typed_cue() {
  let runtime = custom_runtime(
    1,
    vec![Tile::Floor],
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  );
  let mut app = cue_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::Space);
  app.update();

  assert_eq!(
    cues(&app),
    vec![PresentationAudioCue::Waited {
      actor: ActorId::new(1),
    }]
  );
}

#[test]
fn attack_and_death_preserve_typed_event_order() {
  let runtime = custom_runtime(
    2,
    vec![Tile::Floor, Tile::Floor],
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  );
  let mut app = cue_app(runtime, ActorId::new(1));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");
  app.update();

  assert_eq!(
    cues(&app),
    vec![
      PresentationAudioCue::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
      },
      PresentationAudioCue::Died {
        actor: ActorId::new(2),
      },
    ]
  );
}

#[test]
fn rejected_command_clears_stale_cues_without_mutating_core() {
  let mut app = cue_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  assert!(!cues(&app).is_empty());
  let before = {
    let runtime = app.world().resource::<PresentationRuntime>();
    (runtime.snapshot(), runtime.replay_digest())
  };
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .release(KeyCode::ArrowRight);
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();

  assert!(cues(&app).is_empty());
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!((runtime.snapshot(), runtime.replay_digest()), before);
}

#[test]
fn missing_runtime_preserves_existing_cues() {
  let mut app = cue_app(
    PresentationRuntime::start_run(7).expect("content should validate"),
    ActorId::new(1),
  );
  app.update();
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);
  app.update();
  let before = cues(&app);
  app.world_mut().remove_resource::<PresentationRuntime>();

  app.update();

  assert_eq!(cues(&app), before);
}

#[test]
fn missing_audio_cue_resource_is_a_safe_noop() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("content should validate"));
  app.insert_resource(PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.add_plugins(PresentationPlugin);

  app.update();

  assert!(
    app
      .world()
      .get_resource::<PresentationAudioCues>()
      .is_none()
  );
}
