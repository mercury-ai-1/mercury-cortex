//! MCP handler for `project/open` — open a project in the engine.

use std::path::PathBuf;

use serde_json::Value;
use tracing::info;

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::models::ProjectOpenParams;
use crate::mcp::session::SessionId;
use mercury_cortex_core::service::project::ProjectService;

/// Open a project in the engine for indexing.
pub async fn handle_open(ctx: McpContext, _session: SessionId, params: Value) -> McpResult<Value> {
    info!(method = "project/open", "mcp request");
    let p: ProjectOpenParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let info = ProjectService::get_project(&ctx.rt, &p.project_id)
        .await
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    if info.root_path != p.root {
        return Err(McpError::InvalidParams(format!(
            "root_path mismatch: stored is '{}', requested '{}'. \
             Run `mercury-cortex project` to update.",
            info.root_path, p.root
        )));
    }

    let root_path = PathBuf::from(&p.root);
    ctx.engine
        .set_project(p.project_id.clone(), root_path)
        .await;

    Ok(serde_json::json!({"status": "opened", "project_id": p.project_id}))
}
