//! Keyboard command selection, enemy driving, and command submission.

use bevy::app::AppExit;
use bevy::ecs::system::{Res, ResMut};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::time::Time;
use dreadstep_core::{Command, ItemId, RunOutcome};
use serde_json::json;

use crate::{PresentationInput, PresentationRuntime};

use super::format::{command_name, command_value, event_message, event_value, state_payload};
use super::session::{DesktopSession, DesktopStatus, record_session};
use super::{EQUIP_ITEM, PLAYER};

#[expect(
  clippy::too_many_lines,
  reason = "desktop key dispatch stays one exhaustive player command map"
)]
pub(crate) fn desktop_input(
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
    KeyCode::KeyT,
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

pub(crate) fn restart_requested(keys: &ButtonInput<KeyCode>) -> bool {
  keys.just_pressed(KeyCode::KeyR)
    && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
}

pub(crate) fn advance_procedural_floor(
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

pub(crate) fn select_inventory_item(
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

pub(crate) fn command_for_key(
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
    KeyCode::KeyT => session.selected_item.and_then(|item| {
      legal
        .iter()
        .filter_map(|command| match command {
          Command::Throw {
            item: candidate,
            target,
            ..
          } if *candidate == item => Some((*target, *command)),
          _ => None,
        })
        .min_by_key(|(target, _)| *target)
        .map(|(_, command)| command)
    }),
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

pub(crate) fn desktop_enemy_driver(
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

pub(crate) fn submit_command(
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
