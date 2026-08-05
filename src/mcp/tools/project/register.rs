//! MCP handler for `project/register` — register a project in Mercury Cortex.

use std::path::PathBuf;

use serde_json::Value;

use crate::mcp::context::McpContext;
use crate::mcp::models::ProjectRegisterParams;
use crate::mcp::session::SessionId;
use crate::mcp::{McpError, McpResult};
use mercury_cortex_core::service::project::{ProjectService, RegisterParams};
use mercury_cortex_core::service::scaffold;

/// Register a new project with Mercury Cortex.
///
/// Creates the `.mercury-cortex/` directory structure, registers the project
/// in the database, opens it in the engine, and returns the project identity.
///
/// This handler is safe to call multiple times — it will reuse an existing
/// registration when the directory is already initialised.
pub async fn handle_register(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    let p: ProjectRegisterParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let root_str = &p.root;

    let root = PathBuf::from(root_str);

    // Read existing project_id from config, if any
    let config_path = root.join(".mercury-cortex").join("config.json");
    let config_project_id = if config_path.exists() {
        let cp = config_path.clone();
        tokio::task::spawn_blocking(move || scaffold::read_config_project_id(&cp))
            .await
            .map_err(|e| McpError::Transport(format!("join error: {e}")))?
            .map_err(|e| McpError::InvalidParams(format!("cannot read config: {e}")))?
    } else {
        // Create the .mercury-cortex directory structure (scaffold helpers
        // assume the directory exists).
        let root_for_create = root.clone();
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(root_for_create.join(".mercury-cortex"))
        })
        .await
        .map_err(|e| McpError::Transport(format!("join error: {e}")))?
        .map_err(|e| McpError::InvalidParams(format!("failed to create .mercury-cortex: {e}")))?;
        None
    };

    // Derive project metadata from the root path
    let project_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let slug = scaffold::slugify(&project_name);

    // Register via the service (handles both create and update)
    let reg_params = RegisterParams {
        config_project_id,
        name: project_name,
        slug,
        root_path: p.root.clone(),
    };

    let result = ProjectService::register(&ctx.rt, reg_params)
        .await
        .map_err(|e| McpError::InvalidParams(e.to_string()))?;

    let project_id = result.project_id;

    // Write config if this is a new registration
    if !config_path.exists() {
        let project_id = project_id.clone();
        let cp = config_path.clone();
        tokio::task::spawn_blocking(move || scaffold::write_config(&cp, &project_id))
            .await
            .map_err(|e| McpError::Transport(format!("join error: {e}")))?
            .map_err(|e| McpError::InvalidParams(format!("failed to write config: {e}")))?;
    }

    // Open the project in the engine
    ctx.engine
        .set_project(project_id.clone(), root.clone())
        .await;

    Ok(serde_json::json!({
        "status": "registered",
        "project_id": project_id,
    }))
}
