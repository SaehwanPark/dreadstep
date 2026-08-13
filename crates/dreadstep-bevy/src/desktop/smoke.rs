//! Display-free deterministic smoke runner.

use std::process::ExitCode;

use dreadstep_core::{Actor, Command, Direction, Position};
use serde_json::json;

use crate::PresentationRuntime;

use super::format::state_payload;
use super::input::submit_command;
use super::journal::{JournalHandle, export_replay, journal_path};
use super::session::{DesktopSession, record_session};
use super::{
  ATTACK_TARGET, CONSUME_ITEM, EQUIP_ITEM, PICKUP_ITEM, PLAYER, RANGED_TARGET,
  SHOWCASE_COMMAND_KINDS, SHOWCASE_EVENT_KINDS, SMOKE_ENEMY_ATTACK_LIMIT,
};

#[expect(
  clippy::too_many_lines,
  reason = "display-free smoke keeps the exhaustive command/event matrix in one runner"
)]
pub(crate) fn run_smoke(mut runtime: PresentationRuntime, journal: JournalHandle) -> ExitCode {
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
  if let Err(error) = runtime.prepare_smoke_teleport(ATTACK_TARGET, Position::new(4, 1)) {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "reach_attack_fixture_setup", "error": error.to_string() }),
    );
  }
  let reach_fixture_valid = runtime
    .snapshot()
    .actors()
    .iter()
    .find(|actor| actor.id() == PLAYER)
    .zip(
      runtime
        .snapshot()
        .actors()
        .iter()
        .find(|actor| actor.id() == ATTACK_TARGET),
    )
    .is_some_and(|(player, target)| {
      player.melee_reach().value() >= 2
        && (player.position().x() - target.position().x()).unsigned_abs()
          + (player.position().y() - target.position().y()).unsigned_abs()
          == 2
    });
  if !reach_fixture_valid {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "reach_attack_fixture_invalid" }),
    );
  }

  let mut attacks = 0;
  let mut extended_attack_observed = false;
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
      if attacks == 0 {
        failed = true;
        let _ = record_session(
          &mut session,
          "smoke_fault",
          json!({ "reason": "reach_attack_not_legal" }),
        );
      }
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
    if attacks == 0 {
      extended_attack_observed = true;
    }
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

  if !extended_attack_observed {
    failed = true;
    let _ = record_session(
      &mut session,
      "smoke_fault",
      json!({ "reason": "reach_attack_not_observed" }),
    );
  }

  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::Unequip { actor: PLAYER },
  );
  failed |= !drive_smoke_enemies(&mut runtime, &mut session);

  failed |= !submit_command(
    &mut runtime,
    &mut session,
    "smoke",
    Command::UseItem {
      actor: PLAYER,
      item: CONSUME_ITEM,
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

pub(crate) fn drive_smoke_enemies(
  runtime: &mut PresentationRuntime,
  session: &mut DesktopSession,
) -> bool {
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
      if player_is_low && matches!(command, Command::Attack { .. } | Command::RangedAttack { .. }) {
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
