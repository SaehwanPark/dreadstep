//! HUD, enemy-intent, message, audio, and animation synchronization.

use bevy::ecs::world::World;
use dreadstep_core::{ActorId, ActorKind, Command};

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
  } else {
    hud.kind = None;
    hud.position = None;
    hud.hit_points = None;
    hud.ready_at = None;
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
  let command = scheduled_enemy.and_then(|actor| {
    let legal = runtime.legal_commands();
    select_enemy_command(&legal, actor, target)
  });
  let Some(mut intent) = world.get_resource_mut::<PresentationEnemyIntent>() else {
    return;
  };
  intent.actor = scheduled_enemy;
  intent.command = command;
}

/// Selects the shared deterministic enemy-driver preference from core's legal commands.
///
/// Adjacent melee is preferred over clear ranged attacks; one-use noise investigation then chase
/// and wait preserve the deterministic movement fallback. The first command for the actor remains
/// a forward-compatible final fallback.
pub(crate) fn select_enemy_command(
  legal: &[Command],
  actor: ActorId,
  target: ActorId,
) -> Option<Command> {
  legal
    .iter()
    .find(|command| {
      matches!(
        command,
        Command::Attack {
          actor: candidate,
          target: candidate_target,
        } if *candidate == actor && *candidate_target == target
      )
    })
    .copied()
    .or_else(|| {
      legal
        .iter()
        .find(|command| {
          matches!(
            command,
            Command::RangedAttack {
              actor: candidate,
              target: candidate_target,
            } if *candidate == actor && *candidate_target == target
          )
        })
        .copied()
    })
    .or_else(|| {
      legal
        .iter()
        .find(|command| {
          matches!(
            command,
            Command::Investigate {
              actor: candidate,
              ..
            } if *candidate == actor
          )
        })
        .copied()
    })
    .or_else(|| {
      legal
        .iter()
        .find(|command| {
          matches!(
            command,
            Command::Chase {
              actor: candidate,
              target: candidate_target,
            } if *candidate == actor && *candidate_target == target
          )
        })
        .copied()
    })
    .or_else(|| {
      legal
        .iter()
        .find(
          |command| matches!(command, Command::Wait { actor: candidate } if *candidate == actor),
        )
        .copied()
    })
    .or_else(|| {
      legal
        .iter()
        .copied()
        .find(|command| command_actor(*command) == actor)
    })
}

pub(crate) fn command_actor(command: Command) -> ActorId {
  match command {
    Command::Move { actor, .. }
    | Command::Wait { actor }
    | Command::Interact { actor, .. }
    | Command::Kick { actor, .. }
    | Command::Break { actor, .. }
    | Command::Attack { actor, .. }
    | Command::RangedAttack { actor, .. }
    | Command::Chase { actor, .. }
    | Command::Investigate { actor, .. }
    | Command::Equip { actor, .. }
    | Command::Unequip { actor }
    | Command::UseItem { actor, .. }
    | Command::Pickup { actor, .. }
    | Command::Drop { actor, .. }
    | Command::Reload { actor } => actor,
  }
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
