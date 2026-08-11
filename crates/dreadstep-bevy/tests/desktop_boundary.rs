//! Red-first contracts for the runnable desktop presentation boundary.

use bevy::app::App;
use bevy::camera::visibility::Visibility;
use bevy::ecs::entity::Entity;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::window::{PrimaryWindow, Window};
use dreadstep_bevy::{
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection,
  PresentationKeyboardMode, PresentationPlugin, PresentationRenderCommandPlan,
  PresentationRenderNodeProjection, PresentationRenderProjection, PresentationRuntime,
  PresentationSpriteProjection, PresentationState, PresentationTileSize, PresentationWindow,
};
use dreadstep_core::{ActorId, Command, Direction};

#[cfg(feature = "desktop")]
use dreadstep_core::Event;
#[cfg(feature = "desktop")]
use serde_json::Value;

#[test]
fn runtime_exposes_core_legal_commands_without_mutation() {
  let runtime = PresentationRuntime::start_item_run(7).expect("starter item run validates");
  let before = runtime.snapshot();
  let commands = runtime.legal_commands();

  assert!(commands.contains(&Command::Move {
    actor: ActorId::new(1),
    direction: Direction::East,
  }));
  assert!(commands.iter().any(|command| matches!(
    command,
    Command::Equip {
      actor,
      item
    } if *actor == ActorId::new(1) && item.value() == 101
  )));
  assert_eq!(runtime.snapshot(), before);
  assert_eq!(
    runtime.legal_commands(),
    PresentationState::start_item_run(7)
      .expect("second starter item run validates")
      .legal_commands()
  );
  assert_eq!(
    runtime.replay_digest(),
    PresentationState::start_item_run(7)
      .unwrap()
      .replay_digest()
  );
}

#[test]
fn external_keyboard_mode_leaves_command_submission_to_the_desktop_driver() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("starter run validates"));
  app.insert_resource(dreadstep_bevy::PresentationInput::new(ActorId::new(1)));
  app.insert_resource(ButtonInput::<KeyCode>::default());
  app.insert_resource(PresentationKeyboardMode::External);
  app.add_plugins(PresentationPlugin);
  app
    .world_mut()
    .resource_mut::<ButtonInput<KeyCode>>()
    .press(KeyCode::ArrowRight);

  app.update();

  assert_eq!(
    app
      .world()
      .resource::<PresentationRuntime>()
      .replay_digest(),
    PresentationRuntime::start_run(7).unwrap().replay_digest()
  );
}

#[test]
fn presentation_window_reuses_primary_window_and_does_not_spawn_a_second_window() {
  let mut app = App::new();
  let primary = app
    .world_mut()
    .spawn((PrimaryWindow, Window::default()))
    .id();
  app.insert_resource(PresentationWindow::new(640, 360, 2).expect("window validates"));
  app.add_plugins(PresentationPlugin);
  app.update();

  let mut windows = app.world_mut().query::<(Entity, &Window)>();
  let entries = windows.iter(app.world()).collect::<Vec<_>>();
  assert_eq!(entries.len(), 1);
  assert_eq!(entries[0].0, primary);
  assert_eq!(entries[0].1.resolution.physical_width(), 1280);
  assert_eq!(entries[0].1.resolution.physical_height(), 720);
}

#[test]
fn inventory_render_nodes_are_hidden_until_a_hud_places_them() {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_item_run(7).expect("starter item run validates"));
  app.insert_resource(PresentationKeyboardMode::External);
  app.insert_resource(PresentationTileSize::new(32, 32).expect("tile size validates"));
  app.insert_resource(PresentationRenderProjection::default());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.insert_resource(PresentationBevySpriteTransformProjection::new());
  app.add_plugins(PresentationPlugin);
  app.update();

  let entries = app
    .world()
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let hidden = entries
    .iter()
    .filter(|entry| {
      entry.node().placeholder() == dreadstep_bevy::SceneRenderPlaceholder::InventoryItem
    })
    .map(|entry| {
      app
        .world()
        .get_entity(entry.node_entity())
        .expect("inventory node exists")
        .get::<Visibility>()
        .copied()
    })
    .collect::<Vec<_>>();
  assert!(!hidden.is_empty());
  assert!(
    hidden
      .iter()
      .all(|value| *value == Some(Visibility::Hidden))
  );
}

#[cfg(feature = "desktop")]
#[test]
fn showcase_event_formatter_remains_exhaustive_for_current_events() {
  let events = [Event::Waited {
    actor: ActorId::new(1),
    at: dreadstep_core::ActionTime::new(0),
  }];
  assert_eq!(dreadstep_bevy::showcase_event_name(events[0]), "waited");
}

#[cfg(feature = "desktop")]
#[test]
fn smoke_binary_is_display_free_and_emits_complete_ordered_jsonl() {
  use std::fs;
  use std::process::Command as ProcessCommand;

  let root = std::env::temp_dir().join(format!(
    "dreadstep-showcase-test-{}-{}",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .expect("system clock is after epoch")
      .as_nanos()
  ));
  fs::create_dir_all(&root).expect("test journal directory creates");
  let output = ProcessCommand::new(env!("CARGO_BIN_EXE_dreadstep"))
    .args(["--smoke", "--seed", "7", "--log-dir"])
    .arg(&root)
    .output()
    .expect("desktop binary starts");
  assert!(
    output.status.success(),
    "stderr: {}",
    String::from_utf8_lossy(&output.stderr)
  );

  let files = fs::read_dir(&root)
    .expect("journal directory reads")
    .collect::<Result<Vec<_>, _>>()
    .expect("journal entries read");
  assert_eq!(files.len(), 1);
  let journal = fs::read_to_string(files[0].path()).expect("journal reads");
  let records = journal
    .lines()
    .map(|line| serde_json::from_str::<Value>(line).expect("each journal line is JSON"))
    .collect::<Vec<_>>();
  assert!(!records.is_empty());
  for (index, record) in records.iter().enumerate() {
    assert_eq!(record["schema_version"], 1);
    assert_eq!(record["sequence"], index as u64 + 1);
    assert!(record["payload"]["state"].is_object() || record["kind"] == "shutdown");
  }
  let complete = records
    .iter()
    .find(|record| record["kind"] == "smoke_complete")
    .expect("smoke completion record");
  for kind in [
    "move", "wait", "attack", "chase", "equip", "unequip", "use_item",
  ] {
    assert!(
      complete["payload"]["extra"]["commands_observed"]
        .as_array()
        .expect("command coverage array")
        .iter()
        .any(|value| value == kind)
    );
  }
  for kind in [
    "moved",
    "movement_blocked",
    "waited",
    "attacked",
    "died",
    "item_equipped",
    "item_unequipped",
    "item_consumed",
  ] {
    assert!(
      complete["payload"]["extra"]["events_observed"]
        .as_array()
        .expect("event coverage array")
        .iter()
        .any(|value| value == kind)
    );
  }
  assert_eq!(
    records.last().map(|record| &record["kind"]),
    Some(&Value::from("shutdown"))
  );
}

#[cfg(feature = "desktop")]
#[test]
fn smoke_runs_have_identical_semantic_evidence() {
  use std::fs;
  use std::process::Command as ProcessCommand;

  let base = std::env::temp_dir().join(format!(
    "dreadstep-showcase-determinism-{}-{}",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .expect("system clock is after epoch")
      .as_nanos()
  ));
  let directories = [base.join("one"), base.join("two")];
  let mut normalized = Vec::new();
  for directory in directories {
    fs::create_dir_all(&directory).expect("determinism journal directory creates");
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_dreadstep"))
      .args(["--smoke", "--seed", "7", "--log-dir"])
      .arg(&directory)
      .output()
      .expect("desktop binary starts");
    assert!(
      output.status.success(),
      "stderr: {}",
      String::from_utf8_lossy(&output.stderr)
    );
    let file = fs::read_dir(&directory)
      .expect("determinism journal directory reads")
      .next()
      .expect("determinism journal exists")
      .expect("determinism journal entry reads")
      .path();
    let records = fs::read_to_string(file)
      .expect("determinism journal reads")
      .lines()
      .map(|line| serde_json::from_str::<Value>(line).expect("determinism journal parses"))
      .map(|mut value| {
        value["elapsed_ms"] = Value::from(0_u64);
        value["payload"]["extra"]["journal"] = Value::from("<journal>");
        value
      })
      .collect::<Vec<_>>();
    normalized.push(records);
  }
  assert_eq!(normalized[0], normalized[1]);
}

#[cfg(feature = "desktop")]
#[test]
fn malformed_cli_and_unusable_log_destination_exit_without_panic_text() {
  use std::fs;
  use std::process::Command as ProcessCommand;

  let invalid = ProcessCommand::new(env!("CARGO_BIN_EXE_dreadstep"))
    .arg("--not-a-real-option")
    .output()
    .expect("desktop binary starts for invalid CLI");
  assert_eq!(invalid.status.code(), Some(2));
  assert!(!String::from_utf8_lossy(&invalid.stderr).contains("panicked"));

  let root = std::env::temp_dir().join(format!(
    "dreadstep-showcase-bad-log-{}-{}",
    std::process::id(),
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .expect("system clock is after epoch")
      .as_nanos()
  ));
  fs::create_dir_all(&root).expect("bad-log test directory creates");
  let file = root.join("not-a-directory");
  fs::write(&file, b"occupied").expect("bad-log sentinel writes");
  let invalid_log = ProcessCommand::new(env!("CARGO_BIN_EXE_dreadstep"))
    .args(["--smoke", "--log-dir"])
    .arg(file.join("child"))
    .output()
    .expect("desktop binary starts for invalid log path");
  assert_eq!(invalid_log.status.code(), Some(1));
  assert!(!String::from_utf8_lossy(&invalid_log.stderr).contains("panicked"));
}
