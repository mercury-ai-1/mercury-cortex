//! Shared helpers for MCP tool handlers: timing, sanitization, and error wrapping.

use std::time::Instant;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ErrorData};
use serde::Serialize;
use serde_json::Value;

use crate::mcp::context::McpContext;
use crate::mcp::error::McpResult;
use crate::mcp::handler::McpHandler;
use crate::mcp::session::SessionId;

/// Log timing for a tool request stage.
macro_rules! log_timing {
    ($tool:expr, $stage:expr, $start:expr) => {
        tracing::info!(
            tool = $tool,
            stage = $stage,
            elapsed_us = $start.elapsed().as_micros() as u64,
            "tool timing"
        );
    };
}
#[allow(unused_imports)]
pub(crate) use log_timing;

/// Strip control characters from JSON values (except newline/tab).
pub fn sanitize_value(v: &mut Value) {
    match v {
        Value::String(s) => {
            s.retain(|c| !c.is_control() || c == '\n' || c == '\t');
        }
        Value::Array(arr) => {
            for elem in arr.iter_mut() {
                sanitize_value(elem);
            }
        }
        Value::Object(obj) => {
            for val in obj.values_mut() {
                sanitize_value(val);
            }
        }
        _ => {}
    }
}

impl McpHandler {
    /// Execute a parameterless tool with timing and error wrapping.
    pub async fn run_tool<R>(
        &self,
        name: &'static str,
        handler: impl FnOnce(McpContext, SessionId, Value) -> R,
    ) -> Result<CallToolResult, ErrorData>
    where
        R: std::future::Future<Output = McpResult<Value>>,
    {
        let _start = Instant::now();
        log_timing!(name, "request_received", _start);
        let ctx = self.ctx.get().await.map_err(|e| e.to_error_data())?;
        log_timing!(name, "ctx_ready", _start);
        let result = handler(ctx, 0, serde_json::json!({}))
            .await
            .map_err(|e| e.to_error_data())?;
        log_timing!(name, "response", _start);
        Ok(CallToolResult::structured(result))
    }

    /// Execute a tool with typed parameters, including sanitization and timing.
    pub async fn run_tool_with_params<T, R>(
        &self,
        name: &'static str,
        raw_params: Parameters<T>,
        handler: impl FnOnce(McpContext, SessionId, Value) -> R,
    ) -> Result<CallToolResult, ErrorData>
    where
        T: Serialize,
        R: std::future::Future<Output = McpResult<Value>>,
    {
        let _start = Instant::now();
        log_timing!(name, "request_received", _start);
        let mut p = serde_json::to_value(raw_params.0)
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        sanitize_value(&mut p);
        log_timing!(name, "params_deserialized", _start);
        let ctx = self.ctx.get().await.map_err(|e| e.to_error_data())?;
        log_timing!(name, "ctx_ready", _start);
        let result = handler(ctx, 0, p).await.map_err(|e| e.to_error_data())?;
        log_timing!(name, "response", _start);
        Ok(CallToolResult::structured(result))
    }
}
