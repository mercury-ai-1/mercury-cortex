use std::sync::Arc;

use tempfile::TempDir;

use mercury_cortex::mcp::context::{LazyContext, McpContext};
use mercury_cortex::mcp::handler::McpHandler;
use mercury_cortex_core::db;
use mercury_cortex_core::engine::KnowledgeEngine;
use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::runtime::status::RuntimePhase;
use mercury_cortex_core::schema;
use rmcp::ServerHandler;

async fn create_test_context() -> (TempDir, McpContext) {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("test.db");
    let db = db::initialize(&db_path).await.unwrap();
    schema::run_pending(&db).await.unwrap();
    let engine = KnowledgeEngine::new(db.clone());
    let rt = Arc::new(RuntimeContext::new_for_test());
    rt.set_database(db, RuntimePhase::DatabaseConnected);
    rt.set_engine(Arc::new(engine), RuntimePhase::Running);
    let ctx = McpContext::new(rt.engine().unwrap(), rt);
    (tmp, ctx)
}

#[tokio::test]
async fn test_handler_has_all_tools() {
    let (_tmp, ctx) = create_test_context().await;
    let lazy = LazyContext::new();
    lazy.set(ctx);
    let handler = McpHandler { ctx: lazy };

    let expected = [
        "cortex/info",
        "search/code",
        "project/open",
        "project/close",
        "project/status",
        "project/register",
        "project/update",
        "project/update_mcignore",
        "metadata/import",
        "file/metadata",
        "index/paths",
        "workflow/session",
        "workflow/step",
    ];
    for name in &expected {
        assert!(handler.get_tool(name).is_some(), "missing tool: {name}");
    }
    assert!(handler.get_tool("nonexistent").is_none());
}

#[tokio::test]
async fn test_server_info_includes_tools_and_prompts() {
    let (_tmp, ctx) = create_test_context().await;
    let lazy = LazyContext::new();
    lazy.set(ctx);
    let handler = McpHandler { ctx: lazy };

    let info = handler.get_info();
    assert!(!info.server_info.name.is_empty());
    let caps = info.capabilities;
    assert!(caps.tools.is_some(), "should advertise tools capability");
}
