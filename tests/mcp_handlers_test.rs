use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;

use mercury_cortex::mcp::context::McpContext;
use mercury_cortex::mcp::session::SessionId;
use mercury_cortex::mcp::tools;
use mercury_cortex_core::db;
use mercury_cortex_core::engine::KnowledgeEngine;
use mercury_cortex_core::runtime::RuntimeContext;
use mercury_cortex_core::runtime::status::RuntimePhase;
use mercury_cortex_core::schema;

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

/// Create a test project in the database and return its ID and root path.
async fn create_test_project(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
) -> (String, String) {
    use surrealdb::types::Value as SurrealValue;

    // First ensure a user exists (required by owner_id foreign key)
    let user_result: Vec<SurrealValue> = db
        .query("CREATE users SET name = 'test', email = 'test@test.com', agent_name = 'agent-test', type = 'personal', created_at = time::now(), updated_at = time::now() RETURN id")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    let user_record = user_result.into_iter().next().unwrap();
    let user_obj = user_record.as_object().unwrap();
    let user_id = user_obj.get("id").unwrap().clone();

    let root = "/tmp/test_project_root";
    let result: Vec<SurrealValue> = db
        .query(
            "CREATE projects SET owner_id = $owner_id, name = 'test', slug = 'test', root_path = $root, created_at = time::now(), updated_at = time::now() RETURN id"
        )
        .bind(("owner_id", user_id))
        .bind(("root", root))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    let project_record = result.into_iter().next().unwrap();
    let project_obj = project_record.as_object().unwrap();
    let project_rid = project_obj.get("id").unwrap();
    let project_id = mercury_cortex_core::util::record_thing_to_string(project_rid).unwrap();

    (project_id, root.to_string())
}

/// Create a test project rooted at an arbitrary filesystem path.
async fn create_test_project_at(
    db: &surrealdb::Surreal<surrealdb::engine::local::Db>,
    root: &std::path::Path,
) -> (String, String) {
    use surrealdb::types::Value as SurrealValue;

    let user_result: Vec<SurrealValue> = db
        .query("CREATE users SET name = 'test', email = 'test@test.com', agent_name = 'agent-test', type = 'personal', created_at = time::now(), updated_at = time::now() RETURN id")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    let user_record = user_result.into_iter().next().unwrap();
    let user_obj = user_record.as_object().unwrap();
    let user_id = user_obj.get("id").unwrap().clone();

    let root_str = root.to_string_lossy();
    let result: Vec<SurrealValue> = db
        .query(
            "CREATE projects SET owner_id = $owner_id, name = 'test', slug = 'test', root_path = $root, created_at = time::now(), updated_at = time::now() RETURN id"
        )
        .bind(("owner_id", user_id))
        .bind(("root", root_str.as_ref()))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    let project_record = result.into_iter().next().unwrap();
    let project_obj = project_record.as_object().unwrap();
    let project_rid = project_obj.get("id").unwrap();
    let project_id = mercury_cortex_core::util::record_thing_to_string(project_rid).unwrap();

    (project_id, root_str.to_string())
}

#[tokio::test]
async fn test_handle_info_returns_engine_info() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;
    let result = tools::cortex::handle_info(ctx, session, json!({}))
        .await
        .unwrap();
    assert!(result.get("version").and_then(|v| v.as_str()).is_some());
    assert_eq!(result.get("running").and_then(|v| v.as_bool()), Some(false));
}

#[tokio::test]
async fn test_handle_open_and_close() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let (project_id, root) = create_test_project(ctx.engine.db()).await;

    // Open the project
    let result = tools::project::handle_open(
        ctx.clone(),
        session,
        json!({"project_id": project_id, "root": root}),
    )
    .await
    .unwrap();
    assert_eq!(result["status"], "opened");
    assert_eq!(result["project_id"], project_id);

    // Close the project
    let result = tools::project::handle_close(ctx.clone(), session, json!({}))
        .await
        .unwrap();
    assert_eq!(result["status"], "closed");
}

#[tokio::test]
async fn test_handle_open_missing_params() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_open(ctx.clone(), session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("project_id"));

    let err = tools::project::handle_open(ctx, session, json!({"project_id": "p"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("root"));
}

#[tokio::test]
async fn test_handle_project_status_no_project() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;
    let result = tools::project::handle_project_status(ctx, session, json!({}))
        .await
        .unwrap();
    assert_eq!(result["status"], "no_project_open");
}

#[tokio::test]
async fn test_handle_project_status_with_project() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let (project_id, root) = create_test_project(ctx.engine.db()).await;

    tools::project::handle_open(
        ctx.clone(),
        session,
        json!({"project_id": project_id, "root": root}),
    )
    .await
    .unwrap();

    let result = tools::project::handle_project_status(ctx, session, json!({}))
        .await
        .unwrap();
    assert!(!result.is_null());
}

#[tokio::test]
async fn test_handle_search_returns_empty() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let (project_id, root) = create_test_project(ctx.engine.db()).await;

    tools::project::handle_open(
        ctx.clone(),
        session,
        json!({"project_id": project_id, "root": root}),
    )
    .await
    .unwrap();

    let result = tools::search::handle_search(ctx, session, json!({"query": "test", "limit": 10}))
        .await
        .unwrap();
    assert!(result["results"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_handle_search_invalid_params() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;
    let err = tools::search::handle_search(ctx, session, json!({"query": 123}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid"));
}

#[tokio::test]
async fn test_handle_import_metadata_empty() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let project_tmp = TempDir::new().unwrap();
    let root = project_tmp.path().to_path_buf();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    std::fs::write(mc_dir.join(".mcignore"), ".git\n.vscode\n").unwrap();

    let (project_id, _) = create_test_project(&ctx.engine.db().clone()).await;
    ctx.engine.set_project(project_id, root).await;

    let result = tools::metadata::handle_import_metadata(ctx, session, json!({}))
        .await
        .unwrap();
    assert!(result["results"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_handle_file_metadata_missing_path() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::file::handle_get_file_metadata(ctx, session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("path"));
}

#[tokio::test]
async fn test_handle_file_metadata_not_found() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result =
        tools::file::handle_get_file_metadata(ctx, session, json!({"path": "nonexistent.rs"}))
            .await
            .unwrap();
    assert_eq!(result["status"], "not_found");
}

#[tokio::test]
async fn test_handle_import_metadata_returns_zero_when_no_temp_dir() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let project_tmp = TempDir::new().unwrap();
    let root = project_tmp.path().to_path_buf();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    std::fs::write(mc_dir.join(".mcignore"), ".git\n.vscode\n").unwrap();

    let (project_id, _) = create_test_project(&ctx.engine.db().clone()).await;
    ctx.engine.set_project(project_id, root).await;

    let result = tools::metadata::handle_import_metadata(ctx, session, json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["indexed_files"], 0);
    assert_eq!(val["results"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_handle_import_metadata_imports_staged_files() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let project_tmp = TempDir::new().unwrap();
    let root = project_tmp.path().to_path_buf();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    std::fs::write(mc_dir.join(".mcignore"), "build/\n.git/\n").unwrap();
    std::fs::create_dir_all(root.join("lib")).unwrap();
    std::fs::write(root.join("lib").join("main.dart"), "void main() {}\n").unwrap();
    std::fs::write(root.join("README.md"), "# demo\n").unwrap();
    std::fs::create_dir_all(root.join("build")).unwrap();
    std::fs::write(root.join("build").join("generated.dart"), "generated").unwrap();

    let temp_dir = mc_dir.join("temp");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("main.dart.json"),
        r#"{"path": "lib/main.dart", "purpose": "entrypoint", "summary": "app entry"}"#,
    )
    .unwrap();
    std::fs::write(
        temp_dir.join("README.md.json"),
        r#"{"path": "README.md", "purpose": "docs", "summary": "readme"}"#,
    )
    .unwrap();
    std::fs::write(
        temp_dir.join("generated.dart.json"),
        r#"{"path": "build/generated.dart", "purpose": "ignored", "summary": "should be skipped"}"#,
    )
    .unwrap();

    let (project_id, _) = create_test_project_at(&ctx.engine.db().clone(), &root).await;
    ctx.engine.set_project(project_id, root).await;

    let result = tools::metadata::handle_import_metadata(ctx.clone(), session, json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(
        val["indexed_files"], 2,
        "response reports post-import count"
    );
    let results = val["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r["success"] == true));

    let post_import = ctx.engine.count_indexed_files().await.unwrap();
    assert_eq!(
        post_import, 2,
        "ignored build/generated.dart must not be indexed"
    );
}

#[tokio::test]
async fn test_handle_update_mcignore_appends_patterns() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    let mcignore_path = mc_dir.join(".mcignore");
    std::fs::write(&mcignore_path, ".git\n.vscode\n").unwrap();

    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::project::handle_update_mcignore(
        ctx,
        session,
        serde_json::json!({
            "root": root.to_string_lossy(),
            "patterns": ["target", "node_modules"]
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["updated"], true);
    assert_eq!(result["pattern_count"], 2);

    let content = std::fs::read_to_string(&mcignore_path).unwrap();
    assert!(content.contains("target"));
    assert!(content.contains("node_modules"));
    assert!(content.contains(".git"));
}

#[tokio::test]
async fn test_handle_update_mcignore_skips_duplicates() {
    use tempfile::TempDir;
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    let mcignore_path = mc_dir.join(".mcignore");
    std::fs::write(&mcignore_path, "target\n").unwrap();

    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::project::handle_update_mcignore(
        ctx,
        session,
        serde_json::json!({
            "root": root.to_string_lossy(),
            "patterns": ["target"]
        }),
    )
    .await
    .unwrap();

    assert_eq!(result["updated"], false);
}

#[tokio::test]
async fn test_handle_index_paths_returns_paths() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    // Without an active project, it should error
    let result = tools::index::handle_index_paths(ctx.clone(), session, json!({})).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_open_validates_project_exists() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    // Opening a non-existent project should fail
    let result = tools::project::handle_open(
        ctx.clone(),
        session,
        json!({"project_id": "projects:nonexistent", "root": "/tmp/test"}),
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found"));
}

#[tokio::test]
async fn test_workflow_step_returns_content() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result =
        tools::workflow::handle_workflow_step(ctx, session, json!({"mode": "dev", "step": 1}))
            .await
            .unwrap();

    assert_eq!(result["name"], "dev:step1");
    assert_eq!(result["title"], "Analyze the Request");
    assert!(result["content"].as_str().unwrap().contains("Analyze"));
}

#[tokio::test]
async fn test_workflow_step_init_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result =
        tools::workflow::handle_workflow_step(ctx, session, json!({"mode": "init", "step": 1}))
            .await
            .unwrap();

    assert_eq!(result["name"], "init:step1");
    assert!(
        result["content"]
            .as_str()
            .unwrap()
            .contains("Validate Project")
    );
}

#[tokio::test]
async fn test_workflow_step_invalid_step() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err =
        tools::workflow::handle_workflow_step(ctx, session, json!({"mode": "dev", "step": 99}))
            .await
            .unwrap_err();

    assert!(err.to_string().contains("unknown step"));
}

#[tokio::test]
async fn test_workflow_step_invalid_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err =
        tools::workflow::handle_workflow_step(ctx, session, json!({"mode": "invalid", "step": 1}))
            .await
            .unwrap_err();

    assert!(err.to_string().contains("unknown mode"));
}

#[tokio::test]
async fn test_handle_register_missing_root() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_register(ctx, session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("root"));
}

#[tokio::test]
async fn test_handle_register_invalid_root() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_register(
        ctx,
        session,
        json!({"root": "/nonexistent/path/that/does/not/exist"}),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("failed to create"));
}

/// Re-registering through `project/register` (which uses the core
/// `ProjectService`) should reconcile a duplicate record left behind by a
/// stale DB view instead of failing with an identity conflict.
#[tokio::test]
async fn test_handle_register_reconciles_duplicate_root_record() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    ctx.engine
        .db()
        .query(
            "CREATE users SET name = 'test', email = 'test@test.com', \
             agent_name = 'agent-test', type = 'personal', \
             created_at = time::now(), updated_at = time::now()",
        )
        .await
        .unwrap();

    let root = TempDir::new().unwrap();
    let root_str = root.path().to_string_lossy().to_string();

    // First registration creates the canonical project + config.json
    let first = tools::project::handle_register(ctx.clone(), session, json!({"root": root_str}))
        .await
        .unwrap();
    let canonical_id = first["project_id"].as_str().unwrap().to_string();

    // Simulate a stale-DB-view duplicate at the same root. The v005
    // `unique_root_path` index normally prevents a second record with the same
    // root_path, so drop it first to reproduce the pre-v005 state the reconcile
    // safety net was added for.
    ctx.engine
        .db()
        .query("REMOVE INDEX unique_root_path ON TABLE projects")
        .await
        .unwrap();
    let owner_id: Vec<surrealdb::types::Value> = ctx
        .engine
        .db()
        .query("SELECT VALUE id FROM users LIMIT 1")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    let owner = owner_id.into_iter().next().unwrap();
    ctx.engine
        .db()
        .query(
            "CREATE projects SET owner_id = $owner_id, name = 'mct1', slug = 'mct1', \
             root_path = $root, created_at = time::now(), updated_at = time::now()",
        )
        .bind(("owner_id", owner))
        .bind(("root", root_str.clone()))
        .await
        .unwrap()
        .take::<Vec<surrealdb::types::Value>>(0)
        .unwrap();

    let projects: Vec<surrealdb::types::Value> = ctx
        .engine
        .db()
        .query("SELECT * FROM projects")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert_eq!(projects.len(), 2, "precondition: duplicate record exists");

    // Re-registering should reconcile, not fail with an identity conflict
    let second = tools::project::handle_register(ctx.clone(), session, json!({"root": root_str}))
        .await
        .unwrap();
    assert_eq!(
        second["project_id"].as_str().unwrap(),
        canonical_id,
        "canonical project_id should be preserved"
    );

    let projects: Vec<surrealdb::types::Value> = ctx
        .engine
        .db()
        .query("SELECT * FROM projects")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    assert_eq!(projects.len(), 1, "duplicate record should be removed");
}

#[tokio::test]
async fn test_handle_update_missing_project_id() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_update(ctx, session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("project_id"));
}

#[tokio::test]
async fn test_handle_update_missing_metadata() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_update(ctx, session, json!({"project_id": "projects:test"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("metadata"));
}

#[tokio::test]
async fn test_handle_update_invalid_metadata() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_update(
        ctx,
        session,
        json!({"project_id": "projects:test", "metadata": "not an object"}),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("ProjectMetadata"));
}

#[tokio::test]
async fn test_handle_workflow_session_missing_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::workflow::handle_workflow_session(ctx, session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("mode"));
}

#[tokio::test]
async fn test_handle_workflow_session_invalid_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::workflow::handle_workflow_session(ctx, session, json!({"mode": "invalid"}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown mode"));
}

#[tokio::test]
async fn test_handle_workflow_session_dev_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::workflow::handle_workflow_session(ctx, session, json!({"mode": "dev"}))
        .await
        .unwrap();
    assert_eq!(result["mode"], "dev");
    assert!(!result["workflow"]["steps"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_handle_workflow_session_init_mode() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::workflow::handle_workflow_session(ctx, session, json!({"mode": "init"}))
        .await
        .unwrap();
    assert_eq!(result["mode"], "init");
    assert!(!result["workflow"]["steps"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_handle_index_paths_no_active_project() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::index::handle_index_paths(ctx, session, json!({}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("no active project"));
}

#[tokio::test]
async fn test_handle_search_missing_query_defaults() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::search::handle_search(ctx, session, json!({"query": 123}))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid"));
}

#[tokio::test]
async fn test_handle_open_nonexistent_project() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let err = tools::project::handle_open(
        ctx,
        session,
        json!({"project_id": "projects:nonexistent", "root": "/tmp"}),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_handle_import_metadata_no_temp_dir() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let project_tmp = TempDir::new().unwrap();
    let root = project_tmp.path().to_path_buf();
    let mc_dir = root.join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    std::fs::write(mc_dir.join(".mcignore"), ".git\n.vscode\n").unwrap();

    let (project_id, _) = create_test_project(&ctx.engine.db().clone()).await;
    ctx.engine.set_project(project_id, root).await;

    let result = tools::metadata::handle_import_metadata(ctx, session, json!({}))
        .await
        .unwrap();
    assert!(result["results"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_handle_info_returns_version() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::cortex::handle_info(ctx, session, json!({}))
        .await
        .unwrap();
    assert!(result.get("version").is_some());
    assert!(result.get("running").is_some());
}

#[tokio::test]
async fn test_handle_close_without_open_succeeds() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    let result = tools::project::handle_close(ctx, session, json!({}))
        .await
        .unwrap();
    assert_eq!(result["status"], "closed");
}

#[tokio::test]
async fn register_tool_preserves_existing_config_project_id() {
    let (_tmp, ctx) = create_test_context().await;
    let session: SessionId = 1;

    ctx.engine
        .db()
        .query(
            "CREATE users SET name = 'test', email = 'test@test.com', \
             agent_name = 'agent-test', type = 'personal', \
             created_at = time::now(), updated_at = time::now()",
        )
        .await
        .unwrap();

    let root = tempfile::TempDir::new().unwrap();
    let root_str = root.path().to_string_lossy().to_string();
    let mc_dir = root.path().join(".mercury-cortex");
    std::fs::create_dir_all(&mc_dir).unwrap();
    let config_path = mc_dir.join("config.json");
    std::fs::write(
        &config_path,
        serde_json::json!({ "version": "1", "project_id": "projects:existing-id" }).to_string(),
    )
    .unwrap();

    // The config references an already-registered project whose record lives at
    // a DIFFERENT root. If `handle_register` consults the config (Moved path) the
    // id is preserved; if the scaffold read were broken (config_project_id =
    // None) `register` would mint a fresh id and the assertion below fails.
    let old_root = root_str.clone() + "/old-registration";
    let owner_id: Vec<surrealdb::types::Value> = ctx
        .engine
        .db()
        .query("SELECT VALUE id FROM users LIMIT 1")
        .await
        .unwrap()
        .take(0)
        .unwrap();
    let owner = owner_id.into_iter().next().unwrap();
    ctx.engine
        .db()
        .query(
            "CREATE projects:⟨existing-id⟩ SET owner_id = $owner_id, name = 'existing', \
             slug = 'existing', root_path = $root, \
             created_at = time::now(), updated_at = time::now()",
        )
        .bind(("owner_id", owner))
        .bind(("root", old_root))
        .await
        .unwrap()
        .take::<Vec<surrealdb::types::Value>>(0)
        .unwrap();

    let params = json!({ "root": root_str });
    let result = tools::project::handle_register(ctx, session, params)
        .await
        .expect("register must succeed");
    assert_eq!(result["project_id"], "projects:existing-id");

    // Config file is untouched (still the existing project_id).
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("projects:existing-id"));
}
