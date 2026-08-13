//! Deterministic Bevy presentation boundary for Dreadstep.
//!
//! [`PresentationState`] owns a core world and translates human-client intent into the same
//! semantic commands used by headless and agent adapters. It deliberately exposes only immutable
//! projections and core outcomes; rendering, ECS storage, windowing, and presentation effects
//! remain outside authoritative game state.

#![forbid(unsafe_code)]

pub use dreadstep_core::RunOutcome;

#[cfg(feature = "desktop")]
pub mod desktop;

mod animation;
mod audio;
mod camera;
mod focus;
mod hud;
mod input;
mod messages;
mod plugin;
mod render;
mod runtime;
mod scene;
mod snapshot;
mod sync;
mod sync_feedback;
mod sync_render;
mod sync_view;
mod viewport;
mod visibility;
mod window;

pub use animation::{PresentationAnimationCue, PresentationAnimationCues};
pub use audio::{
  PresentationAudioAssetEntry, PresentationAudioAssetManifest, PresentationAudioAssetProjection,
  PresentationAudioCue, PresentationAudioCueKind, PresentationAudioCues,
};
pub use camera::PresentationCamera;
pub use focus::PresentationFocus;
pub use hud::{PresentationEnemyIntent, PresentationHud};
pub use input::{KeyboardIntent, PresentationInput, PresentationKeyboardMode, PresentationSet};
pub use messages::{PresentationMessage, PresentationMessages, showcase_event_name};
pub use plugin::PresentationPlugin;
pub use render::{
  PresentationAssetManifest, PresentationAssetReference, PresentationBevySpriteProjection,
  PresentationBevySpriteTransformProjection, PresentationRenderAssetProjection,
  PresentationRenderCommandPlan, PresentationRenderNodeProjection, PresentationRenderProjection,
  PresentationSpriteProjection, SceneBevySpriteEntry, SceneBevySpriteTransformEntry,
  ScenePixelPosition, SceneRenderAssetEntry, SceneRenderCommand, SceneRenderEntry,
  SceneRenderLayer, SceneRenderNode, SceneRenderNodeEntry, SceneRenderPlaceholder,
  SceneSpriteEntry, SceneSpriteKey, SceneSpriteRole,
};
pub use runtime::{PresentationRuntime, PresentationState};
pub use scene::{
  SceneActor, SceneCamera, SceneFocus, SceneGroundItem, SceneInventoryItem, SceneTile,
  SceneViewport, SceneWindow,
};
pub use snapshot::{PresentationOutput, PresentationSnapshot};
pub use sync::sync_scene;
pub use viewport::PresentationViewport;
pub use visibility::PresentationVisibility;
pub use window::{PresentationTileSize, PresentationWindow};

pub(crate) use scene::{SceneCameraState, SceneViewportState, SceneWindowState};
pub(crate) use sync::sync_runtime_scene;
pub(crate) use sync_feedback::{
  sync_animation_cues, sync_audio_asset_projection, sync_audio_cues, sync_enemy_intent, sync_hud,
  sync_messages,
};
pub(crate) use sync_render::{
  placeholder_sprite, sync_bevy_sprite_projection, sync_bevy_sprite_transform_projection,
  sync_render_asset_projection, sync_render_command_plan, sync_render_nodes,
  sync_render_projection, sync_scene_pixel_positions, sync_sprite_node_components,
  sync_sprite_projection, sync_sprite_transform_components,
};
pub(crate) use sync_view::{
  sync_camera, sync_focus, sync_scene_camera, sync_scene_camera_components, sync_scene_focus,
  sync_scene_viewport, sync_scene_window_components, sync_viewport, sync_visibility,
};
