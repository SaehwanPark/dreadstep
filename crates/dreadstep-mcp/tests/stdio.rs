//! Contract tests for the local stdio MCP server.

use std::{
  io::{BufRead, BufReader, Write},
  process::{Command, Stdio},
};

use dreadstep_mcp::{ActParams, DreadstepMcpServer, StartRunParams};
use dreadstep_protocol::{ActorId, CommandRequest};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::{Value, json};

#[tokio::test]
async fn in_memory_tools_return_structured_versioned_snapshots() {
  let server = DreadstepMcpServer::new(7).expect("fixed scenario should be valid");
  let observe = server.observe().await.expect("observe should succeed");
  assert_eq!(observe.0.protocol_version(), 1);
  let restarted = server
    .start_run(Parameters(StartRunParams { seed: 11 }))
    .await
    .expect("start_run should succeed");
  assert_eq!(
    restarted.0,
    server.observe().await.expect("observe should succeed").0
  );
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
  assert_eq!(DreadstepMcpServer::observe_tool_attr().name, "observe");
  assert_eq!(DreadstepMcpServer::start_run_tool_attr().name, "start_run");
  assert_eq!(DreadstepMcpServer::act_tool_attr().name, "act");
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
  let observed = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/call",
      "params": {"name": "observe", "arguments": {}}
    }),
    &mut input,
    &mut output,
  );
  let acted = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 5,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"wait": {"actor": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let rejected = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 6,
      "method": "tools/call",
      "params": {"name": "act", "arguments": {"request": {"wait": {"actor": 1}}}}
    }),
    &mut input,
    &mut output,
  );
  let after_rejection = round_trip(
    &json!({
      "jsonrpc": "2.0",
      "id": 7,
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
  assert_eq!(tools, ["act", "observe", "start_run"]);
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
  let observed_snapshot = &observed["result"]["structuredContent"];
  let acted_output = &acted["result"]["structuredContent"];
  let after_rejection_snapshot = &after_rejection["result"]["structuredContent"];
  assert_eq!(started_snapshot["protocol_version"], 1);
  assert_eq!(started_snapshot, observed_snapshot);
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
