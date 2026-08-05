//! MCP handler for `file/metadata` — get indexed metadata for a file.

use crate::mcp::context::McpContext;
use crate::mcp::models::FileMetadataParams;
use crate::mcp::session::SessionId;
use crate::mcp::{McpError, McpResult};
use serde_json::Value;
use tracing::info;

/// Get indexed metadata for a specific file by its relative project path.
///
/// Returns the file's metadata record, or `{"status": "not_found"}` if the
/// file is not indexed.
pub async fn handle_get_file_metadata(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    info!(method = "get_file_metadata", "mcp request");
    let p: FileMetadataParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    match ctx.engine.get_file_metadata(&p.path).await {
        Some(entry) => Ok(serde_json::to_value(entry).map_err(McpError::Json)?),
        None => Ok(serde_json::json!({ "status": "not_found" })),
    }
}
