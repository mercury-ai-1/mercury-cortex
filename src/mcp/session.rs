//! Session management for MCP client connections.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Opaque identifier for an active session.
pub type SessionId = u64;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Default session time-to-live: 1 hour.
pub const DEFAULT_SESSION_TTL: Duration = Duration::from_hours(1);

/// An active MCP client session with optional project association.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub client_id: String,
    pub connected_at: Instant,
    /// When this session was created, for TTL checks.
    pub created_at: Instant,
    project_id: Option<String>,
    project_root: Option<PathBuf>,
}

impl Session {
    /// Create a new session with an auto-generated ID.
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::SeqCst),
            client_id: client_id.into(),
            connected_at: Instant::now(),
            created_at: Instant::now(),
            project_id: None,
            project_root: None,
        }
    }

    /// Returns `true` if the session has exceeded the given TTL.
    #[must_use]
    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() >= ttl
    }

    /// Associate this session with a project.
    pub fn set_project(&mut self, pid: String, root: PathBuf) {
        self.project_id = Some(pid);
        self.project_root = Some(root);
    }

    /// Remove project association from this session.
    pub fn clear_project(&mut self) {
        self.project_id = None;
        self.project_root = None;
    }

    /// Whether this session has an active project association.
    #[must_use]
    pub fn has_project(&self) -> bool {
        self.project_id.is_some()
    }

    /// The project ID, if set.
    #[must_use]
    pub fn project_id(&self) -> Option<&str> {
        self.project_id.as_deref()
    }

    /// The project root path, if set.
    #[must_use]
    pub fn project_root(&self) -> Option<&PathBuf> {
        self.project_root.as_ref()
    }
}

/// Manages active MCP sessions.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: RwLock<HashMap<SessionId, Session>>,
}

impl SessionManager {
    /// Create an empty session manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new session.
    pub async fn register(&self, session: Session) {
        self.sessions.write().await.insert(session.id, session);
    }

    /// Look up a session by ID.
    pub async fn get(&self, id: SessionId) -> Option<Session> {
        self.sessions.read().await.get(&id).cloned()
    }

    /// Remove and drop a session.
    pub async fn remove(&self, id: SessionId) {
        self.sessions.write().await.remove(&id);
    }

    /// Number of currently active sessions.
    pub async fn active_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Return a snapshot of all active sessions.
    pub async fn all_sessions(&self) -> Vec<Session> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// Remove sessions that have exceeded the given TTL.
    ///
    /// Returns the number of expired sessions that were removed.
    pub async fn cleanup_expired(&self, ttl: Duration) -> usize {
        let mut map = self.sessions.write().await;
        let before = map.len();
        map.retain(|_, s| !s.is_expired(ttl));
        before - map.len()
    }
}
