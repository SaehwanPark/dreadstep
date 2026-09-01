//! Contract tests for the versioned diagnostic replay export.

use dreadstep_protocol::{
  CommandRequest, PROTOCOL_VERSION, ReplayExport, ReplayScenario, RunOutcome, StateDigest,
};

#[test]
fn replay_export_round_trips_typed_scenario_and_final_evidence() {
  let export = ReplayExport::new(
    7,
    ReplayScenario::Procedural { depth: 3 },
    vec![CommandRequest::Wait {
      actor: dreadstep_protocol::ActorId::new(1),
    }],
    StateDigest::new(11),
    StateDigest::new(22),
    RunOutcome::InProgress,
  );

  let encoded = serde_json::to_string(&export).expect("replay export should serialize");
  let decoded: ReplayExport = serde_json::from_str(&encoded).expect("export should decode");

  assert_eq!(decoded, export);
  assert_eq!(decoded.schema_version(), 2);
  assert_eq!(PROTOCOL_VERSION, 37);
  assert_eq!(decoded.scenario(), ReplayScenario::Procedural { depth: 3 });
  assert_eq!(decoded.commands().len(), 1);
  assert_eq!(decoded.replay_digest(), StateDigest::new(11));
  assert_eq!(decoded.state_digest(), StateDigest::new(22));
}

#[test]
fn replay_export_json_names_the_schema_and_scenario() {
  let export = ReplayExport::new(
    7,
    ReplayScenario::ItemShowcase,
    Vec::new(),
    StateDigest::new(1),
    StateDigest::new(2),
    RunOutcome::Victory,
  );
  let value = serde_json::to_value(export).expect("replay export should serialize");

  assert_eq!(value["schema_version"], 2);
  assert_eq!(value["scenario"]["item_showcase"], serde_json::Value::Null);
  assert_eq!(value["state_digest"], 2);
}

#[test]
fn smoke_fixture_is_an_explicit_diagnostic_scenario() {
  let json = serde_json::to_value(ReplayScenario::SmokeFixture).expect("scenario should encode");
  assert_eq!(json, serde_json::json!("smoke_fixture"));
  assert_eq!(ReplayScenario::SmokeFixture.depth(), None);
}
