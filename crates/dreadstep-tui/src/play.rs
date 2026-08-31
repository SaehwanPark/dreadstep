//! Shared command submission, enemy driving, and journal/frame recording.

use std::collections::BTreeSet;
use std::process::ExitCode;

use dreadstep_core::{Command, RunOutcome, Tile};
use serde_json::{Value, json};

use crate::frame::{TextFrame, render_frame};
use crate::input::UiState;
use crate::journal::{Journal, JournalError, export_replay, export_replay_with_scenario};
use crate::kinds::{command_name, command_value, event_name, outcome_name};
use crate::messages::format_event;
use crate::session::{PLAYER, Scenario, Session};

/// Presentation-owned run status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
  /// The player may still act.
  Running,
  /// Core reported victory.
  Victory,
  /// Core reported player death.
  Defeat,
  /// The process is shutting down.
  Shutdown(String),
  /// An adapter fault occurred.
  Faulted(String),
}

/// One interactive or smoke playthrough.
pub struct Play {
  /// Core-backed session.
  pub session: Session,
  /// Presentation UI state.
  pub ui: UiState,
  journal: Journal,
  /// Adapter run status.
  pub status: Status,
  /// Observed command kinds for smoke coverage.
  pub command_kinds: BTreeSet<String>,
  /// Observed event kinds for smoke coverage.
  pub event_kinds: BTreeSet<String>,
}

impl Play {
  /// Starts a playthrough around an existing session and journal.
  #[must_use]
  pub fn new(session: Session, journal: Journal) -> Self {
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    Self {
      session,
      ui,
      journal,
      status: Status::Running,
      command_kinds: BTreeSet::new(),
      event_kinds: BTreeSet::new(),
    }
  }

  /// Returns the journal path.
  #[must_use]
  pub fn journal_path(&self) -> std::path::PathBuf {
    self.journal.path().to_path_buf()
  }

  /// Records one journal entry.
  ///
  /// # Errors
  ///
  /// Returns [`JournalError`] when the record cannot be written.
  pub fn record(&mut self, kind: &str, payload: Value) -> Result<(), JournalError> {
    self.journal.record(kind, payload)
  }

  /// Renders the current frame.
  #[must_use]
  pub fn frame(&self) -> TextFrame {
    render_frame(&self.session, &self.ui)
  }

  /// Records a plain `frame` journal payload for agent monitoring.
  pub fn record_frame(&mut self, reason: &str) -> bool {
    let frame = self.frame().plain();
    self
      .record(
        "frame",
        json!({
          "reason": reason,
          "seed": self.session.seed(),
          "digest": self.session.digest().value(),
          "outcome": outcome_name(self.session.outcome()),
          "next_actor": self.session.next_actor().map(dreadstep_core::ActorId::value),
          "frame": frame,
        }),
      )
      .is_ok()
  }

  /// Submits one core command and records journal evidence.
  pub fn submit_command(&mut self, source: &str, command: Command) -> bool {
    if !matches!(self.status, Status::Running) {
      return false;
    }
    let before = json!({
      "source": source,
      "command": command_value(command),
      "digest": self.session.digest().value(),
    });
    if self.record("command_requested", before.clone()).is_err() {
      self.fault("journal write failed");
      return false;
    }
    self.command_kinds.insert(command_name(command).to_string());
    match self.session.execute(command) {
      Ok(output) => {
        for event in output.events() {
          self.event_kinds.insert(event_name(*event).to_string());
          self.ui.push_message(format_event(&self.session, *event));
        }
        let payload = json!({
          "source": source,
          "command": command_value(command),
          "events": output.events().iter().copied().map(event_name).collect::<Vec<_>>(),
          "digest": self.session.digest().value(),
        });
        if self.record("action_accepted", payload).is_err() {
          self.fault("journal write failed");
          return false;
        }
        match self.session.outcome() {
          RunOutcome::Defeat => {
            self.status = Status::Defeat;
            self.ui.push_message("You die...".to_string());
            let _ = self.record(
              "terminal_defeat",
              json!({ "reason": "player_died", "digest": self.session.digest().value() }),
            );
          }
          RunOutcome::Victory => {
            self.status = Status::Victory;
            self
              .ui
              .push_message("Showcase complete — every enemy is dead.".to_string());
            let _ = self.record(
              "terminal_victory",
              json!({ "reason": "all_enemies_dead", "digest": self.session.digest().value() }),
            );
          }
          RunOutcome::InProgress => {}
        }
        let _ = self.record_frame("after_command");
        true
      }
      Err(error) => {
        self.ui.push_message(format!("Rejected: {error}"));
        let payload = json!({
          "source": source,
          "command": command_value(command),
          "error": error.to_string(),
          "unchanged": true,
          "before": before,
        });
        if self.record("action_rejected", payload).is_err() {
          self.fault("journal write failed");
          return false;
        }
        let _ = self.record_frame("after_rejection");
        false
      }
    }
  }

  /// Drives scheduled enemies using core preference, with the smoke low-HP wait guard.
  pub fn drive_enemies(&mut self, smoke_guard: bool) -> bool {
    for _ in 0..64 {
      if self.session.next_actor() == Some(PLAYER) {
        return true;
      }
      let Some(actor) = self.session.next_actor() else {
        return true;
      };
      let legal = self.session.legal_commands();
      let command = self
        .session
        .preferred_enemy_command(actor, PLAYER)
        .and_then(|command| {
          let player_is_low = self
            .session
            .actor(PLAYER)
            .is_some_and(|record| record.hit_points().value() <= 3);
          if smoke_guard
            && player_is_low
            && matches!(command, Command::Attack { .. } | Command::RangedAttack { .. })
          {
            legal.iter().copied().find(|candidate| {
              matches!(candidate, Command::Wait { actor: candidate_actor } if *candidate_actor == actor)
            })
          } else {
            Some(command)
          }
        });
      let Some(command) = command else {
        self.fault(format!(
          "no legal enemy command for actor {}",
          actor.value()
        ));
        return false;
      };
      if !self.submit_command("enemy_driver", command) {
        return false;
      }
    }
    self.fault("enemy driver exceeded turn bound");
    false
  }

  /// Restarts the same seed and scenario.
  pub fn restart(&mut self) -> bool {
    let restarted = match self.session.scenario() {
      Scenario::Procedural { depth } => Session::start_procedural_run(self.session.seed(), depth),
      Scenario::ItemShowcase => Session::start_item_run(self.session.seed()),
    };
    match restarted {
      Ok(session) => {
        self.session = session;
        self.ui = UiState::new();
        self.ui.select_default_item(&self.session);
        self.status = Status::Running;
        self.command_kinds.clear();
        self.event_kinds.clear();
        let _ = self.record(
          "run_restarted",
          json!({ "seed": self.session.seed(), "digest": self.session.digest().value() }),
        );
        let _ = self.record_frame("restart");
        true
      }
      Err(error) => {
        self.fault(format!("restart failed: {error}"));
        false
      }
    }
  }

  /// Advances a procedural run after victory.
  pub fn advance_floor(&mut self) -> bool {
    let Scenario::Procedural { depth } = self.session.scenario() else {
      return false;
    };
    if !matches!(self.status, Status::Victory) {
      return false;
    }
    let Some(next_depth) = depth.checked_add(1) else {
      self.fault("procedural floor depth overflow");
      return false;
    };
    match Session::start_procedural_run(self.session.seed(), next_depth) {
      Ok(session) => {
        self.session = session;
        self.ui = UiState::new();
        self.ui.select_default_item(&self.session);
        self.status = Status::Running;
        self.command_kinds.clear();
        self.event_kinds.clear();
        let _ = self.record(
          "floor_advanced",
          json!({
            "seed": self.session.seed(),
            "scenario": "procedural_floor",
            "depth": next_depth,
          }),
        );
        let _ = self.record_frame("floor_advanced");
        true
      }
      Err(error) => {
        self.fault(format!("procedural floor advance failed: {error}"));
        false
      }
    }
  }

  /// Records a fault and marks the playthrough failed.
  pub fn fault(&mut self, reason: impl Into<String>) {
    let reason = reason.into();
    self.status = Status::Faulted(reason.clone());
    let _ = self.record("fault", json!({ "reason": reason }));
  }

  /// Places a smoke terrain fixture.
  pub fn prepare_tile(&mut self, position: dreadstep_core::Position, tile: Tile) -> bool {
    if let Err(error) = self.session.prepare_smoke_tile(position, tile) {
      self.fault(error.to_string());
      return false;
    }
    true
  }

  /// Writes the sibling replay export.
  pub fn export_replay(&self) -> Result<std::path::PathBuf, String> {
    export_replay(&self.session, &self.journal)
  }

  /// Writes the replay export with an explicit diagnostic scenario label.
  pub fn export_replay_as(
    &self,
    scenario: dreadstep_protocol::ReplayScenario,
  ) -> Result<std::path::PathBuf, String> {
    export_replay_with_scenario(&self.session, &self.journal, scenario)
  }

  /// Finalizes the journal with shutdown evidence.
  pub fn shutdown(&mut self, reason: &str) -> ExitCode {
    if matches!(self.status, Status::Faulted(_)) {
      let _ = self.record("shutdown", json!({ "reason": reason, "failed": true }));
      return ExitCode::from(1);
    }
    self.status = Status::Shutdown(reason.to_string());
    if let Err(error) = self.export_replay() {
      let failure_reason = format!("replay export failed: {error}");
      self.status = Status::Faulted(failure_reason.clone());
      let _ = self.record("fault", json!({ "reason": failure_reason }));
      let _ = self.record(
        "shutdown",
        json!({ "reason": reason, "failed": true, "error": error }),
      );
      return ExitCode::from(1);
    }
    let _ = self.record("shutdown", json!({ "reason": reason }));
    ExitCode::SUCCESS
  }
}

#[cfg(test)]
mod tests {
  use std::time::{SystemTime, UNIX_EPOCH};

  use super::{Play, Status};
  use crate::journal::Journal;
  use crate::session::Session;

  fn test_directory() -> std::path::PathBuf {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock is after epoch")
      .as_nanos();
    std::env::temp_dir().join(format!("dreadstep-tui-shutdown-{timestamp}"))
  }

  #[test]
  fn shutdown_faults_when_replay_export_cannot_be_written() {
    let directory = test_directory();
    let journal = Journal::open(&directory).expect("journal should open");
    let mut play = Play::new(
      Session::start_item_run(7).expect("item showcase should start"),
      journal,
    );
    std::fs::remove_dir_all(&directory).expect("test directory should be removable");

    assert_eq!(play.shutdown("test"), std::process::ExitCode::from(1));
    assert!(
      matches!(play.status, Status::Faulted(reason) if reason.starts_with("replay export failed:"))
    );
  }
}
