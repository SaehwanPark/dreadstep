//! Playback-compatible verification for diagnostic replay exports.
//!
//! The verifier reconstructs only the authored content entry points that adapters can export.
//! Every accepted request is sent through `dreadstep-core`; no adapter-side simulation is used.

use std::{fmt, fs, path::Path};

use dreadstep_content::{
  ContentError, procedural_floor, starter_floor, starter_item_showcase_floor,
};
use dreadstep_core::{Command, CommandError, ReplayTrace};
use dreadstep_protocol::{
  REPLAY_EXPORT_SCHEMA_VERSION, ReplayExport, ReplayScenario, RunOutcome,
  StateDigest as ProtocolStateDigest,
};

/// Errors raised while loading or replaying diagnostic evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayVerificationError {
  /// The export file could not be read.
  Read {
    /// The path that was requested.
    path: String,
    /// The operating-system error text.
    message: String,
  },
  /// The file did not contain a valid replay-export JSON value.
  Malformed {
    /// The decoder error text.
    message: String,
  },
  /// The export schema is not supported by this verifier.
  UnsupportedSchema {
    /// The schema version required by this verifier.
    expected: u16,
    /// The schema version carried by the export.
    actual: u16,
  },
  /// The referenced authored content could not be rebuilt.
  Scenario {
    /// The scenario requested by the export.
    scenario: ReplayScenario,
    /// The content validation error text.
    message: String,
  },
  /// The export records an adapter fixture whose setup mutations are not serialized.
  NonReplayable {
    /// The diagnostic-only scenario.
    scenario: ReplayScenario,
  },
  /// One recorded command was rejected during replay.
  CommandRejected {
    /// The zero-based command index.
    index: usize,
    /// The core rejection.
    source: CommandError,
  },
  /// The accepted-command trace digest did not match the export.
  ReplayDigestMismatch {
    /// The digest recorded by the export.
    expected: ProtocolStateDigest,
    /// The digest produced by replaying its commands.
    actual: ProtocolStateDigest,
  },
  /// The final core world digest did not match the export.
  StateDigestMismatch {
    /// The digest recorded by the export.
    expected: ProtocolStateDigest,
    /// The digest produced by replaying its commands.
    actual: ProtocolStateDigest,
  },
  /// The final core outcome did not match the export.
  OutcomeMismatch {
    /// The outcome recorded by the export.
    expected: RunOutcome,
    /// The outcome produced by replaying its commands.
    actual: RunOutcome,
  },
}

impl fmt::Display for ReplayVerificationError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Read { path, message } => write!(formatter, "cannot read replay {path}: {message}"),
      Self::Malformed { message } => write!(formatter, "malformed replay export: {message}"),
      Self::UnsupportedSchema { expected, actual } => write!(
        formatter,
        "unsupported replay schema {actual}; expected {expected}"
      ),
      Self::Scenario { scenario, message } => {
        write!(
          formatter,
          "cannot build replay scenario {scenario:?}: {message}"
        )
      }
      Self::NonReplayable { scenario } => write!(
        formatter,
        "replay scenario {scenario:?} is diagnostic-only and cannot be verified"
      ),
      Self::CommandRejected { index, source } => {
        write!(formatter, "replay command {index} rejected: {source}")
      }
      Self::ReplayDigestMismatch { expected, actual } => write!(
        formatter,
        "replay digest mismatch: expected {}, got {}",
        expected.value(),
        actual.value()
      ),
      Self::StateDigestMismatch { expected, actual } => write!(
        formatter,
        "state digest mismatch: expected {}, got {}",
        expected.value(),
        actual.value()
      ),
      Self::OutcomeMismatch { expected, actual } => {
        write!(
          formatter,
          "outcome mismatch: expected {expected:?}, got {actual:?}"
        )
      }
    }
  }
}

impl std::error::Error for ReplayVerificationError {}

/// Loads and verifies one diagnostic replay export.
///
/// The returned export is the decoded, evidence-checked value. Verification is deterministic for
/// a given file because every request is executed by the core world reconstructed from its seed
/// and scenario metadata.
///
/// # Errors
///
/// Returns [`ReplayVerificationError`] when the file is unreadable, malformed, unsupported, or
/// inconsistent with the reconstructed core state.
pub fn verify_replay_file(path: impl AsRef<Path>) -> Result<ReplayExport, ReplayVerificationError> {
  let path = path.as_ref();
  let bytes = fs::read(path).map_err(|error| ReplayVerificationError::Read {
    path: path.display().to_string(),
    message: error.to_string(),
  })?;
  let export: ReplayExport =
    serde_json::from_slice(&bytes).map_err(|error| ReplayVerificationError::Malformed {
      message: error.to_string(),
    })?;
  if export.schema_version() != REPLAY_EXPORT_SCHEMA_VERSION {
    return Err(ReplayVerificationError::UnsupportedSchema {
      expected: REPLAY_EXPORT_SCHEMA_VERSION,
      actual: export.schema_version(),
    });
  }

  let mut world = build_scenario(export.scenario(), export.seed())?;
  let mut trace = ReplayTrace::new(export.seed());
  for (index, request) in export.commands().iter().copied().enumerate() {
    let command: Command = request.into();
    world
      .execute(command)
      .map_err(|source| ReplayVerificationError::CommandRejected { index, source })?;
    trace.record(command);
  }

  let actual_replay_digest = ProtocolStateDigest::new(trace.digest().value());
  if export.replay_digest() != actual_replay_digest {
    return Err(ReplayVerificationError::ReplayDigestMismatch {
      expected: export.replay_digest(),
      actual: actual_replay_digest,
    });
  }
  let actual_state_digest = ProtocolStateDigest::new(world.digest().value());
  if export.state_digest() != actual_state_digest {
    return Err(ReplayVerificationError::StateDigestMismatch {
      expected: export.state_digest(),
      actual: actual_state_digest,
    });
  }
  let actual_outcome: RunOutcome = world.outcome().into();
  if export.outcome() != actual_outcome {
    return Err(ReplayVerificationError::OutcomeMismatch {
      expected: export.outcome(),
      actual: actual_outcome,
    });
  }
  Ok(export)
}

/// Renders concise process output for a verified replay export.
#[must_use]
pub fn render_replay_verification(export: &ReplayExport) -> String {
  let scenario = match export.scenario() {
    ReplayScenario::Starter => "starter".to_owned(),
    ReplayScenario::ItemShowcase => "item_showcase".to_owned(),
    ReplayScenario::SmokeFixture => "smoke_fixture".to_owned(),
    ReplayScenario::Procedural { depth } => format!("procedural_floor:{depth}"),
  };
  format!(
    "verified_replay\nseed={}\nscenario={scenario}\ncommands={}\nreplay_digest={}\nstate_digest={}\noutcome={}\n",
    export.seed(),
    export.commands().len(),
    export.replay_digest().value(),
    export.state_digest().value(),
    outcome_name(export.outcome()),
  )
}

const fn outcome_name(outcome: RunOutcome) -> &'static str {
  match outcome {
    RunOutcome::InProgress => "in_progress",
    RunOutcome::Defeat => "defeat",
    RunOutcome::Victory => "victory",
  }
}

fn build_scenario(
  scenario: ReplayScenario,
  seed: u64,
) -> Result<dreadstep_core::WorldState, ReplayVerificationError> {
  let result = match scenario {
    ReplayScenario::Starter => starter_floor(),
    ReplayScenario::ItemShowcase => starter_item_showcase_floor(),
    ReplayScenario::SmokeFixture => {
      return Err(ReplayVerificationError::NonReplayable { scenario });
    }
    ReplayScenario::Procedural { depth } => procedural_floor(seed, depth),
  };
  result.map_err(|error: ContentError| ReplayVerificationError::Scenario {
    scenario,
    message: error.to_string(),
  })
}
