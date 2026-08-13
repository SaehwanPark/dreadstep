//! Create-new JSONL journal and replay-export artifacts.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use super::REPLAY_EXPORT_SCHEMA_VERSION;
use super::format::{command_value, outcome_name};
use crate::PresentationRuntime;

pub(crate) type JournalHandle = Arc<Mutex<Journal>>;

pub(crate) struct Journal {
  writer: BufWriter<File>,
  path: PathBuf,
  started: Instant,
  sequence: u64,
}

impl Journal {
  pub(crate) fn open(directory: &Path) -> io::Result<Self> {
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

  pub(crate) fn path(&self) -> &Path {
    &self.path
  }

  pub(crate) fn record(&mut self, kind: &str, payload: Value) -> Result<(), JournalError> {
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

#[derive(Debug)]
pub(crate) struct JournalError(String);

impl JournalError {
  fn io(error: io::Error) -> Self {
    Self(error.to_string())
  }

  fn serialize(error: serde_json::Error) -> Self {
    Self(error.to_string())
  }
}

impl fmt::Display for JournalError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.0)
  }
}

pub(crate) fn record(
  journal: &JournalHandle,
  kind: &str,
  payload: Value,
) -> Result<(), JournalError> {
  let mut guard = match journal.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
  };
  guard.record(kind, payload)
}

pub(crate) fn journal_path(journal: &JournalHandle) -> PathBuf {
  let guard = match journal.lock() {
    Ok(guard) => guard,
    Err(poisoned) => poisoned.into_inner(),
  };
  guard.path().to_path_buf()
}

pub(crate) fn replay_export_value(runtime: &PresentationRuntime) -> Value {
  json!({
    "schema_version": REPLAY_EXPORT_SCHEMA_VERSION,
    "seed": runtime.seed(),
    "commands": runtime
      .replay_commands()
      .iter()
      .copied()
      .map(command_value)
      .collect::<Vec<_>>(),
    "replay_digest": runtime.replay_digest().value(),
    "outcome": outcome_name(runtime.snapshot().outcome()),
  })
}

pub(crate) fn export_replay(
  runtime: &PresentationRuntime,
  journal: &JournalHandle,
) -> Result<PathBuf, String> {
  let journal_path = journal_path(journal);
  let stem = journal_path
    .file_stem()
    .and_then(|value| value.to_str())
    .ok_or_else(|| "run journal path has no valid filename stem".to_string())?;
  let directory = journal_path
    .parent()
    .ok_or_else(|| "run journal path has no parent directory".to_string())?;
  let export = replay_export_value(runtime);
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
        record(
          journal,
          "replay_exported",
          json!({
            "path": path.display().to_string(),
            "schema_version": REPLAY_EXPORT_SCHEMA_VERSION,
            "commands": runtime.replay_commands().len(),
            "replay_digest": runtime.replay_digest().value(),
            "outcome": outcome_name(runtime.snapshot().outcome()),
          }),
        )
        .map_err(|error| error.to_string())?;
        return Ok(path);
      }
      Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
      Err(error) => return Err(error.to_string()),
    }
  }
  Err("could not allocate a unique replay export filename".to_string())
}

pub(crate) fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
  if let Some(message) = payload.downcast_ref::<&str>() {
    return (*message).to_string();
  }
  if let Some(message) = payload.downcast_ref::<String>() {
    return message.clone();
  }
  "non-string panic payload".to_string()
}
