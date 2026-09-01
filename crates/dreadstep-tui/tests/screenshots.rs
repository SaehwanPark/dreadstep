//! README screenshot goldens must match the live TUI renderer.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dreadstep_core::{Command, Position};
use dreadstep_protocol::{REPLAY_EXPORT_SCHEMA_VERSION, ReplayExport, ReplayScenario, RunOutcome};
use dreadstep_tui::{PLAYER, Session, UiState, format_event, render_frame, run};

fn test_directory(label: &str) -> PathBuf {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system clock is after epoch")
    .as_nanos();
  std::env::temp_dir().join(format!(
    "dreadstep-tui-{label}-{}-{timestamp}",
    std::process::id()
  ))
}

#[test]
fn starter_screenshot_matches_renderer() {
  let session = Session::start_item_run(7).expect("item showcase");
  let mut ui = UiState::new();
  ui.select_default_item(&session);
  let rendered = format!("{}\n", render_frame(&session, &ui).plain());
  let committed = include_str!("../../../screenshots/tui-starter.txt");
  assert_eq!(rendered, committed);
}

#[test]
fn status_screenshot_matches_renderer_after_opening_the_door() {
  let mut session = Session::start_item_run(7).expect("item showcase");
  let mut ui = UiState::new();
  ui.select_default_item(&session);
  let output = session
    .execute(Command::Interact {
      actor: PLAYER,
      position: Position::new(2, 1),
    })
    .expect("open starter door");
  ui.push_message(format_event(&session, output.events()[0]));
  let rendered = format!("{}\n", render_frame(&session, &ui).plain());
  let committed = include_str!("../../../screenshots/tui-status.txt");
  assert_eq!(rendered, committed);
}

#[test]
fn readme_embeds_screenshot_files() {
  let readme = include_str!("../../../README.md");
  let starter = include_str!("../../../screenshots/tui-starter.txt").trim_end();
  let status = include_str!("../../../screenshots/tui-status.txt").trim_end();
  assert!(
    readme.contains(starter),
    "README.md must embed screenshots/tui-starter.txt"
  );
  assert!(
    readme.contains(status),
    "README.md must embed screenshots/tui-status.txt"
  );
}

#[test]
fn capture_writes_typed_item_showcase_replay_export() {
  let root = test_directory("capture");
  let capture_dir = root.join("capture");
  let log_dir = root.join("logs");
  let exit = run([
    "--capture",
    capture_dir.to_str().expect("capture path should be UTF-8"),
    "--log-dir",
    log_dir.to_str().expect("log path should be UTF-8"),
    "--seed",
    "7",
  ]);
  assert_eq!(exit, std::process::ExitCode::SUCCESS);

  let replay_path = fs::read_dir(&log_dir)
    .expect("log directory should read")
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .find(|path| {
      path
        .extension()
        .is_some_and(|extension| extension == "json")
    })
    .expect("capture should write a replay export");
  let replay = serde_json::from_str::<ReplayExport>(
    &fs::read_to_string(replay_path).expect("replay export should read"),
  )
  .expect("replay export should decode");
  assert_eq!(replay.schema_version(), REPLAY_EXPORT_SCHEMA_VERSION);
  assert_eq!(replay.seed(), 7);
  assert_eq!(replay.scenario(), ReplayScenario::ItemShowcase);
  assert_eq!(replay.commands().len(), 1);
  assert_eq!(replay.outcome(), RunOutcome::InProgress);
  assert_ne!(replay.replay_digest().value(), 0);
  assert_ne!(replay.state_digest().value(), 0);
  assert!(capture_dir.join("tui-starter.txt").is_file());
  assert!(capture_dir.join("tui-status.txt").is_file());
  let _ = fs::remove_dir_all(root);
}

#[test]
fn procedural_capture_is_rejected_before_startup() {
  let root = test_directory("procedural-capture");
  let exit = run([
    "--procedural",
    "--capture",
    root.to_str().expect("capture path should be UTF-8"),
  ]);
  assert_eq!(exit, std::process::ExitCode::from(2));
  assert!(!root.exists());
}
