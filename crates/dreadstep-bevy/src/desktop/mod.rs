//! Runnable desktop showcase and diagnostic journal.
//!
//! This module is a process/presentation boundary. It owns CLI parsing, the OS window, optional
//! local art, human input, and JSONL diagnostics. Simulation stays in [`crate::PresentationRuntime`].

#![allow(clippy::needless_continue, clippy::needless_pass_by_value)]

use std::ffi::OsString;
use std::panic::{self, AssertUnwindSafe};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::app::{PanicHandlerPlugin, PluginGroup};
use bevy::prelude::{App, DefaultPlugins, ImagePlugin, WindowPlugin, default};
use bevy::window::Window;
use dreadstep_content::ContentError;
use dreadstep_core::{ActorId, ItemId};

use serde_json::json;

use crate::{
  PresentationAnimationCues, PresentationAudioAssetProjection, PresentationAudioCues,
  PresentationBevySpriteProjection, PresentationBevySpriteTransformProjection, PresentationCamera,
  PresentationEnemyIntent, PresentationFocus, PresentationInput, PresentationKeyboardMode,
  PresentationMessages, PresentationPlugin, PresentationRenderAssetProjection,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationRuntime, PresentationSpriteProjection, PresentationVisibility,
};

mod behavior;
mod cli;
pub(crate) mod format;
pub(crate) mod input;
pub(crate) mod journal;
mod plugin;
pub(crate) mod session;
pub(crate) mod smoke;

#[cfg(test)]
mod tests;

pub use cli::DesktopOptions;
pub use format::event_kind;
pub use plugin::DesktopPresentationPlugin;

use cli::{ParseResult, USAGE, parse_options};
use format::state_payload;
use journal::{Journal, JournalHandle, panic_message, record};
use plugin::{DesktopAnimationState, DesktopAudioState, build_manifest};
use session::{DesktopSession, FinalizationHandle};
use smoke::run_smoke;

pub(crate) const PLAYER: ActorId = ActorId::new(1);
pub(crate) const ATTACK_TARGET: ActorId = ActorId::new(2);
pub(crate) const RANGED_TARGET: ActorId = ActorId::new(3);
pub(crate) const EQUIP_ITEM: ItemId = ItemId::new(103);
pub(crate) const FROST_FLASK: ItemId = ItemId::new(104);
pub(crate) const CONSUME_ITEM: ItemId = ItemId::new(101);
pub(crate) const PICKUP_ITEM: ItemId = ItemId::new(102);
pub(crate) const SMOKE_ENEMY_ATTACK_LIMIT: usize = 32;
pub(crate) const ENEMY_DELAY: Duration = Duration::from_millis(150);
pub(crate) const SHOWCASE_MAX_HIT_POINTS: i32 = 10;
pub(crate) const HEALTH_BAR_WIDTH: usize = 10;
pub(crate) const REPLAY_EXPORT_SCHEMA_VERSION: u16 = 1;

/// Every current command kind that must remain demonstrable by the desktop smoke path.
pub const SHOWCASE_COMMAND_KINDS: [&str; 19] = [
  "move",
  "wait",
  "interact",
  "kick",
  "close",
  "break",
  "attack",
  "ranged_attack",
  "cast_chill",
  "throw",
  "retreat",
  "chase",
  "investigate",
  "equip",
  "unequip",
  "use_item",
  "pickup",
  "drop",
  "reload",
];

/// Every current event kind that must remain observable in the desktop smoke path.
pub const SHOWCASE_EVENT_KINDS: [&str; 20] = [
  "moved",
  "movement_blocked",
  "waited",
  "door_opened",
  "door_closed",
  "noise_created",
  "breakable_broken",
  "trap_triggered",
  "status_applied",
  "status_expired",
  "attacked",
  "chill_cast",
  "item_thrown",
  "died",
  "item_equipped",
  "item_unequipped",
  "item_consumed",
  "item_picked_up",
  "item_dropped",
  "reloaded",
];

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
pub(crate) fn run_with_panic_boundary(options: DesktopOptions) -> ExitCode {
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

pub(crate) fn run_with_journal(options: DesktopOptions, journal: JournalHandle) -> ExitCode {
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

pub(crate) fn start_runtime(options: &DesktopOptions) -> Result<PresentationRuntime, ContentError> {
  if options.procedural && !options.smoke {
    PresentationRuntime::start_procedural_run(options.seed, options.depth)
  } else {
    PresentationRuntime::start_item_run(options.seed)
  }
}
pub(crate) fn run_visible(
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
