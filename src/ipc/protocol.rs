use serde::{Deserialize, Serialize};
use serde_json::Value;

use mercury_cortex_core::service::ServiceError;

/// The current IPC wire protocol version.
pub(crate) const PROTOCOL_VERSION: u32 = 1;

/// Error code used when a peer speaks an unsupported protocol version.
pub(crate) const CODE_INVALID_VERSION: &str = "INVALID_VERSION";

/// IPC request: transport-agnostic, one per connection.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IpcRequest<T = Value> {
    pub version: u32,
    pub id: String,
    pub method: String,
    pub params: T,
}

/// Successful IPC response.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IpcSuccess<T = Value> {
    pub version: u32,
    pub id: String,
    pub result: T,
}

/// Failed IPC response.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IpcFailure {
    pub version: u32,
    pub id: String,
    pub error: IpcError,
}

/// Structured error detail.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IpcError {
    pub code: String,
    pub message: String,
    /// Optional recovery guidance. `#[serde(default)]` keeps the wire shape
    /// additive: older peers that omit the field still deserialize.
    #[serde(default)]
    pub recovery: Option<String>,
}

impl IpcFailure {
    pub(crate) fn new(id: &str, code: &str, message: String) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id: id.to_string(),
            error: IpcError {
                code: code.to_string(),
                message,
                recovery: None,
            },
        }
    }

    /// Build a failure with optional recovery guidance.
    pub(crate) fn with_recovery(
        id: &str,
        code: &str,
        message: String,
        recovery: impl Into<String>,
    ) -> Self {
        let mut failure = Self::new(id, code, message);
        failure.error.recovery = Some(recovery.into());
        failure
    }
}

/// Returns `true` when `version` matches the current [`PROTOCOL_VERSION`].
#[must_use]
pub(crate) fn validate_version(version: u32) -> bool {
    version == PROTOCOL_VERSION
}

/// Map a `ServiceError` to an `IpcFailure`.
impl From<(&str, ServiceError)> for IpcFailure {
    fn from((id, err): (&str, ServiceError)) -> Self {
        let code = match &err {
            ServiceError::NotFound(_) => "NOT_FOUND",
            ServiceError::Database(_) => "DATABASE_ERROR",
            ServiceError::Validation(_) => "VALIDATION_ERROR",
            ServiceError::Internal(_) => "INTERNAL_ERROR",
            ServiceError::RuntimeNotReady(_) => "RUNTIME_NOT_READY",
        };
        IpcFailure::new(id, code, err.to_string())
    }
}
