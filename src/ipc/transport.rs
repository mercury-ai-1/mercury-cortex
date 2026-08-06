//! Connection pool for IPC connections (Unix socket or TCP loopback).
//!
//! Provides a [`ConnectionPool`] that limits concurrent connections via a
//! [`tokio::sync::Semaphore`] and reuses idle connections through an
//! internal [`VecDeque`].  Callers obtain a [`PooledConnection`] via
//! [`get_connection`] and return it via [`return_connection`].

use std::collections::VecDeque;
use std::sync::Arc;

use serde_json::Value;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

use super::client::ClientError;
use super::net::{self, Endpoint, IpcStream};
use super::protocol::{
    CODE_INVALID_VERSION, IpcFailure, IpcRequest, IpcSuccess, PROTOCOL_VERSION, validate_version,
};

/// An individual IPC connection wrapping a buffered, platform-agnostic stream.
pub(crate) struct IpcConnection {
    reader: BufReader<IpcStream>,
    /// True when this connection was popped from the idle pool rather than
    /// freshly opened; used to decide stale-connection retry.
    reused: bool,
}

/// Timeout for connecting to the IPC endpoint.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for writing a request to the IPC endpoint.
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
/// Timeout for reading a response from the IPC endpoint.
const READ_TIMEOUT: Duration = Duration::from_secs(10);

impl IpcConnection {
    pub async fn new(endpoint: &Endpoint) -> Result<Self, ClientError> {
        let stream = timeout(CONNECT_TIMEOUT, net::connect(endpoint))
            .await
            .map_err(|_| ClientError::ConnectionFailed("connect timed out".into()))?
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;
        Ok(Self {
            reader: BufReader::new(stream),
            reused: false,
        })
    }

    /// Whether this connection was popped from the idle pool rather than
    /// freshly opened.
    pub(crate) fn reused(&self) -> bool {
        self.reused
    }

    pub async fn call_raw(&mut self, method: &str, params: Value) -> Result<Value, ClientError> {
        let request = IpcRequest {
            version: PROTOCOL_VERSION,
            id: uuid::Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        };

        let request_line = serde_json::to_string(&request)
            .map_err(|e| ClientError::Serialization(e.to_string()))?;

        let mut buf = request_line;
        buf.push('\n');
        timeout(
            WRITE_TIMEOUT,
            self.reader.get_mut().write_all(buf.as_bytes()),
        )
        .await
        .map_err(|_| ClientError::ConnectionFailed("write timed out".into()))?
        .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;
        timeout(WRITE_TIMEOUT, self.reader.get_mut().flush())
            .await
            .map_err(|_| ClientError::ConnectionFailed("flush timed out".into()))?
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        let mut response_line = String::new();
        timeout(READ_TIMEOUT, self.reader.read_line(&mut response_line))
            .await
            .map_err(|_| ClientError::ReadTimeout)?
            .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

        if response_line.is_empty() {
            return Err(ClientError::ConnectionFailed("Empty response".into()));
        }

        if let Ok(success) = serde_json::from_str::<IpcSuccess<Value>>(&response_line) {
            if !validate_version(success.version) {
                return Err(ClientError::RequestFailed {
                    code: CODE_INVALID_VERSION.into(),
                    message: format!("unsupported protocol version {}", success.version),
                });
            }
            return Ok(success.result);
        }

        if let Ok(failure) = serde_json::from_str::<IpcFailure>(&response_line) {
            if !validate_version(failure.version) {
                return Err(ClientError::RequestFailed {
                    code: CODE_INVALID_VERSION.into(),
                    message: format!("unsupported protocol version {}", failure.version),
                });
            }
            return Err(ClientError::RequestFailed {
                code: failure.error.code,
                message: failure.error.message,
            });
        }

        Err(ClientError::Serialization(
            "Unrecognized response format".into(),
        ))
    }
}

/// A pool of reusable [`IpcConnection`]s with a maximum concurrency limit.
pub(crate) struct ConnectionPool {
    endpoint: Endpoint,
    connections: Arc<Mutex<VecDeque<IpcConnection>>>,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    /// Create a new pool for the given `endpoint` with a maximum of 10
    /// concurrent connections.
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            connections: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(10)),
        }
    }

    fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    fn semaphore(&self) -> &Arc<Semaphore> {
        &self.semaphore
    }

    fn connections(&self) -> &Arc<Mutex<VecDeque<IpcConnection>>> {
        &self.connections
    }
}

/// A connection checked out from the pool, together with its semaphore
/// permit.  The permit is released back when this value is dropped.
pub(crate) struct PooledConnection {
    pub(crate) conn: IpcConnection,
    // The permit's Drop enforces the pool's concurrency cap; holding it for
    // the lifetime of the checkout is the entire point.
    #[allow(dead_code)]
    permit: OwnedSemaphorePermit,
}

/// Obtain a connection from the pool.
///
/// Acquires a semaphore permit first, then either reuses an idle connection
/// or creates a fresh one.
pub(crate) async fn get_connection(pool: &ConnectionPool) -> Result<PooledConnection, ClientError> {
    let permit = pool
        .semaphore()
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;

    let conn = {
        let mut guard = pool.connections().lock().await;
        guard.pop_front()
    };

    let conn = match conn {
        Some(mut c) => {
            c.reused = true;
            c
        }
        None => IpcConnection::new(pool.endpoint()).await?,
    };

    Ok(PooledConnection { conn, permit })
}

/// Obtain a connection that is guaranteed freshly opened, bypassing the idle
/// pool. Used to retry once after a stale pooled connection fails.
pub(crate) async fn get_fresh_connection(
    pool: &ConnectionPool,
) -> Result<PooledConnection, ClientError> {
    let permit = pool
        .semaphore()
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| ClientError::ConnectionFailed(e.to_string()))?;
    let conn = IpcConnection::new(pool.endpoint()).await?;
    Ok(PooledConnection { conn, permit })
}

/// Return a connection to the pool for reuse.
///
/// The connection is pushed back into the idle pool so the next caller
/// can reuse it.  The semaphore permit is released when the
/// [`PooledConnection`] is dropped.
pub(crate) async fn return_connection(pool: &ConnectionPool, pooled: PooledConnection) {
    let PooledConnection { conn, permit: _ } = pooled;
    let mut guard = pool.connections.lock().await;
    guard.push_back(conn);
}
