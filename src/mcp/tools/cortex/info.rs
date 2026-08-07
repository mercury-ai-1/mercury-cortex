//! MCP handler for `cortex/info`: engine version and status.

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::session::SessionId;
use serde_json::Value;
use tracing::info;

/// Get the engine version and running status.
pub async fn handle_info(ctx: McpContext, _session: SessionId, _params: Value) -> McpResult<Value> {
    info!(method = "info", "mcp request");
    let info = ctx.engine.info().await;
    serde_json::to_value(info).map_err(McpError::Json)
}
