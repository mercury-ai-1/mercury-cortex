use mercury_cortex::mcp::McpError;
use mercury_cortex_core::engine::EngineError;

#[test]
fn test_mcp_error_from_engine() {
    let err: McpError = EngineError::NotRunning.into();
    assert!(err.to_string().contains("not running"));
}

#[test]
fn test_mcp_context_is_clone_send() {
    fn assert_props<T: Clone + Send>() {}
    assert_props::<mercury_cortex::mcp::context::McpContext>();
}

use std::time::Duration;

use mercury_cortex::mcp::context::{LazyContext, McpContext};
use mercury_cortex_core::db;
use mercury_cortex_core::engine::KnowledgeEngine;
use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::runtime::status::RuntimePhase;
use mercury_cortex_core::schema;
use rmcp::model::ErrorCode;
use std::sync::Arc;

async fn build_ready_context() -> McpContext {
    let tmp = tempfile::TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let db = db::initialize(&db_path).await.unwrap();
    schema::run_pending(&db).await.unwrap();
    let engine = KnowledgeEngine::new(db.clone());
    let rt = Arc::new(RuntimeContext::new_for_test());
    rt.set_database(db, RuntimePhase::DatabaseConnected);
    rt.set_engine(Arc::new(engine), RuntimePhase::Running);
    McpContext::new(rt.engine().unwrap(), rt)
}

#[tokio::test]
async fn lazy_context_returns_ctx_after_set() {
    let lazy = LazyContext::new();
    let ctx = build_ready_context().await;
    let task = tokio::spawn({
        let lazy = lazy.clone();
        async move {
            tokio::time::timeout(Duration::from_secs(5), lazy.get())
                .await
                .expect("get must not hang")
                .expect("set context must be returned")
        }
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    lazy.set(ctx);
    let got = task.await.unwrap();
    let _ = got.engine;
}

#[tokio::test]
async fn lazy_context_times_out_when_unset() {
    let lazy = LazyContext::new_with_timeout(Duration::from_millis(50));
    let err = tokio::time::timeout(Duration::from_secs(5), lazy.get())
        .await
        .expect("get must return within the bounded timeout")
        .err()
        .expect("unset context must time out into NotReady");
    assert!(matches!(err, McpError::NotReady(_)));
}

#[test]
fn mcp_error_maps_invalid_params_to_invalid_params_code() {
    let err = McpError::InvalidParams("root is required".into());
    let data = err.to_error_data();
    assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
}

#[test]
fn mcp_error_maps_engine_to_internal_error_code() {
    let err = McpError::Engine(EngineError::NotRunning);
    let data = err.to_error_data();
    assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
}

#[test]
fn mcp_error_maps_not_ready_to_internal_error_code() {
    let err = McpError::NotReady("engine not ready".into());
    let data = err.to_error_data();
    assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
}

#[test]
fn mcp_error_maps_json_to_invalid_params_code() {
    let err = McpError::Json(serde_json::from_str::<serde_json::Value>("not json").unwrap_err());
    let data = err.to_error_data();
    assert_eq!(data.code, ErrorCode::INVALID_PARAMS);
}

#[test]
fn mcp_error_maps_transport_to_internal_error_code() {
    let err = McpError::Transport("join error".into());
    let data = err.to_error_data();
    assert_eq!(data.code, ErrorCode::INTERNAL_ERROR);
}
