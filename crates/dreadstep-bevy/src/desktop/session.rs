//! Desktop session status, messages, and shutdown finalization.

use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};

use bevy::ecs::resource::Resource;
use bevy::time::{Timer, TimerMode};
use dreadstep_core::ItemId;
use serde_json::Value;

use super::journal::{JournalHandle, record};
use super::{ENEMY_DELAY, EQUIP_ITEM};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DesktopStatus {
  Running,
  Victory,
  Defeat,
  Faulted(String),
  Shutdown(String),
}

#[derive(Resource)]
pub(crate) struct DesktopSession {
  pub(crate) seed: u64,
  pub(crate) procedural: bool,
  pub(crate) depth: u32,
  pub(crate) journal: JournalHandle,
  pub(crate) status: DesktopStatus,
  pub(crate) selected_item: Option<ItemId>,
  pub(crate) messages: VecDeque<String>,
  pub(crate) enemy_timer: Timer,
  pub(crate) command_kinds: BTreeSet<String>,
  pub(crate) event_kinds: BTreeSet<String>,
  pub(crate) terminal_recorded: bool,
}

impl DesktopSession {
  pub(crate) fn new(seed: u64, journal: JournalHandle) -> Self {
    Self::new_with_scenario(seed, false, 1, journal)
  }

  pub(crate) fn new_with_scenario(
    seed: u64,
    procedural: bool,
    depth: u32,
    journal: JournalHandle,
  ) -> Self {
    Self {
      seed,
      procedural,
      depth,
      journal,
      status: DesktopStatus::Running,
      selected_item: Some(EQUIP_ITEM),
      messages: VecDeque::new(),
      enemy_timer: Timer::from_seconds(ENEMY_DELAY.as_secs_f32(), TimerMode::Once),
      command_kinds: BTreeSet::new(),
      event_kinds: BTreeSet::new(),
      terminal_recorded: false,
    }
  }

  pub(crate) fn push_message(&mut self, message: impl Into<String>) {
    self.messages.push_back(message.into());
    while self.messages.len() > 8 {
      let _ = self.messages.pop_front();
    }
  }

  pub(crate) fn fault(&mut self, error: impl Into<String>) {
    let error = error.into();
    self.status = DesktopStatus::Faulted(error.clone());
    self.push_message(format!("Journal/runtime fault: {error}"));
    eprintln!("dreadstep: {error}");
  }
}

#[derive(Default)]
pub(crate) struct FinalizationReport {
  pub(crate) complete: bool,
  pub(crate) error: Option<String>,
}

#[derive(Clone, Resource)]
pub(crate) struct FinalizationHandle(pub(crate) Arc<Mutex<FinalizationReport>>);

impl FinalizationHandle {
  pub(crate) fn new() -> Self {
    Self(Arc::new(Mutex::new(FinalizationReport::default())))
  }

  pub(crate) fn finish(&self, error: Option<String>) {
    let mut report = match self.0.lock() {
      Ok(report) => report,
      Err(poisoned) => poisoned.into_inner(),
    };
    report.complete = true;
    report.error = error;
  }
}

pub(crate) fn record_session(session: &mut DesktopSession, kind: &str, payload: Value) -> bool {
  match record(&session.journal, kind, payload) {
    Ok(()) => true,
    Err(error) => {
      session.fault(format!("journal write failed: {error}"));
      false
    }
  }
}
