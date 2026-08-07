//! MCP handler for `workflow/step`: get instructions for a specific step.

use serde_json::Value;
use tracing::info;

use crate::mcp::context::McpContext;
use crate::mcp::error::{McpError, McpResult};
use crate::mcp::models::WorkflowStepParams;
use crate::mcp::session::SessionId;
use crate::mcp::tools::prompts;

/// Return the title and content for a specific workflow step.
pub async fn handle_workflow_step(
    _ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    let p: WorkflowStepParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let mode = &p.mode;
    let step = p.step as usize;

    info!(method = "workflow/step", mode = %mode, step = %step, "mcp request");

    if mode != "dev" && mode != "init" {
        return Err(McpError::InvalidParams(format!(
            "unknown mode: {mode}. Supported modes: dev, init"
        )));
    }

    let steps = prompts::registry::list_steps(mode);
    let max_step = steps
        .iter()
        .filter_map(|s| s.get("number").and_then(serde_json::Value::as_u64))
        .max()
        .unwrap_or(0) as usize;

    let (title, content) = prompts::registry::get_step(mode, step).ok_or_else(|| {
        McpError::InvalidParams(format!(
            "unknown step {step} for mode '{mode}' (valid: 0-{max_step})"
        ))
    })?;

    Ok(serde_json::json!({
        "name": format!("{mode}:step{step}"),
        "title": title,
        "content": content,
    }))
}
