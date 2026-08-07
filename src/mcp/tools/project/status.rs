//! MCP handler for `project/status`: current project state.

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::session::SessionId;
use serde_json::Value;
use tracing::info;

/// Get the status of the currently active project.
///
/// Returns the status of the active project, or `{"status":
/// "no_project_open"}` if no project is open.
pub async fn handle_project_status(
    ctx: McpContext,
    _session: SessionId,
    _params: Value,
) -> McpResult<Value> {
    info!(method = "project_status", "mcp request");
    match ctx.engine.project_status().await {
        Some(status) => Ok(serde_json::to_value(status).map_err(McpError::Json)?),
        None => Ok(serde_json::json!({ "status": "no_project_open" })),
    }
}
