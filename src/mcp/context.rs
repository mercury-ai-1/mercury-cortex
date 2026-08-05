//! Shared context passed to every MCP handler invocation.

use std::sync::Arc;

use tokio::sync::watch;

use mercury_cortex_core::engine::KnowledgeEngine;
use mercury_cortex_core::runtime::RuntimeContext;

use crate::mcp::error::{McpError, McpResult};

/// Context made available to every MCP handler.
///
/// Holds a reference to the engine so that handlers can interact with the
/// knowledge base without owning or managing it.
#[derive(Clone)]
pub struct McpContext {
    /// The running knowledge engine instance.
    pub engine: Arc<KnowledgeEngine>,
    /// The full runtime context (database, engine, config).
    pub(crate) rt: Arc<RuntimeContext>,
}

impl McpContext {
    /// Create a new context with the given engine and runtime context.
    pub fn new(engine: Arc<KnowledgeEngine>, rt: Arc<RuntimeContext>) -> Self {
        Self { engine, rt }
    }
}

/// A lazily-initialized [`McpContext`] that allows the MCP server to start
/// responding to protocol handshakes (`initialize`, `ping`, `tools/list`)
/// immediately, while deferring engine initialization to the first tool call.
///
/// When a tool handler calls [`get`](LazyContext::get), it blocks until the
/// engine is ready (or returns an error if initialization fails).
///
/// `get` is bounded by a readiness timeout so a missing engine can never
/// hang a tool call indefinitely.
#[derive(Clone)]
pub struct LazyContext {
    rx: watch::Receiver<Option<McpContext>>,
    tx: watch::Sender<Option<McpContext>>,
    timeout: std::time::Duration,
}

/// Default maximum time a tool call waits for engine initialization.
const ENGINE_READY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

impl LazyContext {
    /// Create a new context with no engine yet and the default readiness timeout.
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_timeout(ENGINE_READY_TIMEOUT)
    }

    /// Create a new context with a caller-specified readiness timeout
    /// (primarily for tests).
    #[must_use]
    pub fn new_with_timeout(timeout: std::time::Duration) -> Self {
        let (tx, rx) = watch::channel(None);
        Self { rx, tx, timeout }
    }

    /// Wait for the engine to become available and return the context.
    ///
    /// If the engine has already been initialized, returns immediately.
    /// Otherwise blocks until [`set`](LazyContext::set) is called or
    /// `ENGINE_READY_TIMEOUT` elapses, then returns
    /// [`McpError::NotReady`](crate::mcp::error::McpError::NotReady).
    pub async fn get(&self) -> McpResult<McpContext> {
        let start = std::time::Instant::now();
        let mut rx = self.rx.clone();
        if self.rx.borrow().is_none() {
            tracing::info!("lazy_context waiting for engine init");
        }
        let wait = async {
            loop {
                if let Some(ref ctx) = *rx.borrow_and_update() {
                    return ctx.clone();
                }
                rx.changed().await.ok();
            }
        };
        match tokio::time::timeout(self.timeout, wait).await {
            Ok(ctx) => {
                let elapsed = start.elapsed();
                if elapsed > std::time::Duration::from_millis(1) {
                    tracing::info!(
                        elapsed_us = elapsed.as_micros() as u64,
                        "lazy_context ready"
                    );
                }
                Ok(ctx)
            }
            Err(_) => Err(McpError::NotReady(format!(
                "engine not ready within {}s",
                self.timeout.as_secs()
            ))),
        }
    }

    /// Populate the context with a fully initialized engine.
    ///
    /// Wakes all waiters blocked on [`get`](LazyContext::get).
    pub fn set(&self, ctx: McpContext) {
        self.tx.send_replace(Some(ctx));
    }
}

impl Default for LazyContext {
    fn default() -> Self {
        Self::new()
    }
}
