//! MCP handler for `metadata/import` — import staged AI-generated metadata.

use crate::mcp::context::McpContext;
use crate::mcp::error::McpResult;
use crate::mcp::session::SessionId;
use serde_json::Value;
use tracing::info;

/// Import staged AI-generated metadata from `.mercury-cortex/temp/`.
pub async fn handle_import_metadata(
    ctx: McpContext,
    _session: SessionId,
    _params: Value,
) -> McpResult<Value> {
    info!("import_metadata mcp request");
    ctx.engine.count_indexed_files().await?;
    let results = ctx.engine.submit_metadata().await?;
    let indexed_files = ctx.engine.count_indexed_files().await?;
    Ok(serde_json::json!({
        "indexed_files": indexed_files,
        "results": results,
    }))
}
