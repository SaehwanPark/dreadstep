//! Minimal local stdio MCP server for versioned observation and typed player actions.

use std::{
  error::Error,
  sync::{Arc, Mutex, MutexGuard},
};

use rmcp::{
  ErrorData as McpError, Json, ServerHandler, ServiceExt,
  handler::server::{router::tool::ToolRouter, wrapper::Parameters},
  tool, tool_handler, tool_router,
  transport::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Session, SessionError, SessionOutput};
use dreadstep_protocol::{CommandRequest, WorldSnapshot};

/// Parameters for the `start_run` MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StartRunParams {
  /// Explicit deterministic session seed.
  pub seed: u64,
}

/// Parameters for the `act` MCP tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ActParams {
  /// One explicitly tagged protocol player command.
  pub request: CommandRequest,
}

/// Minimal MCP server exposing the existing in-memory session over local stdio.
#[derive(Debug, Clone)]
pub struct DreadstepMcpServer {
  session: Arc<Mutex<Session>>,
  #[expect(dead_code, reason = "tool_handler macro accesses this router field")]
  tool_router: ToolRouter<Self>,
}

impl DreadstepMcpServer {
  /// Creates a server with a fixed scenario at the requested initial seed.
  ///
  /// # Errors
  ///
  /// Returns a session error when the fixed scenario cannot be constructed.
  pub fn new(seed: u64) -> Result<Self, SessionError> {
    Ok(Self {
      session: Arc::new(Mutex::new(Session::start_run(seed)?)),
      tool_router: Self::tool_router(),
    })
  }

  fn lock_session(&self) -> Result<MutexGuard<'_, Session>, McpError> {
    self
      .session
      .lock()
      .map_err(|_| McpError::internal_error("MCP session lock is poisoned", None))
  }

  /// Runs this server over the official local stdio transport until shutdown.
  ///
  /// # Errors
  ///
  /// Returns an error when the fixed session cannot start, the stdio transport cannot be
  /// initialized, or the service terminates with a transport failure.
  pub async fn serve_stdio(seed: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
    let server = Self::new(seed)?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
  }
}

#[tool_router]
impl DreadstepMcpServer {
  /// Starts or replaces the in-memory fixed scenario with an explicit seed.
  #[tool(
    name = "start_run",
    description = "Start a deterministic fixed Dreadstep scenario and return its snapshot."
  )]
  pub async fn start_run(
    &self,
    Parameters(params): Parameters<StartRunParams>,
  ) -> Result<Json<WorldSnapshot>, McpError> {
    let session = Session::start_run(params.seed)
      .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    let snapshot = session.observe();
    *self.lock_session()? = session;
    Ok(Json(snapshot))
  }

  /// Returns the current versioned world snapshot without mutating the session.
  #[tool(
    name = "observe",
    description = "Observe the current deterministic Dreadstep world snapshot."
  )]
  pub async fn observe(&self) -> Result<Json<WorldSnapshot>, McpError> {
    Ok(Json(self.lock_session()?.observe()))
  }

  /// Executes one typed player action and returns semantic evidence.
  #[tool(
    name = "act",
    description = "Execute one typed player action and return event and snapshot evidence."
  )]
  pub async fn act(
    &self,
    Parameters(params): Parameters<ActParams>,
  ) -> Result<Json<SessionOutput>, McpError> {
    self
      .lock_session()?
      .act(params.request)
      .map(Json)
      .map_err(|error| McpError::invalid_params(error.to_string(), None))
  }
}

#[tool_handler]
impl ServerHandler for DreadstepMcpServer {}
