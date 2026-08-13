//! Contract tests for the local stdio MCP server.

use std::{
  io::{BufRead, BufReader, Write},
  process::{Command, Stdio},
};

use dreadstep_mcp::{ActParams, DreadstepMcpServer, InspectParams, StartRunParams};
use dreadstep_protocol::{ActorId, CommandRequest, PROTOCOL_VERSION};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::{Value, json};

#[tokio::test]
#[expect(
  clippy::too_many_lines,
  reason = "the in-memory contract intentionally shows the ordered tool assertions"
)]
async fn in_memory_tools_return_structured_versioned_outputs() {
  let server = DreadstepMcpServer::new(7).expect("fixed scenario should be valid");
  let observe = server.observe().await.expect("observe should succeed");
  assert_eq!(observe.0.protocol_version(), 26);
  assert_eq!(observe.0.protocol_version(), PROTOCOL_VERSION);
  let restarted = server
    .start_run(Parameters(StartRunParams { seed: 11 }))
    .await
    .expect("start_run should succeed");
  assert_eq!(
    restarted.0,
    server.observe().await.expect("observe should succeed").0
  );
  let before_legal_actions = server.observe().await.expect("observe should succeed");
  let legal_actions = server
    .legal_actions()
    .await
    .expect("legal_actions should succeed");
  assert_eq!(legal_actions.0.len(), 7);
  assert_eq!(
    legal_actions.0[0],
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: dreadstep_protocol::Direction::North,
    }
  );
  assert_eq!(
    before_legal_actions.0,
    server.observe().await.expect("observe should succeed").0
  );
  assert!(
    server
      .get_history()
      .await
      .expect("get_history should succeed")
      .0
      .is_empty()
  );
  let initial_replay = server
    .get_replay()
    .await
    .expect("get_replay should succeed");
  assert_eq!(initial_replay.0.seed(), 11);
  assert!(initial_replay.0.commands().is_empty());
  let inspected = server
    .inspect(Parameters(InspectParams {
      actor: ActorId::new(1),
    }))
    .await
    .expect("inspect should succeed");
  assert_eq!(
    inspected.0.expect("known actor should be present").id(),
    ActorId::new(1)
  );
  let absent = server
    .inspect(Parameters(InspectParams {
      actor: ActorId::new(99),
    }))
    .await
    .expect("inspect should succeed");
  assert!(absent.0.is_none());
  let acted = server
    .act(Parameters(ActParams {
      request: CommandRequest::Wait {
        actor: ActorId::new(1),
      },
    }))
    .await
    .expect("act should succeed");
  assert_eq!(acted.0.seed(), 11);
  assert_eq!(acted.0.events().len(), 1);
  let replay_after_act = server
    .get_replay()
    .await
    .expect("get_replay should succeed");
  assert_eq!(replay_after_act.0.commands().len(), 1);
  assert_eq!(
    server
      .get_history()
      .await
      .expect("get_history should succeed")
      .0,
    vec![CommandRequest::Wait {
      actor: ActorId::new(1),
    }]
  );
  assert!(
    server
      .act(Parameters(ActParams {
        request: CommandRequest::Wait {
          actor: ActorId::new(1),
        },
      }))
      .await
      .is_err()
  );
  assert_eq!(
    server
      .get_history()
      .await
      .expect("get_history should succeed")
      .0
      .len(),
    1
  );
  assert_eq!(
    server
      .get_replay()
      .await
      .expect("get_replay should succeed")
      .0,
    replay_after_act.0
  );
  assert_eq!(DreadstepMcpServer::observe_tool_attr().name, "observe");
  assert_eq!(DreadstepMcpServer::start_run_tool_attr().name, "start_run");
  assert_eq!(
    DreadstepMcpServer::legal_actions_tool_attr().name,
    "legal_actions"
  );
  assert_eq!(DreadstepMcpServer::inspect_tool_attr().name, "inspect");
  assert_eq!(DreadstepMcpServer::act_tool_attr().name, "act");
  assert_eq!(
    DreadstepMcpServer::get_history_tool_attr().name,
    "get_history"
  );
  assert_eq!(
    DreadstepMcpServer::get_replay_tool_attr().name,
    "get_replay"
  );
}

fn round_trip(request: &Value, input: &mut impl Write, output: &mut impl BufRead) -> Value {
  writeln!(input, "{request}").expect("request should be written");
  input.flush().expect("request should be flushed");
  let mut line = String::new();
  output
    .read_line(&mut line)
    .expect("response should be readable");
  serde_json::from_str::<Value>(line.trim()).expect("stdout should contain JSON")
}

#[test]
#[expect(
  clippy::too_many_lines,
  reason = "the subprocess contract intentionally shows the ordered MCP exchange"
)]
fn subprocess_stdio_round_trip_discovers_and_calls_typed_tools() {
  let executable = env!("CARGO_BIN_EXE_dreadstep-mcp");
  let mut child = Command::new(executable)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("MCP binary should spawn");
  let mut input = child.stdin.take().expect("stdin should be piped");
  let stdout = child.stdout.take().expect("stdout should be piped");
  let mut output = BufReader::new(stdout);
  let initialized = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2026-07-28",
        "capabilities": {},
        "clientInfo": {"name": "dreadstep-test", "version": "0.1"}
      }
    }),
    &mut input,
    &mut output,
  );
  writeln!(
    input,
    "{}",
    json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
  )
  .expect("notification should be written");
  input.flush().expect("notification should be flushed");
  let tools_list = round_trip(
    &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    &mut input,
    &mut output,
  );
  let started = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "tools/call",
      "params": {"name": "start_run", "arguments": {"seed": 17}}
    }),
    &mut input,
    &mut output,
  );
  let inspected = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/call",
      "params": {"name": "inspect", "arguments": {"actor": 1}}
    }),
    &mut input,
    &mut output,
  );
  let absent = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 5,
      "method": "tools/call",
      "params": {"name": "inspect", "arguments": {"actor": 99}}
    }),
    &mut input,
    &mut output,
  );
  let legal = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 6,
      "method": "tools/call",
      "params": {"name": "legal_actions", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let history_before = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 7,
      "method": "tools/call",
      "params": {"name": "get_history", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let replay_before = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 8,
      "method": "tools/call",
      "params": {"name": "get_replay", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let observed = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 9,
      "method": "tools/call",
      "params": {"name": "observe", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let reloaded = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 10,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"reload": {"actor": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let enemy_acted = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 11,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"chase": {"actor": 2, "target": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let acted = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 12,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"wait": {"actor": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let history_after_act = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 13,
      "method": "tools/call",
      "params": {"name": "get_history", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let replay_after_act = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 14,
      "method": "tools/call",
      "params": {"name": "get_replay", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let rejected = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 15,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"wait": {"actor": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let history_after_rejection = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 16,
      "method": "tools/call",
      "params": {"name": "get_history", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let replay_after_rejection = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 17,
      "method": "tools/call",
      "params": {"name": "get_replay", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let after_rejection = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 18,
      "method": "tools/call",
      "params": {"name": "observe", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  drop(output);
  drop(input);
  let output = child
    .wait_with_output()
    .expect("MCP process should finish after stdin closes");
  assert!(output.status.success());
  assert!(output.stdout.is_empty());
  assert!(output.stderr.is_empty());

  assert!(initialized["result"]["serverInfo"].is_object());
  let mut tools = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .map(|tool| tool["name"].as_str().expect("tool name should be text"))
    .collect::<Vec<_>>();
  tools.sort_unstable();
  assert_eq!(
    tools,
    [
      "act",
      "get_history",
      "get_replay",
      "inspect",
      "legal_actions",
      "observe",
      "start_item_run",
      "start_run"
    ]
  );
  let get_replay_tool = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .find(|tool| tool["name"] == "get_replay")
    .expect("get_replay tool should be present");
  assert_eq!(get_replay_tool["inputSchema"]["type"], "object");
  assert_eq!(get_replay_tool["outputSchema"]["type"], "object");
  let get_history_tool = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .find(|tool| tool["name"] == "get_history")
    .expect("get_history tool should be present");
  assert_eq!(get_history_tool["inputSchema"]["type"], "object");
  assert_eq!(get_history_tool["outputSchema"]["type"], "array");
  let inspect_tool = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .find(|tool| tool["name"] == "inspect")
    .expect("inspect tool should be present");
  assert_eq!(inspect_tool["inputSchema"]["type"], "object");
  assert!(inspect_tool["inputSchema"]["properties"]["actor"].is_object());
  assert!(inspect_tool["outputSchema"]["anyOf"].is_array());
  let legal_actions_tool = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .find(|tool| tool["name"] == "legal_actions")
    .expect("legal_actions tool should be present");
  assert_eq!(legal_actions_tool["inputSchema"]["type"], "object");
  assert_eq!(legal_actions_tool["outputSchema"]["type"], "array");
  let act_tool = tools_list["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .find(|tool| tool["name"] == "act")
    .expect("act tool should be present");
  assert_eq!(act_tool["inputSchema"]["type"], "object");
  assert!(act_tool["inputSchema"]["properties"]["request"].is_object());
  assert_eq!(act_tool["outputSchema"]["type"], "object");
  let started_snapshot = &started["result"]["structuredContent"];
  let inspected_actor = &inspected["result"]["structuredContent"];
  let absent_actor = &absent["result"]["structuredContent"];
  let legal_actions_output = &legal["result"]["structuredContent"];
  let history_before_output = &history_before["result"]["structuredContent"];
  let replay_before_output = &replay_before["result"]["structuredContent"];
  let observed_snapshot = &observed["result"]["structuredContent"];
  let reloaded_output = &reloaded["result"]["structuredContent"];
  let enemy_acted_output = &enemy_acted["result"]["structuredContent"];
  let acted_output = &acted["result"]["structuredContent"];
  let history_after_act_output = &history_after_act["result"]["structuredContent"];
  let replay_after_act_output = &replay_after_act["result"]["structuredContent"];
  let history_after_rejection_output = &history_after_rejection["result"]["structuredContent"];
  let replay_after_rejection_output = &replay_after_rejection["result"]["structuredContent"];
  let after_rejection_snapshot = &after_rejection["result"]["structuredContent"];
  assert_eq!(started_snapshot["protocol_version"], 26);
  assert_eq!(started_snapshot["protocol_version"], PROTOCOL_VERSION);
  assert_eq!(started_snapshot["outcome"], "in_progress");
  assert_eq!(started_snapshot["actors"][0]["melee_reach"], 1);
  assert_eq!(started_snapshot, observed_snapshot);
  assert_eq!(inspected_actor["id"], 1);
  assert_eq!(inspected_actor, &started_snapshot["actors"][0]);
  assert!(absent_actor.is_null());
  assert_eq!(
    legal_actions_output
      .as_array()
      .expect("actions should be an array")
      .len(),
    7
  );
  assert_eq!(
    legal_actions_output[0],
    json!({"move": {"actor": 1, "direction": "north"}})
  );
  assert_eq!(
    legal_actions_output[6],
    json!({"attack": {"actor": 1, "target": 2}})
  );
  assert!(
    history_before_output
      .as_array()
      .expect("history should be an array")
      .is_empty()
  );
  assert_eq!(replay_before_output["seed"], 17);
  assert!(
    replay_before_output["commands"]
      .as_array()
      .expect("commands should be an array")
      .is_empty()
  );
  assert!(replay_before_output["digest"].is_number());
  assert_eq!(reloaded_output["events"][0]["reloaded"]["actor"], 1);
  assert_eq!(reloaded_output["events"][0]["reloaded"]["ammunition"], 3);
  assert_eq!(reloaded_output["snapshot"]["actors"][0]["ranged_ammo"], 3);
  assert_eq!(reloaded_output["snapshot"]["outcome"], "in_progress");
  assert_eq!(
    enemy_acted_output["events"][0]["movement_blocked"]["actor"],
    2
  );
  assert_eq!(
    history_after_act_output,
    &json!([
      {"reload": {"actor": 1}},
      {"chase": {"actor": 2, "target": 1}},
      {"wait": {"actor": 1}}
    ])
  );
  assert_eq!(history_after_rejection_output, history_after_act_output);
  assert_eq!(replay_after_act_output["seed"], 17);
  assert_eq!(
    replay_after_act_output["commands"],
    json!([
      {"reload": {"actor": 1}},
      {"chase": {"actor": 2, "target": 1}},
      {"wait": {"actor": 1}}
    ])
  );
  assert_eq!(replay_after_rejection_output, replay_after_act_output);
  assert_eq!(acted_output["seed"], 17);
  assert_eq!(acted_output["events"][0]["waited"]["actor"], 1);
  assert_eq!(&acted_output["snapshot"], after_rejection_snapshot);
  assert_eq!(rejected["error"]["code"], -32602);
  assert_eq!(
    started_snapshot["actors"]
      .as_array()
      .expect("actors should be present")
      .len(),
    2
  );
}

#[test]
fn subprocess_stdio_round_trip_accepts_player_drop_in_item_run() {
  let executable = env!("CARGO_BIN_EXE_dreadstep-mcp");
  let mut child = Command::new(executable)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("MCP binary should spawn");
  let mut input = child.stdin.take().expect("stdin should be piped");
  let stdout = child.stdout.take().expect("stdout should be piped");
  let mut output = BufReader::new(stdout);
  let _initialized = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2026-07-28",
        "capabilities": {},
        "clientInfo": {"name": "dreadstep-drop-test", "version": "0.1"}
      }
    }),
    &mut input,
    &mut output,
  );
  writeln!(
    input,
    "{}",
    json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
  )
  .expect("notification should be written");
  input.flush().expect("notification should be flushed");
  let started = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 2,
      "method": "tools/call",
      "params": {"name": "start_item_run", "arguments": {"seed": 17}}
    }),
    &mut input,
    &mut output,
  );
  let legal = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "tools/call",
      "params": {"name": "legal_actions", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let dropped = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"drop": {"actor": 1, "item": 101}}}}
    }),
    &mut input,
    &mut output,
  );
  drop(output);
  drop(input);
  let output = child
    .wait_with_output()
    .expect("MCP process should finish after stdin closes");
  assert!(output.status.success());
  assert!(output.stdout.is_empty());
  assert!(output.stderr.is_empty());
  assert_eq!(
    started["result"]["structuredContent"]["actors"][0]["inventory"][0]["id"],
    101
  );
  assert!(
    legal["result"]["structuredContent"]
      .as_array()
      .expect("legal actions should be an array")
      .iter()
      .any(|command| command == &json!({"drop": {"actor": 1, "item": 101}}))
  );
  assert_eq!(
    dropped["result"]["structuredContent"]["events"][0]["item_dropped"]["item"],
    101
  );
}
