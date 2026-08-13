//! HUD, enemy-intent, message, audio, and animation synchronization.

use bevy::ecs::world::World;
use dreadstep_core::ActorKind;

use crate::{
  PresentationAnimationCue, PresentationAnimationCues, PresentationAudioAssetManifest,
  PresentationAudioAssetProjection, PresentationAudioCue, PresentationAudioCues,
  PresentationEnemyIntent, PresentationHud, PresentationInput, PresentationMessage,
  PresentationMessages, PresentationRuntime,
};

pub(crate) fn sync_hud(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let Some(mut hud) = world.get_resource_mut::<PresentationHud>() else {
    return;
  };
  hud.actor = actor;
  if let Some(record) = snapshot.actors().iter().find(|record| record.id() == actor) {
    hud.kind = Some(record.kind());
    hud.position = Some(record.position());
    hud.hit_points = Some(record.hit_points());
    hud.ready_at = Some(record.ready_at());
    hud.status = record.status();
  } else {
    hud.kind = None;
    hud.position = None;
    hud.hit_points = None;
    hud.ready_at = None;
    hud.status = None;
  }
}

pub(crate) fn sync_enemy_intent(world: &mut World) {
  if world.get_resource::<PresentationEnemyIntent>().is_none() {
    return;
  }
  let Some(runtime) = world.get_resource::<PresentationRuntime>() else {
    if let Some(mut intent) = world.get_resource_mut::<PresentationEnemyIntent>() {
      intent.actor = None;
      intent.command = None;
    }
    return;
  };
  let Some(target) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    if let Some(mut intent) = world.get_resource_mut::<PresentationEnemyIntent>() {
      intent.actor = None;
      intent.command = None;
    }
    return;
  };
  let snapshot = runtime.snapshot();
  let scheduled_enemy = snapshot.next_actor().filter(|actor| {
    snapshot
      .actors()
      .iter()
      .any(|record| record.id() == *actor && record.kind() == ActorKind::Enemy && record.is_alive())
  });
  let command = scheduled_enemy.and_then(|actor| runtime.preferred_enemy_command(actor, target));
  let Some(mut intent) = world.get_resource_mut::<PresentationEnemyIntent>() else {
    return;
  };
  intent.actor = scheduled_enemy;
  intent.command = command;
}

pub(crate) fn sync_messages(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .map(PresentationMessage::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut messages) = world.get_resource_mut::<PresentationMessages>() else {
    return;
  };
  messages.messages = projected;
}

pub(crate) fn sync_audio_cues(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .filter_map(PresentationAudioCue::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut cues) = world.get_resource_mut::<PresentationAudioCues>() else {
    return;
  };
  cues.cues = projected;
}

pub(crate) fn sync_audio_asset_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world.get_resource::<PresentationAudioCues>().is_none()
    || world
      .get_resource::<PresentationAudioAssetManifest>()
      .is_none()
  {
    return;
  }
  let cues = world.resource::<PresentationAudioCues>().cues().to_vec();
  let manifest = world.resource::<PresentationAudioAssetManifest>();
  let projection = PresentationAudioAssetProjection::from_cues(&cues, manifest);
  let Some(mut destination) = world.get_resource_mut::<PresentationAudioAssetProjection>() else {
    return;
  };
  destination.entries = projection.entries;
}

pub(crate) fn sync_animation_cues(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .filter_map(PresentationAnimationCue::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut cues) = world.get_resource_mut::<PresentationAnimationCues>() else {
    return;
  };
  cues.cues = projected;
}
