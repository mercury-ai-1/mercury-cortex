use mercury_cortex::mcp::session::{Session, SessionManager};
use std::path::PathBuf;

#[tokio::test]
async fn test_session_new() {
    let session = Session::new("client-1");
    assert_eq!(session.client_id, "client-1");
    assert!(!session.has_project());
}

#[tokio::test]
async fn test_session_set_project() {
    let mut session = Session::new("client-1");
    session.set_project("proj-1".into(), PathBuf::from("/tmp/test"));
    assert!(session.has_project());
    assert_eq!(session.project_id(), Some("proj-1"));
}

#[tokio::test]
async fn test_session_clear_project() {
    let mut session = Session::new("client-1");
    session.set_project("proj-1".into(), PathBuf::from("/tmp/test"));
    session.clear_project();
    assert!(!session.has_project());
    assert!(session.project_id().is_none());
}

#[tokio::test]
async fn test_manager_register_get() {
    let manager = SessionManager::new();
    let session = Session::new("client-1");
    let id = session.id;
    manager.register(session).await;
    let retrieved = manager.get(id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().client_id, "client-1");
}

#[tokio::test]
async fn test_manager_remove() {
    let manager = SessionManager::new();
    let session = Session::new("client-1");
    let id = session.id;
    manager.register(session).await;
    manager.remove(id).await;
    assert!(manager.get(id).await.is_none());
}

#[tokio::test]
async fn test_manager_active_count() {
    let manager = SessionManager::new();
    assert_eq!(manager.active_count().await, 0);
    manager.register(Session::new("a")).await;
    manager.register(Session::new("b")).await;
    assert_eq!(manager.active_count().await, 2);
}

#[tokio::test]
async fn test_manager_get_unknown() {
    let manager = SessionManager::new();
    assert!(manager.get(999).await.is_none());
}
