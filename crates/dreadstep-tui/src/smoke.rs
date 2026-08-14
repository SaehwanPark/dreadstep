//! Display-free exhaustive command/event coverage gate.

use std::process::ExitCode;

use dreadstep_core::{Actor, Command, Direction, EnemyBehavior, Position, Tile};
use serde_json::json;

use crate::kinds::{SHOWCASE_COMMAND_KINDS, SHOWCASE_EVENT_KINDS};
use crate::play::Play;
use crate::session::{
  ATTACK_TARGET, CONSUME_ITEM, EQUIP_ITEM, FROST_FLASK, PICKUP_ITEM, PLAYER, RANGED_TARGET,
};

const SMOKE_ENEMY_ATTACK_LIMIT: usize = 32;

/// Runs the display-free TUI smoke sequence.
#[expect(
  clippy::too_many_lines,
  reason = "display-free smoke keeps the exhaustive command/event matrix in one runner"
)]
#[must_use]
pub fn run_smoke(mut play: Play) -> ExitCode {
  let mut failed = false;
  failed |= !play.prepare_tile(Position::new(4, 1), Tile::Trap);
  failed |= !play.prepare_tile(Position::new(1, 2), Tile::ChillTrap);
  if play
    .session
    .prepare_smoke_behavior(dreadstep_core::ActorId::new(4), EnemyBehavior::Blocker)
    .and_then(|()| {
      play
        .session
        .prepare_smoke_teleport(dreadstep_core::ActorId::new(4), Position::new(3, 3))
    })
    .is_err()
  {
    failed = true;
    play.fault("blocker_fixture_setup");
  }
  failed |= !play.submit_command(
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::South,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::North,
    },
  );
  let kiter = dreadstep_core::ActorId::new(3);
  if play
    .session
    .prepare_smoke_behavior(kiter, EnemyBehavior::Kiter)
    .is_err()
  {
    failed = true;
    play.fault("kiter_fixture_setup");
  }
  failed |= !play.prepare_tile(Position::new(2, 1), Tile::Floor);
  if play
    .session
    .prepare_smoke_teleport(kiter, Position::new(2, 1))
    .is_err()
  {
    failed = true;
    play.fault("kiter_target_fixture_setup");
  }
  failed |= !play.drive_enemies(true);
  if play
    .session
    .prepare_smoke_behavior(kiter, EnemyBehavior::Frostcaster)
    .is_err()
  {
    failed = true;
    play.fault("frostcaster_fixture_setup");
  }
  failed |= !play.submit_command(
    "smoke",
    Command::RangedAttack {
      actor: PLAYER,
      target: RANGED_TARGET,
    },
  );
  failed |= !play.drive_enemies(true);
  if play
    .session
    .prepare_smoke_teleport(RANGED_TARGET, Position::new(3, 1))
    .is_err()
  {
    failed = true;
    play.fault("throw_target_fixture_setup");
  }
  failed |= !play.submit_command(
    "smoke",
    Command::Throw {
      actor: PLAYER,
      item: FROST_FLASK,
      target: RANGED_TARGET,
    },
  );
  failed |= !play.drive_enemies(true);
  if play
    .session
    .prepare_smoke_behavior(dreadstep_core::ActorId::new(4), EnemyBehavior::Brute)
    .and_then(|()| {
      play
        .session
        .prepare_smoke_teleport(dreadstep_core::ActorId::new(4), Position::new(5, 3))
    })
    .is_err()
  {
    failed = true;
    play.fault("brute_fixture_setup");
  }
  failed |= !play.prepare_tile(Position::new(2, 1), Tile::Breakable);
  failed |= !play.submit_command(
    "smoke",
    Command::Break {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.prepare_tile(Position::new(2, 1), Tile::Door);
  failed |= !play.submit_command(
    "smoke",
    Command::Kick {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.prepare_tile(Position::new(2, 1), Tile::Door);
  failed |= !play.submit_command(
    "smoke",
    Command::Interact {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Close {
      actor: PLAYER,
      position: Position::new(2, 1),
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command("smoke", Command::Reload { actor: PLAYER });
  failed |= !play.drive_enemies(true);
  if play
    .session
    .prepare_smoke_pickup(PLAYER, PICKUP_ITEM)
    .is_err()
  {
    failed = true;
    play.fault("pickup_fixture_setup");
  }
  failed |= !play.submit_command(
    "smoke",
    Command::Pickup {
      actor: PLAYER,
      item: PICKUP_ITEM,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Drop {
      actor: PLAYER,
      item: PICKUP_ITEM,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::East,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Equip {
      actor: PLAYER,
      item: EQUIP_ITEM,
    },
  );
  failed |= !play.drive_enemies(true);
  let player_position = play
    .session
    .actor(PLAYER)
    .map_or(Position::new(1, 1), Actor::position);
  let reach_target_position =
    Position::new(player_position.x().saturating_add(2), player_position.y());
  if play
    .session
    .prepare_smoke_teleport(RANGED_TARGET, Position::new(5, 2))
    .is_err()
  {
    failed = true;
    play.fault("reach_target_clearance_fixture_setup");
  }
  if play
    .session
    .prepare_smoke_teleport(ATTACK_TARGET, reach_target_position)
    .is_err()
  {
    failed = true;
    play.fault("reach_attack_fixture_setup");
  }
  let reach_fixture_valid = play
    .session
    .actor(PLAYER)
    .zip(play.session.actor(ATTACK_TARGET))
    .is_some_and(|(player, target)| {
      player.melee_reach().value() >= 2
        && (player.position().x() - target.position().x()).unsigned_abs()
          + (player.position().y() - target.position().y()).unsigned_abs()
          == 2
    });
  if !reach_fixture_valid {
    failed = true;
    play.fault("reach_attack_fixture_invalid");
  }

  let mut attacks = 0;
  let mut extended_attack_observed = false;
  while play
    .session
    .actor(ATTACK_TARGET)
    .is_some_and(Actor::is_alive)
    && attacks < SMOKE_ENEMY_ATTACK_LIMIT
  {
    let command = play.session.legal_commands().into_iter().find(|command| {
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
        play.fault("reach_attack_not_legal");
      }
      let _ = play.submit_command("smoke", Command::Wait { actor: PLAYER });
      failed |= !play.drive_enemies(true);
      attacks = attacks.saturating_add(1);
      continue;
    };
    if attacks == 0 {
      extended_attack_observed = true;
    }
    failed |= !play.submit_command("smoke", command);
    failed |= !play.drive_enemies(true);
    attacks = attacks.saturating_add(1);
  }
  if play
    .session
    .actor(ATTACK_TARGET)
    .is_some_and(Actor::is_alive)
  {
    failed = true;
    play.fault("attack_target_not_defeated");
  }
  if !extended_attack_observed {
    failed = true;
    play.fault("reach_attack_not_observed");
  }

  failed |= !play.submit_command("smoke", Command::Unequip { actor: PLAYER });
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::UseItem {
      actor: PLAYER,
      item: CONSUME_ITEM,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command(
    "smoke",
    Command::Move {
      actor: PLAYER,
      direction: Direction::North,
    },
  );
  failed |= !play.drive_enemies(true);
  failed |= !play.submit_command("smoke", Command::Wait { actor: PLAYER });
  failed |= !play.drive_enemies(true);

  let command_coverage = SHOWCASE_COMMAND_KINDS
    .iter()
    .all(|kind| play.command_kinds.contains(*kind));
  let event_coverage = SHOWCASE_EVENT_KINDS
    .iter()
    .all(|kind| play.event_kinds.contains(*kind));
  if !command_coverage || !event_coverage {
    failed = true;
    let _ = play.record(
      "smoke_coverage_fault",
      json!({
        "commands_observed": play.command_kinds.iter().cloned().collect::<Vec<_>>(),
        "events_observed": play.event_kinds.iter().cloned().collect::<Vec<_>>(),
        "commands_expected": SHOWCASE_COMMAND_KINDS,
        "events_expected": SHOWCASE_EVENT_KINDS,
      }),
    );
  }
  let _ = play.record(
    if failed {
      "terminal_fault"
    } else {
      "smoke_complete"
    },
    json!({
      "commands_observed": play.command_kinds.iter().cloned().collect::<Vec<_>>(),
      "events_observed": play.event_kinds.iter().cloned().collect::<Vec<_>>(),
      "journal": play.journal_path().display().to_string(),
    }),
  );
  if play.export_replay().is_err() {
    failed = true;
    play.fault("replay_export_failed");
  }
  let _ = play.record(
    "shutdown",
    json!({ "reason": if failed { "smoke_fault" } else { "smoke_complete" } }),
  );
  if failed {
    ExitCode::from(1)
  } else {
    ExitCode::SUCCESS
  }
}
