//! Contract tests for the local stdio MCP observation server.

use std::{
  io::Write,
  process::{Command, Stdio},
};

use dreadstep_mcp::{DreadstepMcpServer, StartRunParams};
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
  assert_eq!(DreadstepMcpServer::observe_tool_attr().name, "observe");
  assert_eq!(DreadstepMcpServer::start_run_tool_attr().name, "start_run");
}

#[test]
fn subprocess_stdio_round_trip_discovers_and_calls_only_observation_tools() {
  let executable = env!("CARGO_BIN_EXE_dreadstep-mcp");
  let requests = [
    json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "protocolVersion": "2026-07-28",
        "capabilities": {},
        "clientInfo": {"name": "dreadstep-test", "version": "0.1"}
      }
    }),
    json!({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}),
    json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "tools/call",
      "params": {"name": "start_run", "arguments": {"seed": 17}}
    }),
    json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "tools/call",
      "params": {"name": "observe", "arguments": {}}
    }),
  ];
  let mut child = Command::new(executable)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("MCP binary should spawn");
  let input = requests
    .iter()
    .map(Value::to_string)
    .collect::<Vec<_>>()
    .join("\n");
  child
    .stdin
    .take()
    .expect("stdin should be piped")
    .write_all(format!("{input}\n").as_bytes())
    .expect("requests should be written");
  let output = child
    .wait_with_output()
    .expect("MCP process should finish after stdin closes");
  assert!(output.status.success());
  assert!(output.stderr.is_empty());

  let responses = output
    .stdout
    .split(|byte| *byte == b'\n')
    .filter(|line| !line.is_empty())
    .map(|line| serde_json::from_slice::<Value>(line).expect("stdout should contain JSON"))
    .collect::<Vec<_>>();
  let by_id = |id| {
    responses
      .iter()
      .find(|response| response["id"] == id)
      .expect("response id should be present")
  };
  assert!(by_id(1)["result"]["serverInfo"].is_object());
  let mut tools = by_id(2)["result"]["tools"]
    .as_array()
    .expect("tools/list should return tools")
    .iter()
    .map(|tool| tool["name"].as_str().expect("tool name should be text"))
    .collect::<Vec<_>>();
  tools.sort_unstable();
  assert_eq!(tools, ["observe", "start_run"]);
  let started = &by_id(3)["result"]["structuredContent"];
  let observed = &by_id(4)["result"]["structuredContent"];
  assert_eq!(started["protocol_version"], 1);
  assert_eq!(started, observed);
  assert_eq!(
    started["actors"]
      .as_array()
      .expect("actors should be present")
      .len(),
    2
  );
}
