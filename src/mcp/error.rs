use thiserror::Error;

use mercury_cortex_core::engine::error::EngineError;

/// Errors that can occur during MCP handler execution.
#[derive(Error, Debug)]
pub enum McpError {
    /// JSON serialization or deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Transport-level failure (e.g. spawning a blocking task).
    #[error("transport error: {0}")]
    Transport(String),
    /// Engine operation failed.
    #[error("engine error: {0}")]
    Engine(#[from] EngineError),
    /// Invalid or missing parameters in a handler request.
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// Engine is not available within the readiness timeout.
    #[error("engine not ready: {0}")]
    NotReady(String),
}

/// Convenience alias for handler results.
pub type McpResult<T> = Result<T, McpError>;

impl McpError {
    /// Map to the rmcp error category MCP clients can act on:
    /// caller-controllable failures are `invalid_params`; everything else is
    /// an internal error.
    #[must_use]
    pub fn to_error_data(&self) -> rmcp::model::ErrorData {
        match self {
            McpError::InvalidParams(msg) => {
                rmcp::model::ErrorData::invalid_params(msg.to_string(), None)
            }
            McpError::Json(e) => rmcp::model::ErrorData::invalid_params(e.to_string(), None),
            McpError::Engine(e) => rmcp::model::ErrorData::internal_error(e.to_string(), None),
            McpError::Transport(msg) | McpError::NotReady(msg) => {
                rmcp::model::ErrorData::internal_error(msg.clone(), None)
            }
        }
    }
}
