//! MCP handler for `index/paths`: list indexed file paths for the active project.
//!
//! Returns the relative paths of all indexed files in the active project.
//! Used by the AI tool during re-run gap-fill to determine which files
//! already have metadata. Intentionally separate from `search/code`, whose
//! responsibility is reusable code discovery rather than index management.

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::session::SessionId;
use serde_json::Value;

/// List all indexed file paths for the active project.
///
/// **Parameters:** None (uses the active project session).
///
/// **Returns:**
/// - `project_id` (string): The active project's record ID.
/// - `paths` (array of strings): Relative paths of all indexed files.
/// - `count` (number): Number of paths returned.
pub async fn handle_index_paths(
    ctx: McpContext,
    _session: SessionId,
    _params: Value,
) -> McpResult<Value> {
    let paths = ctx
        .engine
        .list_indexed_paths()
        .await
        .map_err(|e| McpError::InvalidParams(format!("failed to list indexed paths: {e}")))?;

    let project_id = {
        let ctx_guard = ctx.engine.context().read().await;
        ctx_guard
            .project_id()
            .ok_or_else(|| McpError::InvalidParams("no active project".into()))?
            .to_string()
    };

    Ok(serde_json::json!({
        "project_id": project_id,
        "paths": paths,
        "count": paths.len(),
    }))
}
