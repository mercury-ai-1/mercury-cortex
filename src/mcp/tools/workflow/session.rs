//! MCP handler for `workflow/session` — get session context and step list.

use serde_json::Value;
use tracing::info;

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::models::WorkflowSessionParams;
use crate::mcp::session::SessionId;
use crate::mcp::tools::prompts;

/// Return the workflow session state: engine info, active project, and step list.
pub async fn handle_workflow_session(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    let p: WorkflowSessionParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let mode = &p.mode;

    info!(method = "workflow/session", mode = %mode, "mcp request");

    if mode != "dev" && mode != "init" {
        return Err(McpError::InvalidParams(format!(
            "unknown mode: {mode}. Supported modes: dev, init"
        )));
    }

    let engine = serde_json::to_value(ctx.engine.info().await).map_err(McpError::Json)?;

    let project = match ctx.engine.project_status().await {
        Some(s) => serde_json::to_value(s).map_err(McpError::Json)?,
        None => Value::Null,
    };

    let steps = prompts::registry::list_steps(mode);

    let resp = serde_json::json!({
        "mode": mode,
        "engine": engine,
        "project": project,
        "workflow": {
            "steps": steps,
        },
    });

    Ok(resp)
}
