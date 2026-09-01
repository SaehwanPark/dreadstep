//! Visible-window plugin: assets, HUD spawn, animation pulse, and audio playback.

use std::path::Path;

use bevy::app::{App, AppExit, Last, Plugin, Startup, Update};
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
use bevy::math::Vec2;
use bevy::prelude::{
  BackgroundColor, BorderColor, Node, PositionType, Text, TextColor, TextFont, default,
};
use bevy::time::Time;
use bevy::ui::{FlexDirection, JustifyContent, Overflow, UiRect, px};
use bevy::window::{ClosingWindow, PrimaryWindow, Window};
use dreadstep_core::{StateDigest, Tile};
use serde_json::json;

use crate::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAssetManifest,
  PresentationAssetReference, PresentationAudioAssetManifest, PresentationAudioAssetProjection,
  PresentationAudioCue, PresentationAudioCueKind, PresentationAudioCues, PresentationRuntime,
  PresentationSet, PresentationTileSize, SceneRenderNode, SceneRenderPlaceholder,
};

use super::format::{HudLine, HudLineKind, desktop_update_hud, placeholder_name, state_payload};
use super::input::{desktop_enemy_driver, desktop_input};
use super::journal::{export_replay, record};
use super::session::{DesktopSession, DesktopStatus, FinalizationHandle, record_session};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(crate) struct ShowcaseHud;

#[derive(Resource)]
pub(crate) struct DesktopAssets {
  entries: Vec<DesktopAssetEntry>,
}

pub(crate) struct DesktopAssetEntry {
  family: SceneRenderPlaceholder,
  path: String,
  handle: Option<Handle<Image>>,
  placeholder: Handle<Image>,
  warned: bool,
  outcome_recorded: bool,
}

pub(crate) const ACTOR_PULSE_DURATION: f32 = 0.18;
pub(crate) const ACTOR_PULSE_SCALE: f32 = 0.12;

#[derive(Default, Resource)]
pub(crate) struct DesktopAnimationState {
  previous_cues: Vec<PresentationAnimationCue>,
  previous_token: Option<StateDigest>,
  remaining: f32,
}

impl DesktopAnimationState {
  pub(crate) fn observe(&mut self, token: Option<StateDigest>, cues: &[PresentationAnimationCue]) {
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

  pub(crate) fn advance(&mut self, delta_seconds: f32) {
    self.remaining = (self.remaining - delta_seconds.max(0.0)).max(0.0);
  }

  pub(crate) fn pulse(&self) -> f32 {
    pulse_for_remaining(self.remaining)
  }

  pub(crate) fn update(
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

pub(crate) fn pulse_for_remaining(remaining: f32) -> f32 {
  (remaining / ACTOR_PULSE_DURATION).clamp(0.0, 1.0)
}

pub(crate) fn desktop_update_animation(
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
pub(crate) struct DesktopAudioState {
  previous_cues: Vec<PresentationAudioCue>,
  previous_token: Option<StateDigest>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(crate) struct DesktopAudioPlayback(pub(crate) PresentationAudioCue);

impl DesktopAudioState {
  pub(crate) fn observe(
    &mut self,
    token: Option<StateDigest>,
    cues: &[PresentationAudioCue],
  ) -> bool {
    if self.previous_token == token && self.previous_cues == cues {
      return false;
    }
    self.previous_token = token;
    self.previous_cues = cues.to_vec();
    !cues.is_empty()
  }
}

pub(crate) fn audio_asset_path(path: &str) -> Option<&str> {
  path.strip_prefix("assets/")
}

pub(crate) fn audio_cue_name(cue: PresentationAudioCue) -> &'static str {
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

pub(crate) fn desktop_play_audio(
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

pub(crate) fn build_manifest() -> Result<PresentationAssetManifest, String> {
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

pub(crate) fn build_audio_manifest() -> Result<PresentationAudioAssetManifest, String> {
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

pub(crate) fn desktop_startup(
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

pub(crate) fn spawn_hud_line(
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

pub(crate) fn configure_primary_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
  for mut window in &mut windows {
    window.title = "Dreadstep — Showcase".to_string();
    window.resizable = false;
  }
}

pub(crate) fn desktop_style_sprites(
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
      crate::SceneSpriteKey::Terrain(Tile::Stairs) => Color::srgb(0.85, 0.78, 0.24),
      crate::SceneSpriteKey::Terrain(Tile::Cover) => Color::srgb(0.36, 0.25, 0.12),
      crate::SceneSpriteKey::Terrain(Tile::Wall) => Color::srgb(0.04, 0.06, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::Door) => Color::srgb(0.48, 0.25, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::OpenDoor) => Color::srgb(0.65, 0.4, 0.14),
      crate::SceneSpriteKey::Terrain(Tile::Breakable) => Color::srgb(0.36, 0.22, 0.08),
      crate::SceneSpriteKey::Terrain(Tile::Trap) => Color::srgb(0.58, 0.1, 0.12),
      crate::SceneSpriteKey::Terrain(Tile::ChillTrap) => Color::srgb(0.18, 0.65, 0.9),
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

pub(crate) fn sprite_scale(placeholder: SceneRenderPlaceholder, visible: bool, pulse: f32) -> f32 {
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

pub(crate) fn desktop_assets(
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

pub(crate) fn desktop_fault_exit(
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
pub(crate) fn desktop_finalize(
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

pub(crate) fn desktop_observe_close(
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
