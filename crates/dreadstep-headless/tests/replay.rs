//! Playback-compatible replay verification tests.

use std::fs;

use dreadstep_content::{procedural_floor, starter_item_showcase_floor};
use dreadstep_core::{Command, ReplayTrace};
use dreadstep_headless::{ReplayVerificationError, verify_replay_file};
use dreadstep_protocol::{CommandRequest, ReplayExport, ReplayScenario, RunOutcome, StateDigest};

fn export_for_item_showcase() -> ReplayExport {
  let mut world = starter_item_showcase_floor().expect("item showcase should build");
  let command = Command::Wait {
    actor: dreadstep_core::ActorId::new(1),
  };
  world.execute(command).expect("wait should be accepted");
  let mut trace = ReplayTrace::new(7);
  trace.record(command);
  ReplayExport::new(
    7,
    ReplayScenario::ItemShowcase,
    vec![CommandRequest::from(command)],
    StateDigest::new(trace.digest().value()),
    StateDigest::new(world.digest().value()),
    RunOutcome::from(world.outcome()),
  )
}

#[test]
fn verifies_authored_item_showcase_replay() {
  let directory = tempfile_directory();
  let path = directory.join("item.replay.json");
  fs::write(
    &path,
    serde_json::to_vec_pretty(&export_for_item_showcase()).expect("export should encode"),
  )
  .expect("export should write");

  let verified = verify_replay_file(&path).expect("authored replay should verify");
  assert_eq!(verified.scenario(), ReplayScenario::ItemShowcase);
  assert_eq!(verified.commands().len(), 1);
  assert_eq!(verified.outcome(), RunOutcome::InProgress);
}

#[test]
fn verifies_procedural_replay_and_rejects_state_mismatch() {
  let seed = 7;
  let depth = 3;
  let mut world = procedural_floor(seed, depth).expect("procedural floor should build");
  let command = Command::Wait {
    actor: dreadstep_core::ActorId::new(1),
  };
  world.execute(command).expect("wait should be accepted");
  let mut trace = ReplayTrace::new(seed);
  trace.record(command);
  let mut export = ReplayExport::new(
    seed,
    ReplayScenario::Procedural { depth },
    vec![CommandRequest::from(command)],
    StateDigest::new(trace.digest().value()),
    StateDigest::new(world.digest().value()),
    RunOutcome::from(world.outcome()),
  );

  let directory = tempfile_directory();
  let path = directory.join("procedural.replay.json");
  fs::write(
    &path,
    serde_json::to_vec(&export).expect("export should encode"),
  )
  .expect("export should write");
  verify_replay_file(&path).expect("procedural replay should verify");

  export = ReplayExport::new(
    seed,
    ReplayScenario::Procedural { depth },
    export.commands().to_vec(),
    export.replay_digest(),
    StateDigest::new(0),
    export.outcome(),
  );
  fs::write(
    &path,
    serde_json::to_vec(&export).expect("tampered export should encode"),
  )
  .expect("tampered export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::StateDigestMismatch { .. })
  ));
}

#[test]
fn rejects_unsupported_schema_and_command_replay_digest_mismatch() {
  let directory = tempfile_directory();
  let path = directory.join("invalid.replay.json");
  let mut value = serde_json::to_value(export_for_item_showcase()).expect("export should encode");
  value["schema_version"] = serde_json::json!(1);
  fs::write(
    &path,
    serde_json::to_vec(&value).expect("invalid export should encode"),
  )
  .expect("invalid export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::UnsupportedSchema {
      expected: 2,
      actual: 1
    })
  ));

  value["schema_version"] = serde_json::json!(2);
  value["replay_digest"] = serde_json::json!(0);
  fs::write(
    &path,
    serde_json::to_vec(&value).expect("tampered export should encode"),
  )
  .expect("tampered export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::ReplayDigestMismatch { .. })
  ));
}

#[test]
fn rejects_malformed_export_command_and_outcome_mismatches() {
  let directory = tempfile_directory();
  let path = directory.join("invalid-evidence.replay.json");

  fs::write(&path, b"not json").expect("malformed export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::Malformed { .. })
  ));

  let mut value = serde_json::to_value(export_for_item_showcase()).expect("export should encode");
  value["commands"] = serde_json::json!([{"wait": {"actor": 2}}]);
  fs::write(
    &path,
    serde_json::to_vec(&value).expect("rejected-command export should encode"),
  )
  .expect("rejected-command export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::CommandRejected { index: 0, .. })
  ));

  value = serde_json::to_value(export_for_item_showcase()).expect("export should encode");
  value["outcome"] = serde_json::json!("victory");
  fs::write(
    &path,
    serde_json::to_vec(&value).expect("outcome-mismatch export should encode"),
  )
  .expect("outcome-mismatch export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::OutcomeMismatch {
      expected: RunOutcome::Victory,
      actual: RunOutcome::InProgress
    })
  ));
}

#[test]
fn rejects_diagnostic_smoke_fixture_as_non_replayable() {
  let export = ReplayExport::new(
    7,
    ReplayScenario::SmokeFixture,
    Vec::new(),
    StateDigest::new(0),
    StateDigest::new(0),
    RunOutcome::InProgress,
  );
  let directory = tempfile_directory();
  let path = directory.join("smoke.replay.json");
  fs::write(
    &path,
    serde_json::to_vec(&export).expect("export should encode"),
  )
  .expect("export should write");
  assert!(matches!(
    verify_replay_file(&path),
    Err(ReplayVerificationError::NonReplayable {
      scenario: ReplayScenario::SmokeFixture
    })
  ));
}

#[test]
fn verifier_process_prints_success_and_nonzero_failure() {
  let directory = tempfile_directory();
  let path = directory.join("process.replay.json");
  fs::write(
    &path,
    serde_json::to_vec(&export_for_item_showcase()).expect("export should encode"),
  )
  .expect("export should write");

  let binary = std::env::var("CARGO_BIN_EXE_dreadstep-headless")
    .expect("Cargo should provide the headless binary path");
  let verified = std::process::Command::new(&binary)
    .args([
      "--verify-replay",
      path.to_str().expect("path should be UTF-8"),
    ])
    .output()
    .expect("verifier process should launch");
  assert!(verified.status.success());
  assert!(String::from_utf8_lossy(&verified.stdout).starts_with("verified_replay\n"));

  let smoke_path = directory.join("process-smoke.replay.json");
  let smoke_export = ReplayExport::new(
    7,
    ReplayScenario::SmokeFixture,
    Vec::new(),
    StateDigest::new(0),
    StateDigest::new(0),
    RunOutcome::InProgress,
  );
  fs::write(
    &smoke_path,
    serde_json::to_vec(&smoke_export).expect("smoke export should encode"),
  )
  .expect("smoke export should write");
  let rejected = std::process::Command::new(binary)
    .args([
      "--verify-replay",
      smoke_path.to_str().expect("path should be UTF-8"),
    ])
    .output()
    .expect("verifier process should launch");
  assert_eq!(rejected.status.code(), Some(2));
  assert!(String::from_utf8_lossy(&rejected.stderr).contains("diagnostic-only"));
}

fn tempfile_directory() -> std::path::PathBuf {
  let directory =
    std::env::temp_dir().join(format!("dreadstep-replay-test-{}", std::process::id()));
  fs::create_dir_all(&directory).expect("temporary directory should create");
  directory
}
