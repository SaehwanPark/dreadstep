//! Local stdio MCP server entry point.

use dreadstep_mcp::DreadstepMcpServer;

#[tokio::main]
async fn main() {
  if let Err(error) = DreadstepMcpServer::serve_stdio(0).await {
    eprintln!("dreadstep-mcp: {error}");
    std::process::exit(1);
  }
}
