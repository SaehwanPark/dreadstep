//! Headless presentation plugin orchestration.

use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::world::World;
use bevy::input::{ButtonInput, keyboard::KeyCode};

use crate::{
  KeyboardIntent, PresentationInput, PresentationKeyboardMode, PresentationRuntime,
  PresentationSet, sync_animation_cues, sync_audio_asset_projection, sync_audio_cues,
  sync_bevy_sprite_projection, sync_bevy_sprite_transform_projection, sync_camera,
  sync_enemy_intent, sync_focus, sync_hud, sync_messages, sync_render_asset_projection,
  sync_render_command_plan, sync_render_nodes, sync_render_projection, sync_runtime_scene,
  sync_scene_camera, sync_scene_camera_components, sync_scene_focus, sync_scene_pixel_positions,
  sync_scene_viewport, sync_scene_window_components, sync_sprite_node_components,
  sync_sprite_projection, sync_sprite_transform_components, sync_viewport, sync_visibility,
};

/// A headless Bevy plugin that keeps disposable scene mirrors synchronized with runtime state.
///
/// Runtime-backed scene projections are a safe no-op until [`PresentationRuntime`] is inserted by
/// the application, so app construction can install plugins before selecting or restoring a run.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(
      Update,
      update_presentation.in_set(PresentationSet::Projection),
    );
  }
}

const KEY_PRIORITY: [KeyCode; 10] = [
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
];

pub(crate) fn update_presentation(world: &mut World) {
  dispatch_keyboard_input(world);
  sync_runtime_scene(world);
  sync_visibility(world);
  sync_scene_pixel_positions(world);
  sync_render_projection(world);
  sync_sprite_projection(world);
  sync_render_command_plan(world);
  sync_render_nodes(world);
  sync_bevy_sprite_projection(world);
  sync_bevy_sprite_transform_projection(world);
  sync_sprite_node_components(world);
  sync_sprite_transform_components(world);
  sync_render_asset_projection(world);
  sync_focus(world);
  sync_scene_focus(world);
  sync_camera(world);
  sync_scene_camera(world);
  sync_scene_camera_components(world);
  sync_scene_window_components(world);
  sync_viewport(world);
  sync_scene_viewport(world);
  sync_hud(world);
  sync_enemy_intent(world);
  sync_messages(world);
  sync_audio_cues(world);
  sync_audio_asset_projection(world);
  sync_animation_cues(world);
}

pub(crate) fn dispatch_keyboard_input(world: &mut World) {
  if world
    .get_resource::<PresentationKeyboardMode>()
    .is_some_and(|mode| *mode == PresentationKeyboardMode::External)
  {
    return;
  }
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(key) = world
    .get_resource::<ButtonInput<KeyCode>>()
    .and_then(|input| {
      KEY_PRIORITY
        .iter()
        .copied()
        .find(|key| input.just_pressed(*key))
    })
  else {
    return;
  };
  let Some(intent) = KeyboardIntent::from_key(key) else {
    return;
  };
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  {
    let Some(mut input) = world.get_resource_mut::<ButtonInput<KeyCode>>() else {
      return;
    };
    for supported_key in KEY_PRIORITY {
      input.clear_just_pressed(supported_key);
    }
  }
  let _ = world
    .resource_mut::<PresentationRuntime>()
    .execute(intent.command(actor));
}
