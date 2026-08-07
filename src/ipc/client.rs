use std::sync::Arc;

use serde_json::Value;

use mercury_cortex_core::runtime::RuntimeConfig;
use mercury_cortex_core::service::profile::{ProfileData, UpsertParams};
use mercury_cortex_core::service::project::{RegisterParams, RegisterResult};

use super::net::Endpoint;
use super::transport::{ConnectionPool, get_connection, get_fresh_connection, return_connection};

/// Error returned by [`RuntimeClient`] operations.
#[derive(Debug)]
pub(crate) enum ClientError {
    ConnectionFailed(String),
    RequestFailed {
        code: String,
        message: String,
    },
    Serialization(String),
    /// The daemon did not respond within the read timeout. The request may
    /// already have been executed, so this must not be treated as staleness.
    ReadTimeout,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::ConnectionFailed(msg) => write!(f, "Connection failed: {msg}"),
            ClientError::RequestFailed { code, message } => write!(f, "[{code}] {message}"),
            ClientError::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            ClientError::ReadTimeout => write!(f, "Read timed out"),
        }
    }
}

impl std::error::Error for ClientError {}

/// Client for communicating with a running Mercury Cortex runtime over the
/// platform IPC endpoint (Unix socket or TCP loopback).
///
/// Uses a [`ConnectionPool`] internally to reuse connections and limit
/// concurrency.
pub(crate) struct RuntimeClient {
    pool: Arc<ConnectionPool>,
}

impl RuntimeClient {
    /// Probe the runtime endpoint. Returns `None` if no runtime is running.
    pub async fn try_connect() -> Option<Self> {
        let config = RuntimeConfig::new().ok()?;
        let endpoint = Endpoint::from_socket_path(&config.socket_path);

        if !endpoint.probe() {
            return None;
        }

        let pool = Arc::new(ConnectionPool::new(endpoint));

        // Ping the runtime to confirm it's alive
        let (conn, result) = {
            let Ok(mut pooled) = get_connection(&pool).await else {
                return None;
            };
            let r = pooled.conn.call_raw("runtime/ping", Value::Null).await;
            (pooled, r)
        };
        drop(conn);

        match result {
            Ok(_) => Some(Self { pool }),
            Err(_) => None,
        }
    }

    /// Low-level RPC call (private), which obtains a connection from the pool,
    /// performs the RPC, and returns the connection on success.
    async fn call_raw(&self, method: &str, params: Value) -> Result<Value, ClientError> {
        let mut pooled = get_connection(&self.pool).await?;
        let reused = pooled.conn.reused();
        let result = pooled.conn.call_raw(method, params.clone()).await;

        if result.is_ok() {
            return_connection(&self.pool, pooled).await;
            return result;
        }

        // A reused connection may have gone stale while idle in the pool.
        // Drop it (do not return it) and retry once on a fresh connection.
        // Read timeouts are deliberately excluded: the request was already
        // sent, so the daemon may have executed it and retrying could
        // double-write for mutating RPCs.
        if reused && matches!(&result, Err(ClientError::ConnectionFailed(_))) {
            let err = match &result {
                Err(e) => e,
                Ok(_) => unreachable!("the stale-retry guard above only matches Err"),
            };
            tracing::warn!(
                method,
                error = %err,
                "stale IPC connection; retrying once on a fresh connection"
            );
            drop(pooled);
            let mut fresh = get_fresh_connection(&self.pool).await?;
            let retry = fresh.conn.call_raw(method, params.clone()).await;
            if retry.is_ok() {
                return_connection(&self.pool, fresh).await;
            }
            return retry;
        }

        result
    }

    // ── Typed methods ──

    pub async fn project_register(
        &self,
        params: RegisterParams,
    ) -> Result<RegisterResult, ClientError> {
        let val =
            serde_json::to_value(params).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let result = self.call_raw("project/register", val).await?;
        serde_json::from_value(result).map_err(|e| ClientError::Serialization(e.to_string()))
    }

    pub async fn profile_get(&self) -> Result<Option<ProfileData>, ClientError> {
        let result = self.call_raw("profile/get", Value::Null).await?;
        serde_json::from_value(result).map_err(|e| ClientError::Serialization(e.to_string()))
    }

    pub async fn profile_upsert(&self, params: UpsertParams) -> Result<String, ClientError> {
        let val =
            serde_json::to_value(params).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let result = self.call_raw("profile/upsert", val).await?;
        if let Some(s) = result.as_str() {
            return Ok(s.to_string());
        }
        serde_json::from_value(result).map_err(|e| ClientError::Serialization(e.to_string()))
    }
}
