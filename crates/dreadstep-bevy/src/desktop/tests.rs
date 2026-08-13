//! Unit tests for the desktop process boundary.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::cli::ParseResult;
use super::format::{
  command_value, controls_text, enemy_intent_summary, event_message, event_value, format_hud_stats,
  health_bar_text, scenario_label, terminal_hud_message, visibility_summary_values,
};
use super::input::{
  advance_procedural_floor, command_for_key, desktop_input, restart_requested, submit_command,
};
use super::journal::{Journal, export_replay, journal_path};
use super::plugin::{
  ACTOR_PULSE_DURATION, DesktopAnimationState, DesktopAudioPlayback, DesktopAudioState,
  audio_asset_path, build_audio_manifest, desktop_finalize, desktop_observe_close,
  desktop_play_audio, desktop_style_sprites, desktop_update_animation, pulse_for_remaining,
  sprite_scale,
};
use super::session::DesktopStatus;
use super::*;
use crate::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAssetReference,
  PresentationAudioAssetManifest, PresentationAudioAssetProjection, PresentationAudioCue,
  PresentationAudioCueKind, PresentationAudioCues, PresentationBevySpriteProjection,
  PresentationBevySpriteTransformProjection, PresentationEnemyIntent, PresentationInput,
  PresentationKeyboardMode, PresentationPlugin, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSet, PresentationSpriteProjection, PresentationState, PresentationTileSize,
  PresentationVisibility, SceneRenderNode, SceneRenderPlaceholder,
};
use bevy::app::TaskPoolPlugin;
use bevy::app::{App, AppExit, Last, Update};
use bevy::asset::{AssetApp, AssetPlugin};
use bevy::audio::{AudioSource, PlaybackMode, PlaybackSettings};
use bevy::camera::visibility::Visibility;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::time::Time;
use bevy::transform::components::Transform;
use bevy::window::ClosingWindow;
use dreadstep_core::{
  Actor, ActorKind, BlockReason, Command, Direction, Event, GridMap, Item, Position, RunOutcome,
  Tile, WorldState,
};
use serde_json::{Value, json};

fn test_directory(label: &str) -> PathBuf {
  std::env::temp_dir().join(format!(
    "dreadstep-desktop-{label}-{}-{}",
    std::process::id(),
    match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(duration) => duration.as_nanos(),
      Err(_) => 0,
    }
  ))
}

#[test]
fn cli_parser_rejects_duplicates_and_accepts_smoke_options() {
  let parsed = parse_options([
    OsString::from("--seed"),
    OsString::from("12"),
    OsString::from("--log-dir"),
    OsString::from("logs"),
    OsString::from("--smoke"),
  ])
  .expect("options parse");
  assert_eq!(
    parsed,
    ParseResult::Options(DesktopOptions {
      seed: 12,
      procedural: false,
      depth: 1,
      log_dir: PathBuf::from("logs"),
      smoke: true,
    })
  );
  assert!(parse_options([OsString::from("--smoke"), OsString::from("--smoke")]).is_err());
  assert!(parse_options([OsString::from("--seed")]).is_err());
  assert!(parse_options([OsString::from("--help"), OsString::from("--unknown")]).is_err());
  assert!(parse_options([OsString::from("--help"), OsString::from("--help")]).is_err());
  assert!(parse_options([OsString::from("--log-dir"), OsString::from("--smoke")]).is_err());
  let procedural = parse_options([
    OsString::from("--procedural"),
    OsString::from("--depth"),
    OsString::from("4"),
  ])
  .expect("procedural options parse");
  assert_eq!(
    procedural,
    ParseResult::Options(DesktopOptions {
      procedural: true,
      depth: 4,
      ..DesktopOptions::default()
    })
  );
  assert!(
    parse_options([
      OsString::from("--procedural"),
      OsString::from("--procedural")
    ])
    .is_err()
  );
  assert!(parse_options([OsString::from("--depth")]).is_err());
}

#[test]
fn startup_selection_keeps_procedural_visible_and_item_smoke_fixtures_distinct() {
  let procedural = start_runtime(&DesktopOptions {
    procedural: true,
    depth: 3,
    ..DesktopOptions::default()
  })
  .expect("procedural startup should validate");
  assert_eq!(procedural.snapshot().width(), 13);
  assert_eq!(procedural.snapshot().height(), 9);
  assert_eq!(procedural.snapshot().actors()[1].hit_points().value(), 6);

  let smoke = start_runtime(&DesktopOptions {
    procedural: true,
    smoke: true,
    depth: 3,
    ..DesktopOptions::default()
  })
  .expect("smoke startup should validate");
  assert_eq!(smoke.snapshot().width(), 7);
  assert_eq!(smoke.snapshot().height(), 5);
  assert_eq!(smoke.snapshot().ground_items().len(), 0);
}

#[test]
fn replay_export_is_versioned_ordered_and_create_new() {
  let directory = test_directory("replay-export");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut runtime = PresentationRuntime::start_run(7).expect("starter run");
  let command = Command::Move {
    actor: PLAYER,
    direction: Direction::East,
  };
  runtime.execute(command).expect("starter move accepted");

  let first = export_replay(&runtime, &journal).expect("first export writes");
  let second = export_replay(&runtime, &journal).expect("second export gets a new path");
  assert_ne!(first, second);
  let export =
    serde_json::from_str::<Value>(&fs::read_to_string(&first).expect("first export reads"))
      .expect("first export parses");
  assert_eq!(export["schema_version"], REPLAY_EXPORT_SCHEMA_VERSION);
  assert_eq!(export["seed"], 7);
  assert_eq!(export["commands"].as_array().map(Vec::len), Some(1));
  assert_eq!(export["commands"][0], command_value(command));
  assert_eq!(export["replay_digest"], runtime.replay_digest().value());
  assert_eq!(export["outcome"], "in_progress");

  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert_eq!(
    journal_text.matches("\"kind\":\"replay_exported\"").count(),
    2
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn pickup_key_selects_the_lowest_id_ground_item() {
  let directory = test_directory("pickup-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut runtime = PresentationRuntime::start_item_run(7).expect("starter item run");
  runtime
    .prepare_smoke_pickup(PLAYER, EQUIP_ITEM)
    .expect("first pickup fixture item drops");
  runtime
    .prepare_smoke_pickup(PLAYER, PICKUP_ITEM)
    .expect("second pickup fixture item drops");
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyP, &runtime, &session),
    Some(Command::Pickup {
      actor: PLAYER,
      item: EQUIP_ITEM,
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn ranged_key_selects_the_lowest_id_legal_target() {
  let directory = test_directory("ranged-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::filled(5, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(5), ActorKind::Enemy, Position::new(2, 0)),
      Actor::new(RANGED_TARGET, ActorKind::Enemy, Position::new(3, 0)),
    ],
  )
  .expect("test world should be valid");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyG, &runtime, &session),
    Some(Command::RangedAttack {
      actor: PLAYER,
      target: RANGED_TARGET,
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn reload_key_selects_the_legal_player_reload() {
  let directory = test_directory("reload-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![Actor::with_ranged_ammo(
      PLAYER,
      ActorKind::Player,
      Position::new(0, 0),
      dreadstep_core::HitPoints::new(10),
      1,
    )],
  )
  .expect("test world should be valid");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyR, &runtime, &session),
    Some(Command::Reload { actor: PLAYER })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn interact_key_selects_the_adjacent_legal_door() {
  let directory = test_directory("interact-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Door, Tile::Floor])
      .expect("test map should be valid"),
    vec![Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0))],
  )
  .expect("test world should be valid");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyI, &runtime, &session),
    Some(Command::Interact {
      actor: PLAYER,
      position: Position::new(1, 0),
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn break_key_selects_the_adjacent_legal_breakable_terrain() {
  let directory = test_directory("break-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Breakable, Tile::Floor])
      .expect("test map should be valid"),
    vec![Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0))],
  )
  .expect("test world should be valid");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyB, &runtime, &session),
    Some(Command::Break {
      actor: PLAYER,
      position: Position::new(1, 0),
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn kick_key_selects_the_adjacent_legal_closed_door() {
  let directory = test_directory("kick-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Door, Tile::Floor])
      .expect("test map should be valid"),
    vec![Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0))],
  )
  .expect("test world should be valid");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let session = DesktopSession::new(7, journal);
  assert_eq!(
    command_for_key(KeyCode::KeyK, &runtime, &session),
    Some(Command::Kick {
      actor: PLAYER,
      position: Position::new(1, 0),
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn drop_key_selects_the_selected_legal_player_item() {
  let directory = test_directory("drop-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0))],
  )
  .expect("test world should be valid");
  world
    .give_item(
      PLAYER,
      Item::new(ItemId::new(101), dreadstep_core::ItemDefinitionId::new(1)),
    )
    .expect("first item should be owned");
  world
    .give_item(
      PLAYER,
      Item::new(ItemId::new(102), dreadstep_core::ItemDefinitionId::new(2)),
    )
    .expect("second item should be owned");
  let runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let mut session = DesktopSession::new(7, journal);
  session.selected_item = Some(ItemId::new(102));
  assert_eq!(
    command_for_key(KeyCode::KeyX, &runtime, &session),
    Some(Command::Drop {
      actor: PLAYER,
      item: ItemId::new(102),
    })
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn reload_only_restarts_when_shift_is_held() {
  let mut keys = ButtonInput::<KeyCode>::default();
  keys.press(KeyCode::KeyR);
  assert!(!restart_requested(&keys));
  keys.press(KeyCode::ShiftLeft);
  assert!(restart_requested(&keys));
}

#[test]
fn procedural_victory_can_advance_to_the_next_seeded_floor() {
  let directory = test_directory("procedural-floor-advance");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut runtime =
    PresentationRuntime::start_procedural_run(7, 2).expect("procedural floor should validate");
  let before_digest = runtime.snapshot().digest();
  let mut session = DesktopSession::new_with_scenario(7, true, 2, journal.clone());
  session.status = DesktopStatus::Victory;

  assert!(advance_procedural_floor(&mut runtime, &mut session));
  let expected =
    PresentationRuntime::start_procedural_run(7, 3).expect("next procedural floor should validate");
  assert_eq!(session.status, DesktopStatus::Running);
  assert_eq!(session.seed, 7);
  assert_eq!(session.depth, 3);
  assert_ne!(runtime.snapshot().digest(), before_digest);
  assert_eq!(runtime.snapshot().digest(), expected.snapshot().digest());
  assert_eq!(runtime.replay_digest(), expected.replay_digest());
  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert!(journal_text.contains("\"kind\":\"floor_advanced\""));
  assert!(journal_text.contains("\"depth\":3"));
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn procedural_floor_advance_is_guarded_to_victory_and_procedural_sessions() {
  let directory = test_directory("procedural-floor-advance-guards");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut runtime =
    PresentationRuntime::start_procedural_run(7, 2).expect("procedural floor should validate");
  let original_digest = runtime.snapshot().digest();
  let mut running = DesktopSession::new_with_scenario(7, true, 2, journal.clone());
  assert!(!advance_procedural_floor(&mut runtime, &mut running));
  assert_eq!(runtime.snapshot().digest(), original_digest);
  assert_eq!(running.depth, 2);

  let mut item_runtime = PresentationRuntime::start_item_run(7).expect("item run validates");
  let item_digest = item_runtime.snapshot().digest();
  let mut item_session = DesktopSession::new(7, journal.clone());
  item_session.status = DesktopStatus::Victory;
  assert!(!advance_procedural_floor(
    &mut item_runtime,
    &mut item_session
  ));
  assert_eq!(item_runtime.snapshot().digest(), item_digest);
  assert_eq!(item_session.depth, 1);

  let mut overflow_runtime =
    PresentationRuntime::start_procedural_run(7, 1).expect("procedural floor should validate");
  let mut overflow_session = DesktopSession::new_with_scenario(7, true, u32::MAX, journal.clone());
  overflow_session.status = DesktopStatus::Victory;
  assert!(!advance_procedural_floor(
    &mut overflow_runtime,
    &mut overflow_session
  ));
  assert!(matches!(overflow_session.status, DesktopStatus::Faulted(_)));
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn next_floor_key_dispatches_only_from_procedural_victory() {
  let directory = test_directory("procedural-floor-key");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut app = App::new();
  app.add_message::<AppExit>();
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.insert_resource(
    PresentationRuntime::start_procedural_run(7, 2).expect("procedural floor should validate"),
  );
  let mut session = DesktopSession::new_with_scenario(7, true, 2, journal.clone());
  session.status = DesktopStatus::Victory;
  app.insert_resource(session);
  app.add_systems(Update, desktop_input);
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::KeyN);
  app.update();

  let session = app.world().resource::<DesktopSession>();
  assert_eq!(session.status, DesktopStatus::Running);
  assert_eq!(session.depth, 3);
  assert_eq!(session.seed, 7);
  let runtime = app.world().resource::<PresentationRuntime>();
  let expected =
    PresentationRuntime::start_procedural_run(7, 3).expect("next procedural floor should validate");
  assert_eq!(runtime.snapshot().digest(), expected.snapshot().digest());
  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert!(journal_text.contains("\"action\":\"next_floor\""));
  assert!(journal_text.contains("\"kind\":\"floor_advanced\""));
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn manifest_contains_all_independent_placeholder_families() {
  let manifest = build_manifest().expect("showcase manifest validates");
  assert_eq!(manifest.bindings().len(), 6);
  assert!(
    manifest
      .bindings()
      .iter()
      .all(|(_, reference)| reference.path().starts_with("assets/dreadstep/"))
  );
}

#[test]
fn journal_flushes_schema_and_allocates_non_overwriting_paths() {
  let directory = test_directory("journal");
  let mut first = Journal::open(&directory).expect("first journal creates");
  let second = Journal::open(&directory).expect("second journal creates a suffix");
  assert_ne!(first.path(), second.path());
  first
    .record("test", json!({ "value": 1 }))
    .expect("journal record flushes");
  let line = fs::read_to_string(first.path())
    .expect("journal reads")
    .lines()
    .next()
    .map(str::to_string)
    .expect("journal has a line");
  let value: Value = serde_json::from_str(&line).expect("journal line parses");
  assert_eq!(value["schema_version"], 1);
  assert_eq!(value["sequence"], 1);
}

#[test]
fn caught_panic_payload_becomes_diagnostic_text() {
  let payload =
    panic::catch_unwind(|| panic!("desktop test panic")).expect_err("test panic is caught");
  assert_eq!(panic_message(payload), "desktop test panic");
}

#[test]
fn health_bar_is_fixed_width_and_clamped() {
  assert_eq!(health_bar_text(-3), "[----------]");
  assert_eq!(health_bar_text(5), "[#####-----]");
  assert_eq!(health_bar_text(99), "[##########]");
}

#[test]
fn healing_consumption_is_visible_in_desktop_event_evidence() {
  let event = Event::ItemConsumed {
    actor: PLAYER,
    item: ItemId::new(101),
    healing: Some(dreadstep_core::HealingResult::new(
      2,
      dreadstep_core::HitPoints::new(10),
    )),
    ammunition: None,
  };
  assert_eq!(
    event_value(event),
    json!({
      "kind": "item_consumed",
      "actor": 1,
      "item": 101,
      "healing": {"amount": 2, "remaining_hit_points": 10},
      "ammunition": null,
    })
  );
  assert_eq!(
    event_message(event),
    "Actor 1 consumed item 101 and restored 2 HP (10 HP)."
  );
}

#[test]
fn ammunition_consumption_is_visible_in_desktop_event_evidence() {
  let event = Event::ItemConsumed {
    actor: PLAYER,
    item: ItemId::new(102),
    healing: None,
    ammunition: Some(dreadstep_core::AmmunitionResult::new(2, 3)),
  };
  assert_eq!(
    event_value(event),
    json!({
      "kind": "item_consumed",
      "actor": 1,
      "item": 102,
      "healing": null,
      "ammunition": {"amount": 2, "remaining_ammunition": 3},
    })
  );
  assert_eq!(
    event_message(event),
    "Actor 1 consumed item 102 and restored 2 ammunition (3 shots)."
  );
}

#[test]
fn visibility_summary_distinguishes_active_and_full_map() {
  assert_eq!(visibility_summary_values(false, 0, 0), "FOV full map");
  assert_eq!(
    visibility_summary_values(true, 3, 2),
    "FOV 2 tiles (radius 3)"
  );
}

#[test]
fn hud_stats_report_enemy_pressure_and_missing_player() {
  let state = PresentationState::start_item_run(7).expect("starter item run");
  let snapshot = state.snapshot();
  let status = DesktopStatus::Running;
  let empty_intent = PresentationEnemyIntent::new();
  let text = format_hud_stats(
    None,
    &snapshot,
    &status,
    &scenario_label(false, 1),
    "",
    None,
    Some(&empty_intent),
  );
  assert!(text.contains("Player unavailable"));
  assert!(text.contains("Enemies remaining: 3"));
  assert!(text.contains("FOV full map"));
  assert!(text.contains("Intent: none"));
  assert!(!text.contains("press N"));

  let player = snapshot
    .actors()
    .iter()
    .find(|actor| actor.id() == PLAYER)
    .expect("player exists");
  let text = format_hud_stats(
    Some(player),
    &snapshot,
    &status,
    &scenario_label(false, 1),
    "",
    None,
    Some(&empty_intent),
  );
  assert!(text.contains("HP [##########] 10/10"));
  assert!(text.contains("enemies 3"));
  assert!(text.contains("Intent: none"));
}

#[test]
fn hud_scenario_label_distinguishes_procedural_depth_and_item_fixture() {
  assert_eq!(scenario_label(true, 4), "Procedural floor · depth 4");
  assert_eq!(scenario_label(false, 1), "Starter item floor");
}

#[test]
fn terminal_hud_message_matches_outcome_and_avoids_depth_overflow() {
  assert_eq!(
    terminal_hud_message(&DesktopStatus::Victory, true, 4),
    "Floor cleared — press N for depth 5, or Shift+R to restart"
  );
  assert_eq!(
    terminal_hud_message(&DesktopStatus::Victory, false, 4),
    "Showcase complete — press Shift+R to restart"
  );
  assert_eq!(
    terminal_hud_message(&DesktopStatus::Defeat, true, 4),
    "Showcase failed — press Shift+R to restart"
  );
  assert!(terminal_hud_message(&DesktopStatus::Running, true, 4).is_empty());
  assert_eq!(
    terminal_hud_message(&DesktopStatus::Victory, true, u32::MAX),
    "Floor cleared — next depth unavailable; press Shift+R to restart"
  );
}

#[test]
fn controls_only_advertise_next_floor_for_procedural_runs() {
  assert!(controls_text(true).contains("N next procedural floor"));
  assert!(!controls_text(false).contains("next procedural floor"));
}

#[test]
fn finalization_exports_replay_and_shutdown_before_app_exit() {
  let directory = test_directory("finalization");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let handle = FinalizationHandle::new();
  let mut app = App::new();
  app.add_message::<AppExit>();
  app.insert_resource(PresentationRuntime::start_run(7).expect("starter run validates"));
  app.insert_resource(DesktopSession::new(7, journal.clone()));
  app.insert_resource(handle.clone());
  app.add_systems(Update, |mut exits: MessageWriter<AppExit>| {
    exits.write(AppExit::Success);
  });
  app.add_systems(Last, desktop_finalize);
  app.update();

  let report = handle.0.lock().expect("finalization report lock");
  assert!(report.complete);
  assert!(report.error.is_none());
  drop(report);
  let journal_text = fs::read_dir(&directory)
    .expect("journal directory reads")
    .map(|entry| fs::read_to_string(entry.expect("journal entry reads").path()))
    .collect::<Result<Vec<_>, _>>()
    .expect("journal files read");
  let joined = journal_text.join("\n");
  assert!(joined.contains("\"kind\":\"replay_exported\""));
  assert!(joined.contains("\"kind\":\"shutdown\""));
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn accepted_player_death_sets_defeat_and_records_terminal_once() {
  let directory = test_directory("player-defeat");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_hit_points(
        PLAYER,
        ActorKind::Player,
        Position::new(1, 0),
        dreadstep_core::HitPoints::new(1),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(0, 0),
        dreadstep_core::HitPoints::new(4),
      ),
    ],
  )
  .expect("test world validates");
  let mut runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let mut session = DesktopSession::new(7, journal.clone());

  assert!(submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Wait { actor: PLAYER },
  ));
  assert!(submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Attack {
      actor: ActorId::new(2),
      target: PLAYER,
    },
  ));
  assert_eq!(session.status, DesktopStatus::Defeat);
  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert_eq!(
    journal_text.matches("\"kind\":\"terminal_defeat\"").count(),
    1
  );
  assert!(!submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Wait { actor: PLAYER },
  ));
  let journal_text_after = fs::read_to_string(journal_path(&journal)).expect("journal rereads");
  assert_eq!(
    journal_text_after
      .matches("\"kind\":\"terminal_defeat\"")
      .count(),
    1
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn no_enemy_world_stays_in_progress_instead_of_faking_victory() {
  let directory = test_directory("outcome-no-enemy");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::filled(1, 1, Tile::Floor).expect("test map should be valid"),
    vec![Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0))],
  )
  .expect("test world validates");
  let mut runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let mut session = DesktopSession::new(7, journal.clone());

  assert!(submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Wait { actor: PLAYER },
  ));
  assert_eq!(runtime.snapshot().outcome(), RunOutcome::InProgress);
  assert_eq!(session.status, DesktopStatus::Running);
  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert!(!journal_text.contains("terminal_victory"));
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn accepted_last_enemy_death_sets_victory_and_records_terminal_once() {
  let directory = test_directory("outcome-victory");
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::new(PLAYER, ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        dreadstep_core::HitPoints::new(1),
      ),
    ],
  )
  .expect("test world validates");
  let mut runtime = PresentationRuntime::new(PresentationState::new(7, world));
  let mut session = DesktopSession::new(7, journal.clone());

  assert!(submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Attack {
      actor: PLAYER,
      target: ActorId::new(2),
    },
  ));
  assert_eq!(runtime.snapshot().outcome(), RunOutcome::Victory);
  assert_eq!(session.status, DesktopStatus::Victory);
  let journal_text = fs::read_to_string(journal_path(&journal)).expect("journal reads");
  assert_eq!(journal_text.matches("terminal_victory").count(), 1);
  assert!(!submit_command(
    &mut runtime,
    &mut session,
    "test",
    Command::Wait { actor: PLAYER },
  ));
  let journal_text_after = fs::read_to_string(journal_path(&journal)).expect("journal rereads");
  assert_eq!(journal_text_after.matches("terminal_victory").count(), 1);
  let _ = fs::remove_dir_all(directory);
}

fn defeated_input_app(directory_name: &str) -> (App, PathBuf) {
  let directory = test_directory(directory_name);
  let _ = fs::create_dir_all(&directory);
  let journal = Arc::new(Mutex::new(
    Journal::open(&directory).expect("journal opens"),
  ));
  let mut app = App::new();
  app.add_message::<AppExit>();
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.insert_resource(PresentationRuntime::start_item_run(7).expect("starter item run"));
  let mut session = DesktopSession::new(7, journal);
  session.status = DesktopStatus::Defeat;
  app.insert_resource(session);
  app.add_systems(Update, desktop_input);
  (app, directory)
}

#[test]
fn defeated_shift_restart_returns_to_running_same_seed() {
  let (mut app, directory) = defeated_input_app("defeat-restart");
  {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(KeyCode::KeyR);
    keys.press(KeyCode::ShiftLeft);
  }
  app.update();
  let session = app.world().resource::<DesktopSession>();
  assert_eq!(session.status, DesktopStatus::Running);
  assert_eq!(session.seed, 7);
  assert!(session.messages.is_empty());
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn defeated_escape_and_window_close_preserve_terminal_shutdown() {
  let (mut app, directory) = defeated_input_app("defeat-escape");
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::Escape);
  app.update();
  assert_eq!(
    app.world().resource::<DesktopSession>().status,
    DesktopStatus::Shutdown("escape".to_string())
  );
  let escape_journal = fs::read_dir(&directory)
    .expect("escape journal directory reads")
    .next()
    .expect("escape journal exists")
    .expect("escape journal entry reads")
    .path();
  assert!(
    !fs::read_to_string(escape_journal)
      .expect("escape journal reads")
      .contains("\"kind\":\"shutdown\"")
  );
  let _ = fs::remove_dir_all(directory);

  let (mut app, directory) = defeated_input_app("defeat-close");
  app.world_mut().spawn(ClosingWindow);
  app.add_systems(Update, desktop_observe_close);
  app.update();
  assert_eq!(
    app.world().resource::<DesktopSession>().status,
    DesktopStatus::Shutdown("window_close".to_string())
  );
  let close_journal = fs::read_dir(&directory)
    .expect("close journal directory reads")
    .next()
    .expect("close journal exists")
    .expect("close journal entry reads")
    .path();
  assert!(
    !fs::read_to_string(close_journal)
      .expect("close journal reads")
      .contains("\"kind\":\"shutdown\"")
  );
  let _ = fs::remove_dir_all(directory);
}

#[test]
fn hud_intent_summary_reports_the_exact_chase_target() {
  let mut intent = PresentationEnemyIntent::new();
  intent.actor = Some(ActorId::new(2));
  intent.command = Some(Command::Chase {
    actor: ActorId::new(2),
    target: PLAYER,
  });
  assert_eq!(
    enemy_intent_summary(Some(&intent)),
    "Intent: enemy 2 chases actor 1"
  );
}

#[test]
fn hud_intent_summary_has_missing_and_generic_command_fallbacks() {
  assert_eq!(enemy_intent_summary(None), "Intent unavailable");
  let intent = PresentationEnemyIntent {
    actor: Some(ActorId::new(2)),
    command: Some(Command::Wait {
      actor: ActorId::new(2),
    }),
  };
  assert!(enemy_intent_summary(Some(&intent)).contains("Intent: enemy 2 Wait"));
}

#[test]
fn animation_pulse_is_bounded_and_expires() {
  let assert_near = |actual: f32, expected: f32| {
    assert!((actual - expected).abs() < 0.001);
  };
  assert_near(pulse_for_remaining(-1.0), 0.0);
  assert_near(pulse_for_remaining(ACTOR_PULSE_DURATION), 1.0);
  assert_near(pulse_for_remaining(ACTOR_PULSE_DURATION * 2.0), 1.0);

  let mut state = DesktopAnimationState::default();
  let cues = vec![PresentationAnimationCue::Died { actor: PLAYER }];
  state.update(None, Some(&cues), ACTOR_PULSE_DURATION * 2.0);
  assert_near(state.pulse(), 1.0);
  state.update(None, Some(&cues), ACTOR_PULSE_DURATION / 2.0);
  assert!((state.pulse() - 0.5).abs() < 0.001);
  state.update(None, None, ACTOR_PULSE_DURATION);
  assert_near(state.pulse(), 0.0);
  state.update(None, Some(&[]), 0.0);
  assert_near(state.pulse(), 0.0);
}

#[test]
fn value_identical_cues_retrigger_only_for_a_new_replay_token() {
  let assert_near = |actual: f32, expected: f32| {
    assert!((actual - expected).abs() < 0.001);
  };
  let mut runtime = PresentationRuntime::start_run(7).expect("starter run");
  let first_token = runtime.replay_digest();
  let cue = [PresentationAnimationCue::MovementBlocked {
    actor: PLAYER,
    from: Position::new(0, 0),
    to: Position::new(0, -1),
    reason: BlockReason::Terrain,
  }];
  let mut state = DesktopAnimationState::default();
  state.observe(Some(first_token), &cue);
  state.advance(ACTOR_PULSE_DURATION);
  state.observe(Some(first_token), &cue);
  assert_near(state.pulse(), 0.0);

  runtime
    .execute(Command::Move {
      actor: PLAYER,
      direction: Direction::East,
    })
    .expect("starter move accepted");
  let second_token = runtime.replay_digest();
  assert_ne!(first_token, second_token);
  state.observe(Some(second_token), &cue);
  assert_near(state.pulse(), 1.0);
}

#[test]
fn audio_batch_identity_retriggers_only_for_a_new_replay_token() {
  let mut runtime = PresentationRuntime::start_run(7).expect("starter run");
  let first_token = runtime.replay_digest();
  let cue = [PresentationAudioCue::Moved { actor: PLAYER }];
  let mut state = DesktopAudioState::default();
  assert!(state.observe(Some(first_token), &cue));
  assert!(!state.observe(Some(first_token), &cue));
  runtime
    .execute(Command::Move {
      actor: PLAYER,
      direction: Direction::East,
    })
    .expect("starter move accepted");
  let second_token = runtime.replay_digest();
  assert_ne!(first_token, second_token);
  assert!(state.observe(Some(second_token), &cue));
  assert!(!state.observe(Some(second_token), &[]));
}

#[test]
fn audio_manifest_and_asset_paths_are_complete_and_normalized() {
  let manifest = build_audio_manifest().expect("audio manifest validates");
  assert_eq!(manifest.bindings().len(), 8);
  for (_, reference) in manifest.bindings() {
    assert!(reference.path().starts_with("assets/audio/dreadstep/"));
    assert_eq!(
      audio_asset_path(reference.path()),
      Some(reference.path().trim_start_matches("assets/"))
    );
  }
  assert_eq!(audio_asset_path("audio/root.ogg"), None);
  assert_eq!(
    audio_asset_path("crates/dreadstep-bevy/audio/local.ogg"),
    None
  );
}

#[test]
fn audio_playback_system_is_safe_without_desktop_resources() {
  let mut app = App::new();
  app.add_systems(Update, desktop_play_audio);
  app.update();
}

fn audio_playback_app(manifest: PresentationAudioAssetManifest) -> App {
  let map = dreadstep_core::GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::Floor])
    .expect("audio map validates");
  let world = dreadstep_core::WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        PLAYER,
        ActorKind::Player,
        Position::new(0, 0),
        dreadstep_core::HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ATTACK_TARGET,
        ActorKind::Enemy,
        Position::new(1, 0),
        dreadstep_core::HitPoints::new(1),
      ),
    ],
  )
  .expect("audio world validates");
  let mut app = App::new();
  app.add_plugins((TaskPoolPlugin::default(), AssetPlugin::default()));
  app.init_asset::<AudioSource>();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(manifest);
  app.insert_resource(PresentationAudioAssetProjection::new());
  app.insert_resource(DesktopAudioState::default());
  app.add_plugins(PresentationPlugin);
  app.add_systems(
    Update,
    desktop_play_audio.after(PresentationSet::Projection),
  );
  app
}

fn test_audio_manifest(prefix: &str) -> PresentationAudioAssetManifest {
  let families = [
    (PresentationAudioCueKind::Moved, "moved"),
    (PresentationAudioCueKind::MovementBlocked, "blocked"),
    (PresentationAudioCueKind::Waited, "waited"),
    (PresentationAudioCueKind::Attacked, "attacked"),
    (PresentationAudioCueKind::Died, "died"),
    (PresentationAudioCueKind::ItemEquipped, "equipped"),
    (PresentationAudioCueKind::ItemUnequipped, "unequipped"),
    (PresentationAudioCueKind::ItemConsumed, "consumed"),
  ];
  PresentationAudioAssetManifest::new(
    families
      .into_iter()
      .map(|(family, name)| {
        (
          family,
          PresentationAssetReference::new(format!("assets/audio/dreadstep/{prefix}-{name}.wav"))
            .expect("test audio path validates"),
        )
      })
      .collect(),
  )
  .expect("test audio manifest validates")
}

fn install_test_audio_files(prefix: &str) -> [PathBuf; 2] {
  let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/audio/dreadstep");
  fs::create_dir_all(&directory).expect("test audio directory creates");
  let paths = [
    directory.join(format!("{prefix}-attacked.wav")),
    directory.join(format!("{prefix}-died.wav")),
  ];
  let mut wav = Vec::with_capacity(44);
  wav.extend_from_slice(b"RIFF");
  wav.extend_from_slice(&36_u32.to_le_bytes());
  wav.extend_from_slice(b"WAVEfmt ");
  wav.extend_from_slice(&16_u32.to_le_bytes());
  wav.extend_from_slice(&1_u16.to_le_bytes());
  wav.extend_from_slice(&1_u16.to_le_bytes());
  wav.extend_from_slice(&44_100_u32.to_le_bytes());
  wav.extend_from_slice(&44_100_u32.to_le_bytes());
  wav.extend_from_slice(&1_u16.to_le_bytes());
  wav.extend_from_slice(&8_u16.to_le_bytes());
  wav.extend_from_slice(b"data");
  wav.extend_from_slice(&0_u32.to_le_bytes());
  for path in &paths {
    fs::write(path, &wav).expect("test audio fixture writes");
  }
  paths
}

#[test]
fn existing_audio_files_spawn_ordered_non_looping_playback_requests() {
  let paths = install_test_audio_files("ordered");
  let mut app = audio_playback_app(test_audio_manifest("ordered"));
  app.update();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: PLAYER,
      target: ATTACK_TARGET,
    })
    .expect("adjacent attack should succeed");
  app.update();

  let mut query = app
    .world_mut()
    .query::<(&DesktopAudioPlayback, &PlaybackSettings)>();
  let requests = query
    .iter(app.world())
    .map(|(playback, settings)| (playback.0, settings.mode))
    .collect::<Vec<_>>();
  assert_eq!(requests.len(), 2);
  assert!(
    requests
      .iter()
      .all(|(_, mode)| matches!(mode, PlaybackMode::Despawn))
  );
  assert_eq!(
    requests.iter().map(|(cue, _)| *cue).collect::<Vec<_>>(),
    vec![
      PresentationAudioCue::Attacked {
        attacker: PLAYER,
        target: ATTACK_TARGET,
      },
      PresentationAudioCue::Died {
        actor: ATTACK_TARGET,
      },
    ]
  );
  for path in paths {
    let _ = fs::remove_file(path);
  }
}

#[test]
fn manifest_loss_skips_stale_audio_and_restoration_plays_current_batch_once() {
  let paths = install_test_audio_files("restore");
  let manifest = test_audio_manifest("restore");
  let mut app = audio_playback_app(manifest.clone());
  app.update();
  app
    .world_mut()
    .remove_resource::<PresentationAudioAssetManifest>();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Attack {
      actor: PLAYER,
      target: ATTACK_TARGET,
    })
    .expect("adjacent attack should succeed");
  app.update();
  assert_eq!(
    app
      .world_mut()
      .query::<&DesktopAudioPlayback>()
      .iter(app.world())
      .count(),
    0
  );
  app.insert_resource(manifest);
  app.update();
  assert_eq!(
    app
      .world_mut()
      .query::<&DesktopAudioPlayback>()
      .iter(app.world())
      .count(),
    2
  );
  app.update();
  assert_eq!(
    app
      .world_mut()
      .query::<&DesktopAudioPlayback>()
      .iter(app.world())
      .count(),
    2
  );
  for path in paths {
    let _ = fs::remove_file(path);
  }
}

#[test]
fn pulse_scale_only_changes_visible_living_actor_placeholders() {
  let assert_near = |actual: f32, expected: f32| {
    assert!((actual - expected).abs() < 0.001);
  };
  let pulse = 1.0;
  assert!(sprite_scale(SceneRenderPlaceholder::Player, true, pulse) > 0.75);
  assert!(sprite_scale(SceneRenderPlaceholder::Enemy, true, pulse) > 0.75);
  assert_near(
    sprite_scale(SceneRenderPlaceholder::Enemy, false, pulse),
    0.75,
  );
  assert_near(
    sprite_scale(SceneRenderPlaceholder::DeadActor, true, pulse),
    0.65,
  );
  assert_near(
    sprite_scale(SceneRenderPlaceholder::Terrain, true, pulse),
    1.0,
  );
  assert_near(
    sprite_scale(SceneRenderPlaceholder::GroundItem, true, pulse),
    0.45,
  );
  assert_near(
    sprite_scale(SceneRenderPlaceholder::InventoryItem, true, pulse),
    0.45,
  );
}

#[derive(Clone)]
struct SpriteSnapshot {
  entity: Entity,
  sprite: bevy::sprite::Sprite,
  visibility: Visibility,
  transform: Option<Transform>,
}

fn animation_system_app() -> App {
  let map = dreadstep_core::GridMap::from_tiles(
    5,
    3,
    vec![
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Floor,
      Tile::Floor,
      Tile::Floor,
      Tile::Wall,
      Tile::Floor,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
      Tile::Wall,
    ],
  )
  .expect("animation map validates");
  let mut world = dreadstep_core::WorldState::new(
    map,
    vec![
      Actor::new(PLAYER, ActorKind::Player, Position::new(0, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(2, 1)),
    ],
  )
  .expect("animation world validates");
  world
    .give_item(
      PLAYER,
      Item::new(ItemId::new(10), dreadstep_core::ItemDefinitionId::new(101)),
    )
    .expect("inventory item validates");
  world
    .give_item(
      ActorId::new(2),
      Item::new(ItemId::new(11), dreadstep_core::ItemDefinitionId::new(102)),
    )
    .expect("ground item validates");
  world
    .drop_item(ActorId::new(2), ItemId::new(11))
    .expect("ground item drops");

  let mut app = App::new();
  app.insert_resource(PresentationRuntime::new(PresentationState::new(7, world)));
  app.insert_resource(PresentationInput::new(PLAYER));
  app.insert_resource(PresentationKeyboardMode::External);
  app.insert_resource(PresentationVisibility::new(PLAYER, 1));
  app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size validates"));
  app.insert_resource(PresentationRenderProjection::new());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.insert_resource(PresentationBevySpriteTransformProjection::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(DesktopAnimationState::default());
  app.insert_resource(Time::<()>::default());
  app.add_plugins(PresentationPlugin);
  app.add_systems(
    Update,
    (desktop_update_animation, desktop_style_sprites)
      .chain()
      .after(PresentationSet::Projection),
  );
  app.update();
  app
}

fn capture_placeholder(app: &mut App, placeholder: SceneRenderPlaceholder) -> SpriteSnapshot {
  app
    .world_mut()
    .query::<(
      Entity,
      &SceneRenderNode,
      &bevy::sprite::Sprite,
      &Visibility,
      Option<&Transform>,
    )>()
    .iter(app.world())
    .find_map(|(entity, node, sprite, visibility, transform)| {
      (node.placeholder() == placeholder).then_some(SpriteSnapshot {
        entity,
        sprite: sprite.clone(),
        visibility: *visibility,
        transform: transform.copied(),
      })
    })
    .expect("placeholder sprite should exist")
}

#[test]
fn desktop_animation_system_pulses_visible_actor_without_touching_other_visual_state() {
  let mut app = animation_system_app();
  let player_before = capture_placeholder(&mut app, SceneRenderPlaceholder::Player);
  let enemy_before = capture_placeholder(&mut app, SceneRenderPlaceholder::Enemy);
  let terrain_before = capture_placeholder(&mut app, SceneRenderPlaceholder::Terrain);
  let ground_before = capture_placeholder(&mut app, SceneRenderPlaceholder::GroundItem);
  let inventory_before = capture_placeholder(&mut app, SceneRenderPlaceholder::InventoryItem);

  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Move {
      actor: PLAYER,
      direction: Direction::West,
    })
    .expect("blocked movement cue should be accepted");
  app.update();

  let player_after = capture_placeholder(&mut app, SceneRenderPlaceholder::Player);
  let enemy_after = capture_placeholder(&mut app, SceneRenderPlaceholder::Enemy);
  let terrain_after = capture_placeholder(&mut app, SceneRenderPlaceholder::Terrain);
  let ground_after = capture_placeholder(&mut app, SceneRenderPlaceholder::GroundItem);
  let inventory_after = capture_placeholder(&mut app, SceneRenderPlaceholder::InventoryItem);

  assert_eq!(player_before.entity, player_after.entity);
  assert!(
    player_after.sprite.custom_size.expect("player size").x
      > player_before.sprite.custom_size.expect("player size").x
  );
  for (before, after) in [
    (&player_before, &player_after),
    (&enemy_before, &enemy_after),
    (&terrain_before, &terrain_after),
    (&ground_before, &ground_after),
    (&inventory_before, &inventory_after),
  ] {
    assert_eq!(before.sprite.image, after.sprite.image);
    assert_eq!(before.sprite.color, after.sprite.color);
    assert_eq!(before.visibility, after.visibility);
    assert_eq!(before.transform, after.transform);
  }
  assert_eq!(
    enemy_before.sprite.custom_size,
    enemy_after.sprite.custom_size
  );
  assert_eq!(
    terrain_before.sprite.custom_size,
    terrain_after.sprite.custom_size
  );
  assert_eq!(
    ground_before.sprite.custom_size,
    ground_after.sprite.custom_size
  );
  assert_eq!(
    inventory_before.sprite.custom_size,
    inventory_after.sprite.custom_size
  );
}
