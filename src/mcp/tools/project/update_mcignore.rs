//! MCP handler for `project/update_mcignore`: append ignore patterns from AI.
//!
//! The AI tool analyzes the project structure and determines which patterns
//! to add. The handler validates the request and writes the patterns; it
//! does not analyze the project or make decisions about what to ignore.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::mcp::context::McpContext;
use crate::mcp::models::UpdateMcIgnoreParams;
use crate::mcp::session::SessionId;
use crate::mcp::{McpError, McpResult};

/// Append AI-detected ignore patterns to `.mcignore`.
///
/// The AI tool analyzes the project structure and determines which patterns
/// to add. The handler validates the request and writes the patterns; it
/// does not analyze the project or make decisions about what to ignore.
pub async fn handle_update_mcignore(
    ctx: McpContext,
    _session: SessionId,
    params: Value,
) -> McpResult<Value> {
    let p: UpdateMcIgnoreParams =
        serde_json::from_value(params).map_err(|e| McpError::InvalidParams(e.to_string()))?;
    let root_str = &p.root;
    let patterns = p.patterns;

    if patterns.is_empty() {
        return Err(McpError::InvalidParams(
            "patterns must be a non-empty array".into(),
        ));
    }

    // Validate each pattern is non-empty
    for p in &patterns {
        if p.trim().is_empty() {
            return Err(McpError::InvalidParams(
                "patterns must not contain empty entries".into(),
            ));
        }
    }

    let root = PathBuf::from(root_str);
    let mcignore_path = root.join(".mercury-cortex").join(".mcignore");

    if !mcignore_path.exists() {
        return Err(McpError::InvalidParams(
            ".mcignore not found; run `mercury-cortex project` first".into(),
        ));
    }

    let updated = append_mcignore_patterns(&mcignore_path, &patterns)
        .await
        .map_err(|e| McpError::InvalidParams(format!("failed to update .mcignore: {e}")))?;

    if updated {
        let (project_id, root_path) = {
            let ctx_guard = ctx.engine.context().read().await;
            if let Some(pid) = ctx_guard.project_id() {
                (pid.to_string(), root.clone())
            } else {
                (String::new(), root.clone())
            }
        };

        if !project_id.is_empty() {
            ctx.engine.set_project(project_id, root_path).await;
        }
    }

    Ok(serde_json::json!({
        "updated": updated,
        "pattern_count": patterns.len(),
    }))
}

/// Append AI-provided patterns to `.mcignore`, skipping duplicates.
async fn append_mcignore_patterns(path: &Path, patterns: &[String]) -> Result<bool, anyhow::Error> {
    let existing_content = if path.exists() {
        let p = path.to_path_buf();
        tokio::task::spawn_blocking(move || std::fs::read_to_string(&p))
            .await
            .map_err(|e| anyhow::anyhow!("join error: {e}"))?
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?
    } else {
        String::new()
    };

    let existing_set: BTreeSet<String> = existing_content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let to_add: Vec<&str> = patterns
        .iter()
        .map(std::string::String::as_str)
        .filter(|p| !existing_set.contains(*p))
        .collect();

    if to_add.is_empty() {
        return Ok(false);
    }

    let mut new_content = existing_content;
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    for pattern in &to_add {
        new_content.push_str(pattern);
        new_content.push('\n');
    }

    let p = path.to_path_buf();
    tokio::task::spawn_blocking(move || std::fs::write(&p, &new_content))
        .await
        .map_err(|e| anyhow::anyhow!("join error: {e}"))?
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", path.display()))?;

    Ok(true)
}
