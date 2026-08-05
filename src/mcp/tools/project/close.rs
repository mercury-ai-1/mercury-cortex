//! MCP handler for `project/close` — close the active project.

use serde_json::Value;
use tracing::info;

use crate::mcp::context::McpContext;
use crate::mcp::error::McpResult;
use crate::mcp::session::SessionId;

/// Close the currently active project.
pub async fn handle_close(
    ctx: McpContext,
    _session: SessionId,
    _params: Value,
) -> McpResult<Value> {
    info!(method = "project/close", "mcp request");
    ctx.engine.clear_project().await;
    Ok(serde_json::json!({"status": "closed"}))
}
