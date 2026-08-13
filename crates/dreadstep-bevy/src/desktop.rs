//! Runnable desktop showcase and diagnostic journal.
//!
//! This module is intentionally a process/presentation boundary.  It owns command-line parsing,
//! the OS window, optional local art, human input, and JSONL diagnostics; the deterministic core
//! remains inside [`crate::PresentationRuntime`].  The smoke runner uses the same action and
//! enemy-driver helpers without creating a renderer, which keeps CI display-free and makes the
//! visible path easy to regression-test.

#![cfg(feature = "desktop")]
#![allow(clippy::needless_continue, clippy::needless_pass_by_value)]

use std::collections::{BTreeSet, VecDeque};
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::app::{AppExit, Last, PanicHandlerPlugin, Plugin, PluginGroup, Startup, Update};
use bevy::asset::{AssetServer, Handle, LoadState};
use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageReader;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::image::Image;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::math::Vec2;
use bevy::prelude::{
  App, BackgroundColor, BorderColor, DefaultPlugins, ImagePlugin, Node, PositionType, Text,
  TextColor, TextFont, Timer, TimerMode, WindowPlugin, default,
};
use bevy::time::Time;
use bevy::ui::{FlexDirection, JustifyContent, Overflow, UiRect, px};
use bevy::window::{ClosingWindow, PrimaryWindow, Window};
use dreadstep_content::ContentError;
use dreadstep_core::{
  Actor, ActorId, ActorKind, BlockReason, Command, Direction, Event, Item, ItemId, Position,
  RunOutcome, StateDigest, Tile,
};
use serde_json::{Value, json};

use crate::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAssetManifest,
  PresentationAssetReference, PresentationAudioAssetManifest, PresentationAudioAssetProjection,
  PresentationAudioCue, PresentationAudioCueKind, PresentationAudioCues,
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationCamera,
  PresentationEnemyIntent, PresentationFocus, PresentationInput, PresentationKeyboardMode,
  PresentationMessages, PresentationPlugin, PresentationRenderAssetProjection,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSet, PresentationSnapshot, PresentationSpriteProjection,
  PresentationTileSize, PresentationVisibility, SceneRenderNode, SceneRenderPlaceholder,
};

const PLAYER: ActorId = ActorId::new(1);
const ATTACK_TARGET: ActorId = ActorId::new(2);
const RANGED_TARGET: ActorId = ActorId::new(3);
const EQUIP_ITEM: ItemId = ItemId::new(101);
const PICKUP_ITEM: ItemId = ItemId::new(102);
const SMOKE_ENEMY_ATTACK_LIMIT: usize = 32;
const ENEMY_DELAY: Duration = Duration::from_millis(150);
const SHOWCASE_MAX_HIT_POINTS: i32 = 10;
const HEALTH_BAR_WIDTH: usize = 10;
const REPLAY_EXPORT_SCHEMA_VERSION: u16 = 1;

/// Every current command kind that must remain demonstrable by the desktop smoke path.
pub const SHOWCASE_COMMAND_KINDS: [&str; 14] = [
  "move",
  "wait",
  "interact",
  "kick",
  "break",
  "attack",
  "ranged_attack",
  "chase",
  "equip",
  "unequip",
  "use_item",
  "pickup",
  "drop",
  "reload",
];

/// Every current event kind that must remain observable in the desktop smoke path.
pub const SHOWCASE_EVENT_KINDS: [&str; 15] = [
  "moved",
  "movement_blocked",
  "waited",
  "door_opened",
  "noise_created",
  "breakable_broken",
  "trap_triggered",
  "attacked",
  "died",
  "item_equipped",
  "item_unequipped",
  "item_consumed",
  "item_picked_up",
  "item_dropped",
  "reloaded",
];

const USAGE: &str = "Usage: dreadstep [--seed <u64>] [--procedural] [--depth <u32>] [--log-dir <path>] [--smoke] [--help]";

/// Parsed command-line options for the desktop process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopOptions {
  /// Seed passed to the deterministic starter-item scenario.
  pub seed: u64,
  /// Start the visible client from the seeded procedural floor instead of the item fixture.
  pub procedural: bool,
  /// One-based authored depth passed to the procedural floor generator.
  pub depth: u32,
  /// Directory in which this run's JSONL journal is created.
  pub log_dir: PathBuf,
  /// Run the display-free deterministic showcase sequence.
  pub smoke: bool,
}

impl Default for DesktopOptions {
  fn default() -> Self {
    Self {
      seed: 7,
      procedural: false,
      depth: 1,
      log_dir: PathBuf::from("dreadstep-logs"),
      smoke: false,
    }
  }
}

/// Runs the desktop showcase process and returns a portable process exit code.
pub fn run<I>(arguments: I) -> ExitCode
where
  I: IntoIterator<Item = OsString>,
{
  let options = match parse_options(arguments) {
    Ok(ParseResult::Help) => {
      println!("{USAGE}");
      return ExitCode::SUCCESS;
    }
    Ok(ParseResult::Options(options)) => options,
    Err(error) => {
      eprintln!("dreadstep: {error}\n{USAGE}");
      return ExitCode::from(2);
    }
  };

  run_with_panic_boundary(options)
}

#[derive(Debug, Eq, PartialEq)]
enum ParseResult {
  Help,
  Options(DesktopOptions),
}

#[expect(
  clippy::too_many_lines,
  reason = "the desktop process keeps its small exhaustive CLI grammar in one parser"
)]
fn parse_options<I>(arguments: I) -> Result<ParseResult, String>
where
  I: IntoIterator<Item = OsString>,
{
  let mut options = DesktopOptions::default();
  let mut seed_seen = false;
  let mut procedural_seen = false;
  let mut depth_seen = false;
  let mut log_dir_seen = false;
  let mut smoke_seen = false;
  let mut help_seen = false;
  let mut iter = arguments.into_iter();
  while let Some(argument) = iter.next() {
    if argument == "--help" || argument == "-h" {
      if help_seen {
        return Err("duplicate --help".to_string());
      }
      help_seen = true;
      continue;
    }
    if argument == "--smoke" {
      if smoke_seen {
        return Err("duplicate --smoke".to_string());
      }
      smoke_seen = true;
      options.smoke = true;
      continue;
    }
    if argument == "--procedural" {
      if procedural_seen {
        return Err("duplicate --procedural".to_string());
      }
      procedural_seen = true;
      options.procedural = true;
      continue;
    }
    if argument == "--seed" {
      if seed_seen {
        return Err("duplicate --seed".to_string());
      }
      seed_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--seed requires a value".to_string())?;
      if matches!(
        value.to_str(),
        Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
      ) {
        return Err("--seed requires a value".to_string());
      }
      let value = value
        .into_string()
        .map_err(|_| "--seed must be an unsigned integer".to_string())?;
      options.seed = value
        .parse::<u64>()
        .map_err(|_| "--seed must be an unsigned integer".to_string())?;
      continue;
    }
    if argument == "--depth" {
      if depth_seen {
        return Err("duplicate --depth".to_string());
      }
      depth_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--depth requires a value".to_string())?;
      if matches!(
        value.to_str(),
        Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
      ) {
        return Err("--depth requires a value".to_string());
      }
      let value = value
        .into_string()
        .map_err(|_| "--depth must be an unsigned integer".to_string())?;
      options.depth = value
        .parse::<u32>()
        .map_err(|_| "--depth must be an unsigned integer".to_string())?;
      continue;
    }
    if argument == "--log-dir" {
      if log_dir_seen {
        return Err("duplicate --log-dir".to_string());
      }
      log_dir_seen = true;
      let value = iter
        .next()
        .ok_or_else(|| "--log-dir requires a path".to_string())?;
      if value.is_empty()
        || matches!(
          value.to_str(),
          Some("--help" | "-h" | "--seed" | "--procedural" | "--depth" | "--log-dir" | "--smoke",)
        )
      {
        return Err("--log-dir requires a path".to_string());
      }
      options.log_dir = PathBuf::from(value);
      continue;
    }
    let display = argument.to_string_lossy();
    return Err(format!("unknown argument {display}"));
  }
  if help_seen {
    Ok(ParseResult::Help)
  } else {
    Ok(ParseResult::Options(options))
  }
}

fn run_with_panic_boundary(options: DesktopOptions) -> ExitCode {
  let journal = match Journal::open(&options.log_dir) {
    Ok(journal) => Arc::new(Mutex::new(journal)),
    Err(error) => {
      eprintln!(
        "dreadstep: cannot create journal in {}: {error}",
        options.log_dir.display()
      );
      return ExitCode::from(1);
    }
  };

  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    run_with_journal(options, Arc::clone(&journal))
  }));
  match result {
    Ok(code) => code,
    Err(payload) => {
      let message = panic_message(payload);
      let _ = record(&journal, "unexpected_panic", json!({ "message": message }));
      eprintln!("dreadstep: unexpected panic: {message}");
      ExitCode::from(1)
    }
  }
}

fn run_with_journal(options: DesktopOptions, journal: JournalHandle) -> ExitCode {
  let scenario = options.procedural && !options.smoke;
  let initial_runtime = match start_runtime(&options) {
    Ok(runtime) => runtime,
    Err(error) => {
      eprintln!("dreadstep: starter scenario failed: {error}");
      let _ = record(
        &journal,
        "startup_fault",
        json!({ "error": error.to_string() }),
      );
      return ExitCode::from(1);
    }
  };
  if let Err(error) = record(
    &journal,
    "run_started",
    state_payload(
      &initial_runtime,
      json!({
        "seed": options.seed,
        "mode": if options.smoke { "smoke" } else { "desktop" },
        "scenario": if scenario {
          "procedural_floor"
        } else {
          "starter_item_floor"
        },
        "depth": scenario.then_some(options.depth),
      }),
    ),
  ) {
    eprintln!("dreadstep: cannot write run start: {error}");
    return ExitCode::from(1);
  }

  if options.smoke {
    return run_smoke(initial_runtime, journal);
  }

  match run_visible(
    initial_runtime,
    options.seed,
    scenario,
    options.depth,
    journal,
  ) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("dreadstep: runtime failure: {error}");
      ExitCode::from(1)
    }
  }
}

fn start_runtime(options: &DesktopOptions) -> Result<PresentationRuntime, ContentError> {
  if options.procedural && !options.smoke {
    PresentationRuntime::start_procedural_run(options.seed, options.depth)
  } else {
    PresentationRuntime::start_item_run(options.seed)
  }
}

type JournalHandle = Arc<Mutex<Journal>>;

struct Journal {
  writer: BufWriter<File>,
  path: PathBuf,
  started: Instant,
  sequence: u64,
}

impl Journal {
  fn open(directory: &Path) -> io::Result<Self> {
    fs::create_dir_all(directory)?;
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(duration) => duration.as_millis(),
      Err(_) => 0,
    };
    let pid = std::process::id();
    for counter in 0_u32..10_000 {
      let suffix = if counter == 0 {
        String::new()
      } else {
        format!("-{counter}")
      };
      let path = directory.join(format!("run-{timestamp}-{pid}{suffix}.jsonl"));
      match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(file) => {
          return Ok(Self {
            writer: BufWriter::new(file),
            path,
            started: Instant::now(),
            sequence: 0,
          });
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
        Err(error) => return Err(error),
      }
    }
    Err(io::Error::new(
      io::ErrorKind::AlreadyExists,
      "could not allocate a unique run journal filename",
    ))
  }

  fn path(&self) -> &Path {
    &self.path
  }

  fn record(&mut self, kind: &str, payload: Value) -> Result<(), JournalError> {
    self.sequence = self
      .sequence
      .checked_add(1)
      .ok_or_else(|| JournalError("journal sequence overflow".to_string()))?;
    let entry = json!({
      "schema_version": 1,
      "sequence": self.sequence,
      "elapsed_ms": self.started.elapsed().as_millis(),
      "kind": kind,
      "payload": payload,
    });
    serde_json::to_writer(&mut self.writer, &entry).map_err(JournalError::serialize)?;
    self.writer.write_all(b"\n").map_err(JournalError::io)?;
    self.writer.flush().map_err(JournalError::io)
  }
}

#[derive(Debug)]
struct JournalError(String);

impl JournalError {
  fn io(error: io::Error) -> Self {
    Self(error.to_string())
  }

  fn serialize(error: serde_json::Error) -> Self {
    Self(error.to_string())
  }
}

impl fmt::Display for JournalError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.0)
  }
}

fn record(journal: &JournalHandle, kind: &str, payload: Value) -> Result<(), JournalError> {
  let mut guard = match journal.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
  };
  guard.record(kind, payload)
}

fn journal_path(journal: &JournalHandle) -> PathBuf {
  let guard = match journal.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
  };
  guard.path().to_path_buf()
}

fn replay_export_value(runtime: &PresentationRuntime) -> Value {
  json!({
    "schema_version": REPLAY_EXPORT_SCHEMA_VERSION,
    "seed": runtime.seed(),
    "commands": runtime
      .replay_commands()
      .iter()
      .copied()
      .map(command_value)
      .collect::<Vec<_>>(),
    "replay_digest": runtime.replay_digest().value(),
    "outcome": outcome_name(runtime.snapshot().outcome()),
  })
}

fn export_replay(
  runtime: &PresentationRuntime,
  journal: &JournalHandle,
) -> Result<PathBuf, String> {
  let journal_path = journal_path(journal);
  let stem = journal_path
    .file_stem()
    .and_then(|value| value.to_str())
    .ok_or_else(|| "run journal path has no valid filename stem".to_string())?;
  let directory = journal_path
    .parent()
    .ok_or_else(|| "run journal path has no parent directory".to_string())?;
  let export = replay_export_value(runtime);
  for counter in 0_u32..10_000 {
    let suffix = if counter == 0 {
      String::new()
    } else {
      format!("-{counter}")
    };
    let path = directory.join(format!("{stem}.replay{suffix}.json"));
    match OpenOptions::new().create_new(true).write(true).open(&path) {
      Ok(mut file) => {
        serde_json::to_writer_pretty(&mut file, &export).map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        record(
          journal,
          "replay_exported",
          json!({
            "path": path.display().to_string(),
            "schema_version": REPLAY_EXPORT_SCHEMA_VERSION,
            "commands": runtime.replay_commands().len(),
            "replay_digest": runtime.replay_digest().value(),
            "outcome": outcome_name(runtime.snapshot().outcome()),
          }),
        )
        .map_err(|error| error.to_string())?;
        return Ok(path);
      }
      Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(error.to_string()),
    }
  }
  Err("could not allocate a unique replay export filename".to_string())
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    return (*message).to_string();
  }
  if let Some(message) = payload.downcast_ref::<String>() {
    return message.clone();
  }
  "non-string panic payload".to_string()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DesktopStatus {
  Running,
  Victory,
  Defeat,
  Faulted(String),
  Shutdown(String),
}

#[derive(Resource)]
struct DesktopSession {
  seed: u64,
  procedural: bool,
  depth: u32,
  journal: JournalHandle,
  status: DesktopStatus,
  selected_item: Option<ItemId>,
  messages: VecDeque<String>,
  enemy_timer: Timer,
  command_kinds: BTreeSet<String>,
  event_kinds: BTreeSet<String>,
  terminal_recorded: bool,
}

impl DesktopSession {
  fn new(seed: u64, journal: JournalHandle) -> Self {
    Self::new_with_scenario(seed, false, 1, journal)
  }

  fn new_with_scenario(seed: u64, procedural: bool, depth: u32, journal: JournalHandle) -> Self {
    Self {
      seed,
      procedural,
      depth,
      journal,
      status: DesktopStatus::Running,
      selected_item: Some(EQUIP_ITEM),
      messages: VecDeque::new(),
      enemy_timer: Timer::from_seconds(ENEMY_DELAY.as_secs_f32(), TimerMode::Once),
      command_kinds: BTreeSet::new(),
      event_kinds: BTreeSet::new(),
      terminal_recorded: false,
    }
  }

  fn push_message(&mut self, message: impl Into<String>) {
    self.messages.push_back(message.into());
    while self.messages.len() > 8 {
      let _ = self.messages.pop_front();
    }
  }

  fn fault(&mut self, error: impl Into<String>) {
    let error = error.into();
    self.status = DesktopStatus::Faulted(error.clone());
    self.push_message(format!("Journal/runtime fault: {error}"));
    eprintln!("dreadstep: {error}");
  }
}

#[derive(Default)]
struct FinalizationReport {
  complete: bool,
  error: Option<String>,
}

#[derive(Clone, Resource)]
struct FinalizationHandle(Arc<Mutex<FinalizationReport>>);

impl FinalizationHandle {
  fn new() -> Self {
    Self(Arc::new(Mutex::new(FinalizationReport::default())))
  }

  fn finish(&self, error: Option<String>) {
    let mut report = match self.0.lock() {
      Ok(report) => report,
      Err(poisoned) => poisoned.into_inner(),
    };
    report.complete = true;
    report.error = error;
  }
}

fn record_session(session: &mut DesktopSession, kind: &str, payload: Value) -> bool {
  match record(&session.journal, kind, payload) {
    Ok(()) => true,
    Err(error) => {
      session.fault(format!("journal write failed: {error}"));
      false
    }
  }
}

/// Bevy plugin that adds the visible desktop controls, HUD, placeholders, and journal hooks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DesktopPresentationPlugin;

impl Plugin for DesktopPresentationPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, desktop_startup);
    app.add_systems(Update, desktop_input.before(PresentationSet::Projection));
    app.add_systems(
      Update,
      desktop_enemy_driver.before(PresentationSet::Projection),
    );
    app.add_systems(Update, (desktop_fault_exit, desktop_observe_close));
    app.add_systems(Last, desktop_finalize);
    app.add_systems(
      Update,
      (
        configure_primary_window,
        desktop_update_animation,
        desktop_play_audio,
        desktop_style_sprites,
        desktop_assets,
        desktop_update_hud,
      )
        .chain()
        .after(PresentationSet::Projection),
    );
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HudLineKind {
  Stats,
  Inventory,
  Messages,
  Controls,
  Journal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
struct HudLine(HudLineKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
struct ShowcaseHud;

#[derive(Resource)]
struct DesktopAssets {
  entries: Vec<DesktopAssetEntry>,
}

struct DesktopAssetEntry {
  family: SceneRenderPlaceholder,
  path: String,
  handle: Option<Handle<Image>>,
  placeholder: Handle<Image>,
  warned: bool,
  outcome_recorded: bool,
}

const ACTOR_PULSE_DURATION: f32 = 0.18;
const ACTOR_PULSE_SCALE: f32 = 0.12;

#[derive(Default, Resource)]
struct DesktopAnimationState {
  previous_cues: Vec<PresentationAnimationCue>,
  previous_token: Option<StateDigest>,
  remaining: f32,
}

impl DesktopAnimationState {
  fn observe(&mut self, token: Option<StateDigest>, cues: &[PresentationAnimationCue]) {
    if self.previous_token == token && self.previous_cues == cues {
      return;
    }
    self.previous_token = token;
    self.previous_cues = cues.to_vec();
    self.remaining = if cues.is_empty() {
      0.0
    } else {
      ACTOR_PULSE_DURATION
    };
  }

  fn advance(&mut self, delta_seconds: f32) {
    self.remaining = (self.remaining - delta_seconds.max(0.0)).max(0.0);
  }

  fn pulse(&self) -> f32 {
    pulse_for_remaining(self.remaining)
  }

  fn update(
    &mut self,
    token: Option<StateDigest>,
    cues: Option<&[PresentationAnimationCue]>,
    delta_seconds: f32,
  ) {
    self.advance(delta_seconds);
    if let Some(cues) = cues {
      self.observe(token, cues);
    }
  }
}

fn pulse_for_remaining(remaining: f32) -> f32 {
  (remaining / ACTOR_PULSE_DURATION).clamp(0.0, 1.0)
}

fn desktop_update_animation(
  time: Res<Time>,
  runtime: Option<Res<PresentationRuntime>>,
  cues: Option<Res<PresentationAnimationCues>>,
  state: Option<ResMut<DesktopAnimationState>>,
) {
  let Some(mut state) = state else { return };
  state.update(
    runtime.as_deref().map(PresentationRuntime::replay_digest),
    cues.as_deref().map(PresentationAnimationCues::cues),
    time.delta_secs(),
  );
}

#[derive(Default, Resource)]
struct DesktopAudioState {
  previous_cues: Vec<PresentationAudioCue>,
  previous_token: Option<StateDigest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
struct DesktopAudioPlayback(PresentationAudioCue);

impl DesktopAudioState {
  fn observe(&mut self, token: Option<StateDigest>, cues: &[PresentationAudioCue]) -> bool {
    if self.previous_token == token && self.previous_cues == cues {
      return false;
    }
    self.previous_token = token;
    self.previous_cues = cues.to_vec();
    !cues.is_empty()
  }
}

fn audio_asset_path(path: &str) -> Option<&str> {
  path.strip_prefix("assets/")
}

fn audio_cue_name(cue: PresentationAudioCue) -> &'static str {
  match cue {
    PresentationAudioCue::Moved { .. } => "moved",
    PresentationAudioCue::MovementBlocked { .. } => "movement_blocked",
    PresentationAudioCue::Waited { .. } => "waited",
    PresentationAudioCue::Attacked { .. } => "attacked",
    PresentationAudioCue::Died { .. } => "died",
    PresentationAudioCue::ItemEquipped { .. } => "item_equipped",
    PresentationAudioCue::ItemUnequipped { .. } => "item_unequipped",
    PresentationAudioCue::ItemConsumed { .. } => "item_consumed",
    PresentationAudioCue::ItemPickedUp { .. } => "item_picked_up",
  }
}

fn desktop_play_audio(
  runtime: Option<Res<PresentationRuntime>>,
  cues: Option<Res<PresentationAudioCues>>,
  manifest: Option<Res<PresentationAudioAssetManifest>>,
  asset_server: Option<Res<AssetServer>>,
  state: Option<ResMut<DesktopAudioState>>,
  mut commands: Commands,
  mut session: Option<ResMut<DesktopSession>>,
) {
  let (Some(runtime), Some(cues), Some(manifest), Some(asset_server), Some(mut state)) =
    (runtime, cues, manifest, asset_server, state)
  else {
    return;
  };
  if !state.observe(Some(runtime.replay_digest()), cues.cues()) {
    return;
  }
  let projection = PresentationAudioAssetProjection::from_cues(cues.cues(), &manifest);
  for entry in projection.entries() {
    let path = entry.reference().path();
    if !Path::new(path).is_file() {
      if let Some(session) = &mut session {
        let _ = record_session(
          &mut *session,
          "audio_outcome",
          json!({ "cue": audio_cue_name(entry.cue()), "path": path, "outcome": "missing_optional_audio" }),
        );
      }
      continue;
    }
    let Some(asset_path) = audio_asset_path(path) else {
      if let Some(session) = &mut session {
        let _ = record_session(
          &mut *session,
          "audio_outcome",
          json!({ "cue": audio_cue_name(entry.cue()), "path": path, "outcome": "unsupported_asset_root" }),
        );
      }
      continue;
    };
    let handle: Handle<AudioSource> = asset_server.load(asset_path.to_string());
    commands.spawn((
      DesktopAudioPlayback(entry.cue()),
      AudioPlayer::new(handle),
      PlaybackSettings::DESPAWN,
    ));
    if let Some(session) = &mut session {
      let _ = record_session(
        &mut *session,
        "audio_outcome",
        json!({ "cue": audio_cue_name(entry.cue()), "path": path, "outcome": "requested" }),
      );
    }
  }
}

fn build_manifest() -> Result<PresentationAssetManifest, String> {
  let paths = [
    (
      SceneRenderPlaceholder::Terrain,
      "assets/dreadstep/terrain.png",
    ),
    (
      SceneRenderPlaceholder::Player,
      "assets/dreadstep/player.png",
    ),
    (SceneRenderPlaceholder::Enemy, "assets/dreadstep/enemy.png"),
    (
      SceneRenderPlaceholder::DeadActor,
      "assets/dreadstep/dead.png",
    ),
    (
      SceneRenderPlaceholder::GroundItem,
      "assets/dreadstep/ground-item.png",
    ),
    (
      SceneRenderPlaceholder::InventoryItem,
      "assets/dreadstep/inventory-item.png",
    ),
  ];
  let mut bindings = Vec::with_capacity(paths.len());
  for (family, path) in paths {
    let reference = PresentationAssetReference::new(path)
      .ok_or_else(|| format!("invalid presentation asset path {path}"))?;
    bindings.push((family, reference));
  }
  PresentationAssetManifest::new(bindings)
    .ok_or_else(|| "asset manifest does not contain all six families".to_string())
}

fn build_audio_manifest() -> Result<PresentationAudioAssetManifest, String> {
  let paths = [
    (
      PresentationAudioCueKind::Moved,
      "assets/audio/dreadstep/moved.ogg",
    ),
    (
      PresentationAudioCueKind::MovementBlocked,
      "assets/audio/dreadstep/movement-blocked.ogg",
    ),
    (
      PresentationAudioCueKind::Waited,
      "assets/audio/dreadstep/waited.ogg",
    ),
    (
      PresentationAudioCueKind::Attacked,
      "assets/audio/dreadstep/attacked.ogg",
    ),
    (
      PresentationAudioCueKind::Died,
      "assets/audio/dreadstep/died.ogg",
    ),
    (
      PresentationAudioCueKind::ItemEquipped,
      "assets/audio/dreadstep/item-equipped.ogg",
    ),
    (
      PresentationAudioCueKind::ItemUnequipped,
      "assets/audio/dreadstep/item-unequipped.ogg",
    ),
    (
      PresentationAudioCueKind::ItemConsumed,
      "assets/audio/dreadstep/item-consumed.ogg",
    ),
  ];
  let mut bindings = Vec::with_capacity(paths.len());
  for (family, path) in paths {
    let reference = PresentationAssetReference::new(path)
      .ok_or_else(|| format!("invalid presentation audio path {path}"))?;
    bindings.push((family, reference));
  }
  PresentationAudioAssetManifest::new(bindings)
    .ok_or_else(|| "audio manifest does not contain all eight cue families".to_string())
}

fn desktop_startup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut session: ResMut<DesktopSession>,
  mut exit: bevy::ecs::message::MessageWriter<AppExit>,
) {
  let manifest = match build_manifest() {
    Ok(manifest) => manifest,
    Err(error) => {
      session.fault(error);
      exit.write(AppExit::error());
      return;
    }
  };
  let audio_manifest = match build_audio_manifest() {
    Ok(manifest) => manifest,
    Err(error) => {
      session.fault(error);
      exit.write(AppExit::error());
      return;
    }
  };
  let mut entries = Vec::with_capacity(manifest.bindings().len());
  for (family, reference) in manifest.bindings() {
    let path = reference.path().to_string();
    let local_path = Path::new(&path);
    let handle = if local_path.is_file() {
      let asset_path = match path.strip_prefix("assets/") {
        Some(asset_path) => asset_path,
        None => path.as_str(),
      };
      let handle = asset_server.load(asset_path.to_string());
      if !record_session(
        &mut session,
        "asset_outcome",
        json!({ "family": placeholder_name(*family), "path": path, "outcome": "requested" }),
      ) {
        exit.write(AppExit::error());
      }
      Some(handle)
    } else {
      if !record_session(
        &mut session,
        "presentation_warning",
        json!({ "family": placeholder_name(*family), "path": path, "warning": "missing_optional_asset" }),
      ) {
        exit.write(AppExit::error());
      }
      None
    };
    entries.push(DesktopAssetEntry {
      family: *family,
      path,
      handle,
      placeholder: crate::placeholder_sprite(*family, None).image,
      warned: false,
      outcome_recorded: false,
    });
  }
  commands.insert_resource(DesktopAssets { entries });
  commands.insert_resource(audio_manifest);

  commands
    .spawn((
      Node {
        position_type: PositionType::Absolute,
        left: px(416.0),
        top: px(8.0),
        width: px(216.0),
        height: px(344.0),
        padding: UiRect::all(px(8.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        overflow: Overflow::clip(),
        ..default()
      },
      BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.94)),
      BorderColor::all(Color::srgb(0.35, 0.45, 0.65)),
      ShowcaseHud,
    ))
    .with_children(|parent| {
      spawn_hud_line(parent, HudLineKind::Stats, "Dreadstep");
      spawn_hud_line(parent, HudLineKind::Inventory, "Inventory");
      spawn_hud_line(parent, HudLineKind::Messages, "Messages");
      spawn_hud_line(parent, HudLineKind::Controls, "Controls");
      spawn_hud_line(parent, HudLineKind::Journal, "Journal");
    });
}

fn spawn_hud_line(
  parent: &mut bevy::ecs::hierarchy::ChildSpawnerCommands,
  line: HudLineKind,
  text: &str,
) {
  parent.spawn((
    Text::new(text),
    TextFont::from_font_size(12.0),
    TextColor(Color::srgb(0.85, 0.9, 1.0)),
    HudLine(line),
  ));
}

fn configure_primary_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
  for mut window in &mut windows {
    window.title = "Dreadstep — Showcase".to_string();
    window.resizable = false;
  }
}

fn desktop_style_sprites(
  tile_size: Option<Res<PresentationTileSize>>,
  animation: Option<Res<DesktopAnimationState>>,
  mut nodes: Query<(
    &SceneRenderNode,
    &mut bevy::sprite::Sprite,
    &mut bevy::camera::visibility::Visibility,
  )>,
) {
  let tile_size = tile_size.map(|value| *value);
  let pulse = animation
    .as_deref()
    .map_or(0.0, DesktopAnimationState::pulse);
  for (node, mut sprite, mut visibility) in &mut nodes {
    let placeholder = node.placeholder();
    let color = match node.key() {
      crate::SceneSpriteKey::Terrain(Tile::Floor) => Color::srgb(0.16, 0.2, 0.24),
      crate::SceneSpriteKey::Terrain(Tile::Cover) => Color::srgb(0.36, 0.25, 0.12),
      crate::SceneSpriteKey::Terrain(Tile::Wall) => Color::srgb(0.04, 0.06, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::Door) => Color::srgb(0.48, 0.25, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::Breakable) => Color::srgb(0.36, 0.22, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::Trap) => Color::srgb(0.58, 0.1, 0.12),
      crate::SceneSpriteKey::Player => Color::srgb(0.1, 0.85, 0.35),
      crate::SceneSpriteKey::Enemy => Color::srgb(0.9, 0.2, 0.22),
      crate::SceneSpriteKey::DeadActor => Color::srgb(0.4, 0.4, 0.45),
      crate::SceneSpriteKey::GroundItem(_) => Color::srgb(0.95, 0.7, 0.16),
      crate::SceneSpriteKey::InventoryItem(_) => Color::srgb(0.22, 0.5, 0.95),
    };
    let scale = sprite_scale(placeholder, node.is_visible(), pulse);
    sprite.color = color;
    sprite.custom_size = tile_size.map(|size| {
      #[allow(clippy::cast_precision_loss)]
      Vec2::new(size.width() as f32 * scale, size.height() as f32 * scale)
    });
    if placeholder == SceneRenderPlaceholder::InventoryItem || !node.is_visible() {
      *visibility = bevy::camera::visibility::Visibility::Hidden;
    } else {
      *visibility = bevy::camera::visibility::Visibility::Inherited;
    }
  }
}

fn sprite_scale(placeholder: SceneRenderPlaceholder, visible: bool, pulse: f32) -> f32 {
  let base = match placeholder {
    SceneRenderPlaceholder::Terrain => 1.0,
    SceneRenderPlaceholder::Player | SceneRenderPlaceholder::Enemy => 0.75,
    SceneRenderPlaceholder::DeadActor => 0.65,
    SceneRenderPlaceholder::GroundItem | SceneRenderPlaceholder::InventoryItem => 0.45,
  };
  if visible
    && matches!(
      placeholder,
      SceneRenderPlaceholder::Player | SceneRenderPlaceholder::Enemy
    )
  {
    base * (1.0 + (ACTOR_PULSE_SCALE * pulse))
  } else {
    base
  }
}

fn desktop_assets(
  asset_server: Option<Res<AssetServer>>,
  assets: Option<ResMut<DesktopAssets>>,
  mut nodes: Query<(&SceneRenderNode, &mut bevy::sprite::Sprite)>,
  mut session: Option<ResMut<DesktopSession>>,
) {
  let Some(asset_server) = asset_server else {
    return;
  };
  let Some(mut assets) = assets else { return };
  for entry in &mut assets.entries {
    let Some(handle) = entry.handle.clone() else {
      continue;
    };
    let load_state = asset_server.get_load_state(&handle);
    if matches!(load_state, Some(LoadState::Failed(_))) && !entry.warned {
      entry.warned = true;
      for (node, mut sprite) in &mut nodes {
        if node.placeholder() == entry.family {
          sprite.image = entry.placeholder.clone();
        }
      }
      if let Some(session) = &mut session {
        let _ = record_session(
          &mut *session,
          "asset_outcome",
          json!({ "family": placeholder_name(entry.family), "path": entry.path, "outcome": "failed_fallback" }),
        );
      }
      continue;
    }
    if matches!(load_state, Some(LoadState::Loaded)) && !entry.outcome_recorded {
      entry.outcome_recorded = true;
      if let Some(session) = &mut session {
        let _ = record_session(
          &mut *session,
          "asset_outcome",
          json!({ "family": placeholder_name(entry.family), "path": entry.path, "outcome": "loaded" }),
        );
      }
    }
    for (node, mut sprite) in &mut nodes {
      if node.placeholder() == entry.family {
        sprite.image = handle.clone();
      }
    }
  }
}

fn desktop_fault_exit(
  session: Option<ResMut<DesktopSession>>,
  runtime: Option<Res<PresentationRuntime>>,
  mut exit: bevy::ecs::message::MessageWriter<AppExit>,
) {
  let Some(mut session) = session else { return };
  let DesktopStatus::Faulted(error) = session.status.clone() else {
    return;
  };
  if !session.terminal_recorded {
    session.terminal_recorded = true;
    let payload = runtime.map_or_else(
      || json!({ "error": error }),
      |runtime| state_payload(&runtime, json!({ "error": error })),
    );
    let _ = record_session(&mut session, "terminal_fault", payload);
  }
  exit.write(AppExit::error());
}

/// Flushes final journal/replay evidence while the Bevy world still owns the runtime.
///
/// [`App::run`](bevy::app::App::run) consumes the app and replaces the caller's world with an
/// empty one, so shutdown work cannot safely inspect resources after the runner returns. This
/// system observes the exit message before that handoff and reports only the final error through a
/// small external handle.
fn desktop_finalize(
  mut exits: MessageReader<AppExit>,
  runtime: Option<Res<PresentationRuntime>>,
  session: Option<ResMut<DesktopSession>>,
  handle: Option<Res<FinalizationHandle>>,
) {
  if exits.read().next().is_none() {
    return;
  }
  let Some(handle) = handle else { return };
  {
    let report = match handle.0.lock() {
      Ok(report) => report,
      Err(poisoned) => poisoned.into_inner(),
    };
    if report.complete {
      return;
    }
  }
  let Some(runtime) = runtime else {
    handle.finish(Some(
      "desktop runtime resource missing before finalization".to_string(),
    ));
    return;
  };
  let Some(mut session) = session else {
    handle.finish(Some(
      "desktop session resource missing before finalization".to_string(),
    ));
    return;
  };
  let mut error = None;
  if let Err(export_error) = export_replay(&runtime, &session.journal) {
    let _ = record_session(
      &mut session,
      "terminal_fault",
      state_payload(&runtime, json!({ "error": export_error })),
    );
    error = Some(export_error);
  }
  let status = session.status.clone();
  let reason = if error.is_some() || matches!(status, DesktopStatus::Faulted(_)) {
    "runtime_fault".to_string()
  } else {
    match &status {
      DesktopStatus::Shutdown(reason) => reason.clone(),
      DesktopStatus::Victory => "showcase_complete".to_string(),
      DesktopStatus::Defeat => "showcase_defeat".to_string(),
      DesktopStatus::Running => "window_closed_or_ctrl_c".to_string(),
      DesktopStatus::Faulted(_) => "runtime_fault".to_string(),
    }
  };
  if let Err(journal_error) = record(&session.journal, "shutdown", json!({ "reason": reason })) {
    error.get_or_insert_with(|| journal_error.to_string());
  }
  if error.is_none()
    && let DesktopStatus::Faulted(status_error) = status
  {
    error = Some(status_error);
  }
  handle.finish(error);
}

fn desktop_observe_close(
  closing_windows: Query<Entity, With<ClosingWindow>>,
  mut session: ResMut<DesktopSession>,
) {
  if closing_windows.is_empty()
    || !matches!(
      session.status,
      DesktopStatus::Running | DesktopStatus::Defeat
    )
  {
    return;
  }
  session.status = DesktopStatus::Shutdown("window_close".to_string());
}

#[allow(clippy::too_many_lines)]
fn desktop_input(
  keys: Res<ButtonInput<KeyCode>>,
  mut runtime: ResMut<PresentationRuntime>,
  mut session: ResMut<DesktopSession>,
  mut exit: bevy::ecs::message::MessageWriter<AppExit>,
) {
  if !matches!(
    session.status,
    DesktopStatus::Running | DesktopStatus::Victory | DesktopStatus::Defeat
  ) {
    return;
  }
  if keys.just_pressed(KeyCode::Escape) {
    let _ = record_session(
      &mut session,
      "input_request",
      json!({ "key": "Escape", "action": "shutdown" }),
    );
    if matches!(session.status, DesktopStatus::Faulted(_)) {
      exit.write(AppExit::error());
      return;
    }
    session.status = DesktopStatus::Shutdown("escape".to_string());
    exit.write(AppExit::Success);
    return;
  }
  if restart_requested(&keys) {
    let _ = record_session(
      &mut session,
      "input_request",
      json!({ "key": "Shift+R", "action": "restart" }),
    );
    if matches!(session.status, DesktopStatus::Faulted(_)) {
      exit.write(AppExit::error());
      return;
    }
    let restarted = if session.procedural {
      PresentationRuntime::start_procedural_run(session.seed, session.depth)
    } else {
      PresentationRuntime::start_item_run(session.seed)
    };
    match restarted {
      Ok(restarted) => {
        let payload = state_payload(&restarted, json!({ "seed": session.seed }));
        *runtime = restarted;
        session.status = DesktopStatus::Running;
        session.messages.clear();
        session.selected_item = Some(EQUIP_ITEM);
        session.enemy_timer.reset();
        session.command_kinds.clear();
        session.event_kinds.clear();
        session.terminal_recorded = false;
        let _ = record_session(&mut session, "run_restarted", payload);
      }
      Err(error) => session.fault(format!("restart failed: {error}")),
    }
    return;
  }
  if keys.just_pressed(KeyCode::KeyN) {
    if matches!(session.status, DesktopStatus::Victory) && session.procedural {
      let _ = record_session(
        &mut session,
        "input_request",
        json!({ "key": "KeyN", "action": "next_floor" }),
      );
      let _ = advance_procedural_floor(&mut runtime, &mut session);
    }
    return;
  }
  if keys.just_pressed(KeyCode::Tab) {
    let reverse = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    select_inventory_item(&runtime, &mut session, reverse);
    let selected_item = session.selected_item.map(ItemId::value);
    let _ = record_session(
      &mut session,
      "input_request",
      json!({ "input": "select_inventory", "selected_item": selected_item }),
    );
    return;
  }
  if !matches!(session.status, DesktopStatus::Running) {
    return;
  }

  let key = [
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyE,
    KeyCode::KeyQ,
    KeyCode::KeyU,
    KeyCode::KeyP,
    KeyCode::KeyX,
    KeyCode::KeyR,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::KeyW,
    KeyCode::KeyS,
    KeyCode::KeyA,
    KeyCode::KeyD,
    KeyCode::Enter,
    KeyCode::Space,
  ]
  .into_iter()
  .find(|key| keys.just_pressed(*key));
  let Some(key) = key else { return };
  let _ = record_session(
    &mut session,
    "input_request",
    json!({ "key": format!("{key:?}"), "actor": PLAYER.value() }),
  );
  if matches!(session.status, DesktopStatus::Faulted(_)) {
    return;
  }
  if runtime.snapshot().next_actor() != Some(PLAYER) {
    session.push_message(format!("Unavailable input: {key:?} (enemy scheduled)."));
    let snapshot = state_payload(
      &runtime,
      json!({ "key": format!("{key:?}"), "reason": "actor_not_scheduled" }),
    );
    let _ = record_session(&mut session, "action_rejected", snapshot);
    return;
  }
  let command = command_for_key(key, &runtime, &session);
  let Some(command) = command else {
    session.push_message(format!("Unavailable input: {key:?}"));
    let snapshot = state_payload(
      &runtime,
      json!({ "key": format!("{key:?}"), "reason": "unavailable" }),
    );
    let _ = record_session(&mut session, "action_rejected", snapshot);
    return;
  };
  let _ = submit_command(&mut runtime, &mut session, "player", command);
}

fn restart_requested(keys: &ButtonInput<KeyCode>) -> bool {
  keys.just_pressed(KeyCode::KeyR)
    && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
}

fn advance_procedural_floor(
  runtime: &mut PresentationRuntime,
  session: &mut DesktopSession,
) -> bool {
  if !session.procedural || !matches!(session.status, DesktopStatus::Victory) {
    return false;
  }
  let Some(next_depth) = session.depth.checked_add(1) else {
    session.fault("procedural floor depth overflow");
    return false;
  };
  let next_runtime = match PresentationRuntime::start_procedural_run(session.seed, next_depth) {
    Ok(runtime) => runtime,
    Err(error) => {
      session.fault(format!("procedural floor advance failed: {error}"));
      return false;
    }
  };
  let payload = state_payload(
    &next_runtime,
    json!({
      "seed": session.seed,
      "scenario": "procedural_floor",
      "depth": next_depth,
    }),
  );
  *runtime = next_runtime;
  session.depth = next_depth;
  session.status = DesktopStatus::Running;
  session.messages.clear();
  session.selected_item = None;
  session.enemy_timer.reset();
  session.command_kinds.clear();
  session.event_kinds.clear();
  session.terminal_recorded = false;
  record_session(session, "floor_advanced", payload)
}

fn select_inventory_item(
  runtime: &PresentationRuntime,
  session: &mut DesktopSession,
  reverse: bool,
) {
  let snapshot = runtime.snapshot();
  let Some(actor) = snapshot.actors().iter().find(|actor| actor.id() == PLAYER) else {
    session.selected_item = None;
    return;
  };
  let items = actor.inventory();
  if items.is_empty() {
    session.selected_item = None;
    return;
  }
  let current = session
    .selected_item
    .and_then(|selected| items.iter().position(|item| item.id() == selected));
  let index = match (current, reverse) {
    (Some(index), false) => (index + 1) % items.len(),
    (Some(index), true) => (index + items.len() - 1) % items.len(),
    (None, _) => 0,
  };
  session.selected_item = items.get(index).map(|item| item.id());
}

fn command_for_key(
  key: KeyCode,
  runtime: &PresentationRuntime,
  session: &DesktopSession,
) -> Option<Command> {
  let legal = runtime.legal_commands();
  let candidate = match key {
    KeyCode::KeyF => legal
      .iter()
      .filter_map(|command| match command {
        Command::Attack { target, .. } => Some((*target, *command)),
        _ => None,
      })
      .min_by_key(|(target, _)| *target)
      .map(|(_, command)| command),
    KeyCode::KeyG => legal
      .iter()
      .filter_map(|command| match command {
        Command::RangedAttack { target, .. } => Some((*target, *command)),
        _ => None,
      })
      .min_by_key(|(target, _)| *target)
      .map(|(_, command)| command),
    KeyCode::KeyE => session.selected_item.map(|item| Command::Equip {
      actor: PLAYER,
      item,
    }),
    KeyCode::KeyQ => Some(Command::Unequip { actor: PLAYER }),
    KeyCode::KeyU => session.selected_item.map(|item| Command::UseItem {
      actor: PLAYER,
      item,
    }),
    KeyCode::KeyP => legal
      .iter()
      .filter_map(|command| match command {
        Command::Pickup { item, .. } => Some((*item, *command)),
        _ => None,
      })
      .min_by_key(|(item, _)| *item)
      .map(|(_, command)| command),
    KeyCode::KeyX => session.selected_item.and_then(|item| {
      legal.iter().copied().find(|command| {
        matches!(
          command,
          Command::Drop {
            actor: PLAYER,
            item: candidate,
          } if *candidate == item
        )
      })
    }),
    KeyCode::KeyR => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Reload { actor: PLAYER })),
    KeyCode::KeyI => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Interact { actor: PLAYER, .. })),
    KeyCode::KeyK => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Kick { actor: PLAYER, .. })),
    KeyCode::KeyB => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Break { actor: PLAYER, .. })),
    other => crate::KeyboardIntent::from_key(other).map(|intent| intent.command(PLAYER)),
  }?;
  legal.into_iter().find(|command| *command == candidate)
}

fn desktop_enemy_driver(
  time: Res<Time>,
  mut runtime: ResMut<PresentationRuntime>,
  mut session: ResMut<DesktopSession>,
  input: Res<PresentationInput>,
) {
  if !matches!(session.status, DesktopStatus::Running) {
    return;
  }
  if runtime.snapshot().next_actor() == Some(PLAYER) {
    session.enemy_timer.reset();
    return;
  }
  session.enemy_timer.tick(time.delta());
  if !session.enemy_timer.is_finished() {
    return;
  }
  let actor = runtime.snapshot().next_actor();
  let Some(actor) = actor else {
    return;
  };
  let legal = runtime.legal_commands();
  let command = crate::select_enemy_command(&legal, actor, input.actor());
  if let Some(command) = command {
    let _ = submit_command(&mut runtime, &mut session, "enemy_driver", command);
  } else {
    session.fault(format!(
      "no legal enemy command for actor {}",
      actor.value()
    ));
  }
  session.enemy_timer.reset();
}

fn submit_command(
  runtime: &mut PresentationRuntime,
  session: &mut DesktopSession,
  source: &str,
  command: Command,
) -> bool {
  if !matches!(session.status, DesktopStatus::Running) {
    return false;
  }
  let before = state_payload(
    runtime,
    json!({ "source": source, "command": command_value(command) }),
  );
  if !record_session(session, "command_requested", before.clone()) {
    return false;
  }
  session
    .command_kinds
    .insert(command_name(command).to_string());
  match runtime.execute(command) {
    Ok(output) => {
      for event in output.events() {
        session
          .event_kinds
          .insert(crate::showcase_event_name(*event).to_string());
        session.push_message(event_message(*event));
      }
      let payload = state_payload(
        runtime,
        json!({
          "source": source,
          "command": command_value(command),
          "events": output.events().iter().copied().map(event_value).collect::<Vec<_>>(),
        }),
      );
      if !record_session(session, "action_accepted", payload) {
        return false;
      }
      match runtime.snapshot().outcome() {
        RunOutcome::Defeat => {
          session.status = DesktopStatus::Defeat;
          session.push_message("Showcase failed — the player is dead.");
          let _ = record_session(
            session,
            "terminal_defeat",
            state_payload(runtime, json!({ "reason": "player_died" })),
          );
        }
        RunOutcome::Victory => {
          session.status = DesktopStatus::Victory;
          session.push_message("Showcase complete — every enemy is dead.");
          let _ = record_session(
            session,
            "terminal_victory",
            state_payload(runtime, json!({ "reason": "all_enemies_dead" })),
          );
        }
        RunOutcome::InProgress => {}
      }
      session.enemy_timer.reset();
      true
    }
    Err(error) => {
      session.push_message(format!("Rejected: {error}"));
      let payload = state_payload(
        runtime,
        json!({
          "source": source,
          "command": command_value(command),
          "error": error.to_string(),
          "unchanged": true,
          "before": before,
        }),
      );
      if !record_session(session, "action_rejected", payload) {
        return false;
      }
      false
    }
  }
}

fn run_visible(
  runtime: PresentationRuntime,
  seed: u64,
  procedural: bool,
  depth: u32,
  journal: JournalHandle,
) -> Result<(), String> {
  let tile_size = crate::PresentationTileSize::new(32, 32)
    .ok_or_else(|| "invalid 32x32 tile size".to_string())?;
  let viewport =
    crate::PresentationViewport::new(7, 5).ok_or_else(|| "invalid 7x5 viewport".to_string())?;
  let window = crate::PresentationWindow::new(640, 360, 2)
    .ok_or_else(|| "invalid 640x360 window".to_string())?;
  let manifest = build_manifest()?;
  let finalization = FinalizationHandle::new();
  let mut app = App::new();
  app.add_plugins(
    DefaultPlugins
      .build()
      .disable::<PanicHandlerPlugin>()
      .set(ImagePlugin::default_nearest())
      .set(WindowPlugin {
        primary_window: Some(Window {
          resolution: bevy::window::WindowResolution::new(
            window.physical_width(),
            window.physical_height(),
          )
          .with_scale_factor_override({
            #[allow(clippy::cast_precision_loss)]
            {
              window.pixel_scale() as f32
            }
          }),
          resizable: false,
          title: "Dreadstep — Showcase".to_string(),
          ..default()
        }),
        ..default()
      }),
  );
  app.insert_resource(runtime);
  app.insert_resource(finalization.clone());
  app.insert_resource(DesktopSession::new_with_scenario(
    seed,
    procedural,
    depth,
    journal.clone(),
  ));
  app.insert_resource(PresentationInput::new(PLAYER));
  app.insert_resource(PresentationKeyboardMode::External);
  app.insert_resource(PresentationFocus::new(PLAYER));
  app.insert_resource(PresentationCamera::new(PLAYER));
  app.insert_resource(PresentationVisibility::new(PLAYER, 3));
  app.insert_resource(viewport);
  app.insert_resource(crate::PresentationHud::new(PLAYER));
  app.insert_resource(PresentationEnemyIntent::new());
  app.insert_resource(PresentationMessages::new());
  app.insert_resource(PresentationAnimationCues::new());
  app.insert_resource(DesktopAnimationState::default());
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(PresentationAudioAssetProjection::new());
  app.insert_resource(DesktopAudioState::default());
  app.insert_resource(tile_size);
  app.insert_resource(window);
  app.insert_resource(PresentationRenderProjection::default());
  app.insert_resource(PresentationSpriteProjection::new());
  app.insert_resource(PresentationRenderCommandPlan::new());
  app.insert_resource(PresentationRenderNodeProjection::new());
  app.insert_resource(PresentationBevySpriteProjection::new());
  app.insert_resource(PresentationBevySpriteTransformProjection::new());
  app.insert_resource(PresentationRenderAssetProjection::new());
  app.insert_resource(manifest);
  app.add_plugins((PresentationPlugin, DesktopPresentationPlugin));
  let _exit = app.run();
  let report = match finalization.0.lock() {
    Ok(report) => report,
    Err(poisoned) => poisoned.into_inner(),
  };
  if let Some(error) = &report.error {
    Err(error.clone())
  } else if report.complete {
    Ok(())
  } else {
    Err("desktop exited before finalization".to_string())
  }
}

#[allow(clippy::too_many_lines)]
fn run_smoke(mut runtime: PresentationRuntime, journal: JournalHandle) -> ExitCode {
  let mut session = DesktopSession::new(runtime.seed(), journal.clone());
  let mut failed = false;
  if let Err(error) = runtime.prepare_smoke_trap(Position::new(4, 1)) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "trap_fixture_setup", "error": error.to_string() }),
    );
  }
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::RangedAttack {
      actor: PLAYER,
      target: RANGED_TARGET,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  if let Err(error) = runtime.prepare_smoke_breakable(Position::new(2, 1)) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "breakable_fixture_setup", "error": error.to_string() }),
    );
  }
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Break {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  if let Err(error) = runtime.prepare_smoke_door(Position::new(2, 1)) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "door_fixture_setup", "error": error.to_string() }),
    );
  }
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Kick {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  if let Err(error) = runtime.prepare_smoke_door(Position::new(2, 1)) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "interact_door_fixture_setup", "error": error.to_string() }),
    );
  }
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Interact {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Reload { actor: PLAYER },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  if let Err(error) = runtime.prepare_smoke_pickup(PLAYER, PICKUP_ITEM) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "pickup_fixture_setup", "error": error.to_string() }),
    );
  }
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Pickup {
      actor: PLAYER,
      item: PICKUP_ITEM,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Drop {
      actor: PLAYER,
      item: PICKUP_ITEM,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::East,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Equip {
      actor: PLAYER,
      item: EQUIP_ITEM,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Unequip { actor: PLAYER },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);

  let mut attacks = 0;
  while runtime
    .snapshot()
    .actors()
    .iter()
    .find(|actor| actor.id() == ATTACK_TARGET)
    .is_some_and(Actor::is_alive)
    && attacks < SMOKE_ENEMY_ATTACK_LIMIT
  {
    let command = runtime.legal_commands().into_iter().find(|command| {
      matches!(
        command,
        Command::Attack {
          actor: PLAYER,
          target: ATTACK_TARGET
        }
      )
    });
    let Some(command) = command else {
      let _ = submit_command(
        &mut runtime,
        &mut session,
        "smoke",
        Command::Wait { actor: PLAYER },
      );
      failed |= !drive_smoke_enemies(&mut runtime, &mut session);
      attacks = attacks.saturating_add(1);
      continue;
    };
    failed |= !submit_command(&mut runtime, &mut session, "smoke", command);
    failed |= !drive_smoke_enemies(&mut runtime, &mut session);
    attacks = attacks.saturating_add(1);
  }
  if runtime
    .snapshot()
    .actors()
    .iter()
    .find(|actor| actor.id() == ATTACK_TARGET)
    .is_some_and(Actor::is_alive)
  {
    failed = true;
    if !record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "attack_target_not_defeated", "attempts": attacks }),
    ) {
      failed = true;
    }
  }

  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::UseItem {
      actor: PLAYER,
      item: EQUIP_ITEM,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::North,
    },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);
  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Wait { actor: PLAYER },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);

  let command_coverage = SHOWCASE_COMMAND_KINDS
    .iter()
    .all(|kind| session.command_kinds.contains(*kind));
  let event_coverage = SHOWCASE_EVENT_KINDS
    .iter()
    .all(|kind| session.event_kinds.contains(*kind));
  if !command_coverage || !event_coverage {
    failed = true;
    let commands_observed = session.command_kinds.iter().cloned().collect::<Vec<_>>();
    let events_observed = session.event_kinds.iter().cloned().collect::<Vec<_>>();
    if !record_session(
      &mut session,
      "smoke_coverage_fault",
      json!({
        "commands_observed": commands_observed,
        "events_observed": events_observed,
        "commands_expected": SHOWCASE_COMMAND_KINDS,
        "events_expected": SHOWCASE_EVENT_KINDS,
      }),
    ) {
      failed = true;
    }
  }
  let commands_observed = session.command_kinds.iter().cloned().collect::<Vec<_>>();
  let events_observed = session.event_kinds.iter().cloned().collect::<Vec<_>>();
  let journal_name = journal_path(&journal).display().to_string();
  let terminal_payload = state_payload(
    &runtime,
    json!({
      "commands_observed": commands_observed,
      "events_observed": events_observed,
      "journal": journal_name,
    }),
  );
  if !record_session(
    &mut session,
    if failed {
      "terminal_fault"
    } else {
      "smoke_complete"
    },
    terminal_payload,
  ) {
    failed = true;
  }
  if export_replay(&runtime, &journal).is_err() {
    failed = true;
    let _ = record_session(
      &mut session,
      "replay_export_fault",
      json!({ "reason": "replay_export_failed" }),
    );
  }
  if !record_session(
    &mut session,
    "shutdown",
    json!({ "reason": if failed { "smoke_fault" } else { "smoke_complete" } }),
  ) {
    failed = true;
  }
  if failed {
    ExitCode::from(1)
  } else {
    ExitCode::SUCCESS
  }
}

fn drive_smoke_enemies(runtime: &mut PresentationRuntime, session: &mut DesktopSession) -> bool {
  for _ in 0..64 {
    if runtime.snapshot().next_actor() == Some(PLAYER) {
      return true;
    }
    let Some(actor) = runtime.snapshot().next_actor() else {
      return true;
    };
    let legal = runtime.legal_commands();
    let command = crate::select_enemy_command(&legal, actor, PLAYER).and_then(|command| {
      let player_is_low = runtime
        .snapshot()
        .actors()
        .iter()
        .find(|record| record.id() == PLAYER)
        .is_some_and(|record| record.hit_points().value() <= 3);
      if player_is_low && matches!(command, Command::Attack { .. }) {
        legal
          .iter()
          .copied()
          .find(|candidate| matches!(candidate, Command::Wait { actor: candidate_actor } if *candidate_actor == actor))
      } else {
        Some(command)
      }
    });
    let Some(command) = command else {
      session.fault(format!(
        "smoke enemy actor {} has no legal command",
        actor.value()
      ));
      return false;
    };
    if !submit_command(runtime, session, "enemy_driver", command) {
      return false;
    }
  }
  session.fault("smoke enemy driver exceeded 64 actions");
  false
}

fn state_payload(runtime: &PresentationRuntime, extra: Value) -> Value {
  let snapshot = runtime.snapshot();
  json!({
    "state": snapshot_value(&snapshot),
    "state_digest": snapshot.digest().value(),
    "replay_digest": runtime.replay_digest().value(),
    "extra": extra,
  })
}

fn snapshot_value(snapshot: &crate::PresentationSnapshot) -> Value {
  let actors = snapshot
    .actors()
    .iter()
    .map(actor_value)
    .collect::<Vec<_>>();
  let ground_items = snapshot
    .ground_items()
    .iter()
    .map(|stack| {
      json!({
        "position": position_value(stack.position()),
        "items": stack.items().iter().copied().map(item_value).collect::<Vec<_>>(),
      })
    })
    .collect::<Vec<_>>();
  json!({
    "map": {
      "width": snapshot.width(),
      "height": snapshot.height(),
      "tiles": snapshot.tiles().iter().copied().map(tile_name).collect::<Vec<_>>(),
    },
    "outcome": outcome_name(snapshot.outcome()),
    "actors": actors,
    "ground_items": ground_items,
    "scheduler": {
      "current_time": snapshot.current_time().value(),
      "next_actor": snapshot.next_actor().map(ActorId::value),
    },
  })
}

fn actor_value(actor: &Actor) -> Value {
  json!({
    "id": actor.id().value(),
    "kind": actor_kind_name(actor.kind()),
    "position": position_value(actor.position()),
    "hit_points": actor.hit_points().value(),
    "melee_reach": actor.melee_reach().value(),
    "ranged_ammo": actor.ranged_ammo(),
    "alive": actor.is_alive(),
    "ready_at": actor.ready_at().value(),
    "equipped": actor.equipped_item().map(ItemId::value),
    "inventory": actor.inventory().iter().copied().map(item_value).collect::<Vec<_>>(),
  })
}

fn item_value(item: Item) -> Value {
  json!({ "id": item.id().value(), "definition": item.definition().value() })
}

fn position_value(position: Position) -> Value {
  json!({ "x": position.x(), "y": position.y() })
}

fn tile_name(tile: Tile) -> &'static str {
  match tile {
    Tile::Floor => "floor",
    Tile::Cover => "cover",
    Tile::Wall => "wall",
    Tile::Door => "door",
    Tile::Breakable => "breakable",
    Tile::Trap => "trap",
  }
}

fn actor_kind_name(kind: ActorKind) -> &'static str {
  match kind {
    ActorKind::Player => "player",
    ActorKind::Enemy => "enemy",
  }
}

fn outcome_name(outcome: RunOutcome) -> &'static str {
  match outcome {
    RunOutcome::InProgress => "in_progress",
    RunOutcome::Defeat => "defeat",
    RunOutcome::Victory => "victory",
  }
}

fn placeholder_name(placeholder: SceneRenderPlaceholder) -> &'static str {
  match placeholder {
    SceneRenderPlaceholder::Terrain => "terrain",
    SceneRenderPlaceholder::Player => "player",
    SceneRenderPlaceholder::Enemy => "enemy",
    SceneRenderPlaceholder::DeadActor => "dead",
    SceneRenderPlaceholder::GroundItem => "ground_item",
    SceneRenderPlaceholder::InventoryItem => "inventory_item",
  }
}

fn command_name(command: Command) -> &'static str {
  match command {
    Command::Move { .. } => "move",
    Command::Wait { .. } => "wait",
    Command::Interact { .. } => "interact",
    Command::Break { .. } => "break",
    Command::Kick { .. } => "kick",
    Command::Attack { .. } => "attack",
    Command::RangedAttack { .. } => "ranged_attack",
    Command::Chase { .. } => "chase",
    Command::Equip { .. } => "equip",
    Command::Unequip { .. } => "unequip",
    Command::UseItem { .. } => "use_item",
    Command::Pickup { .. } => "pickup",
    Command::Drop { .. } => "drop",
    Command::Reload { .. } => "reload",
  }
}

fn command_value(command: Command) -> Value {
  match command {
    Command::Move { actor, direction } => {
      json!({ "kind": "move", "actor": actor.value(), "direction": direction_name(direction) })
    }
    Command::Wait { actor } => json!({ "kind": "wait", "actor": actor.value() }),
    Command::Interact { actor, position } => json!({
      "kind": "interact",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Break { actor, position } => json!({
      "kind": "break",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Kick { actor, position } => json!({
      "kind": "kick",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Attack { actor, target } => {
      json!({ "kind": "attack", "actor": actor.value(), "target": target.value() })
    }
    Command::RangedAttack { actor, target } => {
      json!({ "kind": "ranged_attack", "actor": actor.value(), "target": target.value() })
    }
    Command::Chase { actor, target } => {
      json!({ "kind": "chase", "actor": actor.value(), "target": target.value() })
    }
    Command::Equip { actor, item } => {
      json!({ "kind": "equip", "actor": actor.value(), "item": item.value() })
    }
    Command::Unequip { actor } => json!({ "kind": "unequip", "actor": actor.value() }),
    Command::UseItem { actor, item } => {
      json!({ "kind": "use_item", "actor": actor.value(), "item": item.value() })
    }
    Command::Pickup { actor, item } => {
      json!({ "kind": "pickup", "actor": actor.value(), "item": item.value() })
    }
    Command::Drop { actor, item } => {
      json!({ "kind": "drop", "actor": actor.value(), "item": item.value() })
    }
    Command::Reload { actor } => json!({ "kind": "reload", "actor": actor.value() }),
  }
}

fn direction_name(direction: Direction) -> &'static str {
  match direction {
    Direction::North => "north",
    Direction::South => "south",
    Direction::West => "west",
    Direction::East => "east",
  }
}

fn event_value(event: Event) -> Value {
  match event {
    Event::Moved { actor, from, to } => {
      json!({ "kind": "moved", "actor": actor.value(), "from": position_value(from), "to": position_value(to) })
    }
    Event::MovementBlocked {
      actor,
      from,
      to,
      reason,
    } => json!({
      "kind": "movement_blocked",
      "actor": actor.value(),
      "from": position_value(from),
      "to": position_value(to),
      "reason": block_reason_value(reason),
    }),
    Event::Waited { actor, at } => {
      json!({ "kind": "waited", "actor": actor.value(), "at": at.value() })
    }
    Event::DoorOpened { actor, position } => json!({
      "kind": "door_opened",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Event::BreakableBroken { actor, position } => json!({
      "kind": "breakable_broken",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Event::NoiseCreated {
      actor,
      position,
      radius,
    } => json!({
      "kind": "noise_created",
      "actor": actor.value(),
      "position": position_value(position),
      "radius": radius,
    }),
    Event::TrapTriggered {
      actor,
      position,
      damage,
      remaining_hit_points,
    } => json!({
      "kind": "trap_triggered",
      "actor": actor.value(),
      "position": position_value(position),
      "damage": damage.value(),
      "remaining_hit_points": remaining_hit_points.value(),
    }),
    Event::Attacked {
      attacker,
      target,
      damage,
      remaining_hit_points,
    } => json!({
      "kind": "attacked",
      "attacker": attacker.value(),
      "target": target.value(),
      "damage": damage.value(),
      "remaining_hit_points": remaining_hit_points.value(),
    }),
    Event::Died { actor } => json!({ "kind": "died", "actor": actor.value() }),
    Event::ItemEquipped { actor, item } => {
      json!({ "kind": "item_equipped", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemUnequipped { actor, item } => {
      json!({ "kind": "item_unequipped", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemConsumed {
      actor,
      item,
      healing,
      ammunition,
    } => {
      let healing = healing.map_or(Value::Null, |result| {
        json!({
          "amount": result.amount(),
          "remaining_hit_points": result.remaining_hit_points().value(),
        })
      });
      let ammunition = ammunition.map_or(Value::Null, |result| {
        json!({
          "amount": result.amount(),
          "remaining_ammunition": result.remaining_ammunition(),
        })
      });
      json!({ "kind": "item_consumed", "actor": actor.value(), "item": item.value(), "healing": healing, "ammunition": ammunition })
    }
    Event::ItemPickedUp { actor, item } => {
      json!({ "kind": "item_picked_up", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemDropped { actor, item } => {
      json!({ "kind": "item_dropped", "actor": actor.value(), "item": item.value() })
    }
    Event::Reloaded { actor, ammunition } => {
      json!({ "kind": "reloaded", "actor": actor.value(), "ammunition": ammunition })
    }
  }
}

fn block_reason_value(reason: BlockReason) -> Value {
  match reason {
    BlockReason::Terrain => json!({ "kind": "terrain" }),
    BlockReason::Actor(actor) => json!({ "kind": "actor", "actor": actor.value() }),
  }
}

fn event_message(event: Event) -> String {
  match event {
    Event::Moved { actor, to, .. } => {
      format!("Actor {} moved to ({}, {}).", actor.value(), to.x(), to.y())
    }
    Event::MovementBlocked { actor, reason, .. } => {
      format!("Actor {} blocked by {:?}.", actor.value(), reason)
    }
    Event::Waited { actor, at } => format!("Actor {} waited at t{}.", actor.value(), at.value()),
    Event::DoorOpened { actor, position } => format!(
      "Actor {} opened the door at ({}, {}).",
      actor.value(),
      position.x(),
      position.y()
    ),
    Event::BreakableBroken { actor, position } => format!(
      "Actor {} broke terrain at ({}, {}).",
      actor.value(),
      position.x(),
      position.y()
    ),
    Event::NoiseCreated {
      actor,
      position,
      radius,
    } => format!(
      "Actor {} created noise at ({}, {}) with radius {}.",
      actor.value(),
      position.x(),
      position.y(),
      radius
    ),
    Event::TrapTriggered {
      actor,
      position,
      damage,
      remaining_hit_points,
    } => format!(
      "Actor {} triggered a trap at ({}, {}) for {} damage ({} HP left).",
      actor.value(),
      position.x(),
      position.y(),
      damage.value(),
      remaining_hit_points.value()
    ),
    Event::Attacked {
      attacker,
      target,
      remaining_hit_points,
      ..
    } => format!(
      "Actor {} hit {} ({} HP left).",
      attacker.value(),
      target.value(),
      remaining_hit_points.value()
    ),
    Event::Died { actor } => format!("Actor {} died.", actor.value()),
    Event::ItemEquipped { actor, item } => {
      format!("Actor {} equipped item {}.", actor.value(), item.value())
    }
    Event::ItemUnequipped { actor, item } => {
      format!("Actor {} unequipped item {}.", actor.value(), item.value())
    }
    Event::ItemConsumed {
      actor,
      item,
      healing,
      ammunition,
    } => {
      if let Some(ammunition) = ammunition {
        format!(
          "Actor {} consumed item {} and restored {} ammunition ({} shots).",
          actor.value(),
          item.value(),
          ammunition.amount(),
          ammunition.remaining_ammunition()
        )
      } else if let Some(healing) = healing {
        format!(
          "Actor {} consumed item {} and restored {} HP ({} HP).",
          actor.value(),
          item.value(),
          healing.amount(),
          healing.remaining_hit_points().value()
        )
      } else {
        format!("Actor {} consumed item {}.", actor.value(), item.value())
      }
    }
    Event::ItemPickedUp { actor, item } => {
      format!("Actor {} picked up item {}.", actor.value(), item.value())
    }
    Event::ItemDropped { actor, item } => {
      format!("Actor {} dropped item {}.", actor.value(), item.value())
    }
    Event::Reloaded { actor, ammunition } => {
      format!("Actor {} reloaded to {} shots.", actor.value(), ammunition)
    }
  }
}

fn health_bar_text(hit_points: i32) -> String {
  let clamped = usize::try_from(hit_points.clamp(0, SHOWCASE_MAX_HIT_POINTS)).unwrap_or_default();
  let maximum = usize::try_from(SHOWCASE_MAX_HIT_POINTS).unwrap_or_default();
  let filled = ((clamped * HEALTH_BAR_WIDTH) + (maximum / 2)) / maximum;
  format!(
    "[{}{}]",
    "#".repeat(filled),
    "-".repeat(HEALTH_BAR_WIDTH - filled)
  )
}

fn visibility_summary_values(active: bool, radius: u32, visible_tiles: usize) -> String {
  if active {
    format!("FOV {visible_tiles} tiles (radius {radius})")
  } else {
    "FOV full map".to_string()
  }
}

fn visibility_summary(visibility: Option<&PresentationVisibility>) -> String {
  visibility.map_or_else(
    || visibility_summary_values(false, 0, 0),
    |visibility| {
      visibility_summary_values(
        visibility.is_active(),
        visibility.radius(),
        visibility.visible_positions().len(),
      )
    },
  )
}

fn enemy_intent_summary(intent: Option<&PresentationEnemyIntent>) -> String {
  let Some(intent) = intent else {
    return "Intent unavailable".to_string();
  };
  match (intent.actor(), intent.command()) {
    (Some(actor), Some(Command::Chase { target, .. })) => {
      format!(
        "Intent: enemy {} chases actor {}",
        actor.value(),
        target.value()
      )
    }
    (Some(actor), Some(command)) => format!("Intent: enemy {} {:?}", actor.value(), command),
    _ => "Intent: none".to_string(),
  }
}

fn scenario_label(procedural: bool, depth: u32) -> String {
  if procedural {
    format!("Procedural floor · depth {depth}")
  } else {
    "Starter item floor".to_string()
  }
}

fn format_hud_stats(
  player: Option<&Actor>,
  snapshot: &PresentationSnapshot,
  status: &DesktopStatus,
  scenario: &str,
  visibility: Option<&PresentationVisibility>,
  intent: Option<&PresentationEnemyIntent>,
) -> String {
  let enemies_remaining = snapshot
    .actors()
    .iter()
    .filter(|actor| actor.kind() == ActorKind::Enemy && actor.is_alive())
    .count();
  let Some(player) = player else {
    return format!(
      "{}\nPlayer unavailable\nTurn t={} next={}\nEnemies remaining: {}\n{}\n{}\nStatus: {:?}",
      scenario,
      snapshot.current_time().value(),
      snapshot
        .next_actor()
        .map_or_else(|| "-".to_string(), |id| id.value().to_string()),
      enemies_remaining,
      visibility_summary(visibility),
      enemy_intent_summary(intent),
      status
    );
  };
  let hit_points = i32::from(player.hit_points().value());
  format!(
    "{}\nHP {} {}/{}  pos ({},{})\nTurn t={} next={}  enemies {}\n{}\n{}\nStatus: {:?}",
    scenario,
    health_bar_text(hit_points),
    hit_points.clamp(0, SHOWCASE_MAX_HIT_POINTS),
    SHOWCASE_MAX_HIT_POINTS,
    player.position().x(),
    player.position().y(),
    snapshot.current_time().value(),
    snapshot
      .next_actor()
      .map_or_else(|| "-".to_string(), |id| id.value().to_string()),
    enemies_remaining,
    visibility_summary(visibility),
    enemy_intent_summary(intent),
    status
  )
}

fn desktop_update_hud(
  runtime: Option<Res<PresentationRuntime>>,
  session: Option<Res<DesktopSession>>,
  visibility: Option<Res<PresentationVisibility>>,
  intent: Option<Res<PresentationEnemyIntent>>,
  mut lines: Query<(&mut Text, &HudLine), With<HudLine>>,
) {
  let Some(runtime) = runtime else { return };
  let Some(session) = session else { return };
  let snapshot = runtime.snapshot();
  let player = snapshot.actors().iter().find(|actor| actor.id() == PLAYER);
  let stats = format_hud_stats(
    player,
    &snapshot,
    &session.status,
    &scenario_label(session.procedural, session.depth),
    visibility.as_deref(),
    intent.as_deref(),
  );
  let inventory = player.map_or_else(
    || "Inventory unavailable".to_string(),
    |player| {
      let items = player
        .inventory()
        .iter()
        .map(|item| {
          let selected = session.selected_item == Some(item.id());
          let equipped = player.equipped_item() == Some(item.id());
          format!(
            "{}item {} (def {}){}",
            if selected { "> " } else { "  " },
            item.id().value(),
            item.definition().value(),
            if equipped { " [equipped]" } else { "" }
          )
        })
        .collect::<Vec<_>>();
      if items.is_empty() {
        "(empty)".to_string()
      } else {
        items.join("\n")
      }
    },
  );
  let messages = if session.messages.is_empty() {
    "(no events yet)".to_string()
  } else {
    session
      .messages
      .iter()
      .cloned()
      .collect::<Vec<_>>()
      .join("\n")
  };
  let controls = "Arrows/WASD move  Space/Enter wait\nF attack  G ranged  Tab select  E equip  P pickup  X drop\nQ unequip  U consume  R reload  Shift+R restart  N next procedural floor after victory\nEsc/close quit";
  let journal = format!(
    "{}\nseed {}",
    journal_path(&session.journal).display(),
    session.seed
  );
  for (mut text, line) in &mut lines {
    let value = match line.0 {
      HudLineKind::Stats => &stats,
      HudLineKind::Inventory => &inventory,
      HudLineKind::Messages => &messages,
      HudLineKind::Controls => controls,
      HudLineKind::Journal => &journal,
    };
    *text = Text::new(value);
  }
}

/// The exhaustive formatter is public for integration tests and future coverage checks.
#[must_use]
pub fn event_kind(event: Event) -> &'static str {
  crate::showcase_event_name(event)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::PresentationState;
  use bevy::app::TaskPoolPlugin;
  use bevy::asset::{AssetApp, AssetPlugin};
  use bevy::audio::PlaybackMode;
  use bevy::camera::visibility::Visibility;
  use bevy::ecs::message::MessageWriter;
  use bevy::transform::components::Transform;
  use dreadstep_core::{GridMap, WorldState};

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
    let expected = PresentationRuntime::start_procedural_run(7, 3)
      .expect("next procedural floor should validate");
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
    let mut overflow_session =
      DesktopSession::new_with_scenario(7, true, u32::MAX, journal.clone());
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
    let expected = PresentationRuntime::start_procedural_run(7, 3)
      .expect("next procedural floor should validate");
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
      None,
      Some(&empty_intent),
    );
    assert!(text.contains("Player unavailable"));
    assert!(text.contains("Enemies remaining: 3"));
    assert!(text.contains("FOV full map"));
    assert!(text.contains("Intent: none"));

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
}
