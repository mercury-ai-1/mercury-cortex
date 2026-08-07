//! MCP handler for `project/update`: save AI-generated project metadata.

use serde_json::Value;

use crate::mcp::context::McpContext;
use crate::mcp::models::ProjectUpdateParams;
use crate::mcp::session::SessionId;
use crate::mcp::{McpError, McpResult};
use mercury_cortex_core::service::project::{ProjectService, UpdateMetadataParams};

/// Save AI-generated project metadata to the project record.
///
/// The AI assistant analyses the project and sends the detected language,
/// framework, build system, package manager, and technology stack.  Mercury
/// Cortex stores only the fields the AI explicitly provides; omitted fields
/// are not overwritten in the database.
pub async fn handle_update(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    let p: ProjectUpdateParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let params = UpdateMetadataParams {
        project_id: p.project_id,
        language: p.metadata.language,
        framework: p.metadata.framework,
    };

    ProjectService::update_metadata(&ctx.rt, params)
        .await
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    Ok(serde_json::json!({ "status": "updated" }))
}
