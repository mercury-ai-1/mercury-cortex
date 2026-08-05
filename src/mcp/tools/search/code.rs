//! MCP handler for `search/code` — search indexed file metadata.

use serde_json::Value;
use tracing::info;

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::session::SessionId;
use mercury_cortex_core::engine::SearchQuery;

/// Search indexed file metadata by query, path, purpose, or features.
///
/// **Parameters:**
/// - `query` (string, optional): Full-text search across path, purpose, and summary.
/// - `path` (string, optional): Filter by file path substring.
/// - `purpose` (string, optional): Filter by file purpose.
/// - `features` (array of strings, optional): Filter by feature tags.
/// - `limit` (number, optional): Maximum number of results.
pub async fn handle_search(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    info!(method = "search", "mcp request");
    let query: SearchQuery =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let results = ctx.engine.search(&query).await?;
    Ok(serde_json::json!({ "results": results }))
}
