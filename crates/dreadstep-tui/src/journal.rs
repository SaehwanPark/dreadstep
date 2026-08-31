//! Create-new JSONL journal and replay-export artifacts.

#![allow(clippy::needless_continue, clippy::needless_pass_by_value)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::session::Session;
use dreadstep_protocol::{ReplayExport, ReplayScenario};

/// A flushed, create-new JSONL run journal.
#[derive(Debug)]
pub struct Journal {
  writer: BufWriter<File>,
  path: PathBuf,
  started: Instant,
  sequence: u64,
}

impl Journal {
  /// Opens a unique journal file under `directory`.
  ///
  /// # Errors
  ///
  /// Returns an I/O error when the directory or file cannot be created.
  pub fn open(directory: &Path) -> io::Result<Self> {
    fs::create_dir_all(directory)?;
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
      Ok(duration) => duration.as_millis(),
      Err(_) => 0,
    };
    let pid = std::process::id();
    for counter in 0_u32..10_000 {
      let suffix = if counter == 0 {
        String::new()
      } else {
        format!("-{counter}")
      };
      let path = directory.join(format!("run-{timestamp}-{pid}{suffix}.jsonl"));
      match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(file) => {
          return Ok(Self {
            writer: BufWriter::new(file),
            path,
            started: Instant::now(),
            sequence: 0,
          });
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
        Err(error) => return Err(error),
      }
    }
    Err(io::Error::new(
      io::ErrorKind::AlreadyExists,
      "could not allocate a unique run journal filename",
    ))
  }

  /// Returns the journal path.
  #[must_use]
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// Appends one flushed JSONL record.
  ///
  /// # Errors
  ///
  /// Returns a [`JournalError`] on sequence overflow, serialization failure, or I/O failure.
  pub fn record(&mut self, kind: &str, payload: Value) -> Result<(), JournalError> {
    self.sequence = self
      .sequence
      .checked_add(1)
      .ok_or_else(|| JournalError("journal sequence overflow".to_string()))?;
    let entry = json!({
      "schema_version": 1,
      "sequence": self.sequence,
      "elapsed_ms": self.started.elapsed().as_millis(),
      "kind": kind,
      "payload": payload,
    });
    serde_json::to_writer(&mut self.writer, &entry).map_err(JournalError::serialize)?;
    self.writer.write_all(b"\n").map_err(JournalError::io)?;
    self.writer.flush().map_err(JournalError::io)
  }
}

/// Journal write failure.
#[derive(Debug)]
pub struct JournalError(String);

impl JournalError {
  fn io(error: io::Error) -> Self {
    Self(error.to_string())
  }

  fn serialize(error: serde_json::Error) -> Self {
    Self(error.to_string())
  }
}

impl std::fmt::Display for JournalError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(&self.0)
  }
}

impl std::error::Error for JournalError {}

/// Writes a sibling replay-export JSON file from the accepted core trace.
///
/// # Errors
///
/// Returns a string when a unique filename cannot be allocated or written.
pub fn export_replay(session: &Session, journal: &Journal) -> Result<PathBuf, String> {
  let scenario = match session.scenario() {
    crate::session::Scenario::ItemShowcase => ReplayScenario::ItemShowcase,
    crate::session::Scenario::Procedural { depth } => ReplayScenario::Procedural { depth },
  };
  export_replay_with_scenario(session, journal, scenario)
}

/// Writes a replay export using an explicit diagnostic scenario label.
pub fn export_replay_with_scenario(
  session: &Session,
  journal: &Journal,
  scenario: ReplayScenario,
) -> Result<PathBuf, String> {
  let journal_path = journal.path();
  let stem = journal_path
    .file_stem()
    .and_then(|value| value.to_str())
    .ok_or_else(|| "run journal path has no valid filename stem".to_string())?;
  let directory = journal_path
    .parent()
    .ok_or_else(|| "run journal path has no parent directory".to_string())?;
  let export = ReplayExport::new(
    session.seed(),
    scenario,
    session
      .replay_commands()
      .iter()
      .copied()
      .map(dreadstep_protocol::CommandRequest::from)
      .collect(),
    dreadstep_protocol::StateDigest::new(session.replay_digest().value()),
    dreadstep_protocol::StateDigest::new(session.digest().value()),
    session.outcome().into(),
  );
  for counter in 0_u32..10_000 {
    let suffix = if counter == 0 {
      String::new()
    } else {
      format!("-{counter}")
    };
    let path = directory.join(format!("{stem}.replay{suffix}.json"));
    match OpenOptions::new().create_new(true).write(true).open(&path) {
      Ok(mut file) => {
        serde_json::to_writer_pretty(&mut file, &export).map_err(|error| error.to_string())?;
        file.write_all(b"\n").map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        return Ok(path);
      }
      Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(error.to_string()),
    }
  }
  Err("could not allocate a unique replay export filename".to_string())
}
