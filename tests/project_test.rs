use std::fs;
use std::path::Path;

use anyhow::Context;
use tempfile::TempDir;

use mercury_cortex::commands::project;
use mercury_cortex_core::client::CoreClient;
use mercury_cortex_core::db;
use mercury_cortex_core::schema;

/// Helper: initialise a temporary database with all migrations applied and a
/// `CoreClient` facade bound to it (sharing the same connection).
async fn setup_db(_tmp: &Path) -> (mercury_cortex_core::SurrealDb, CoreClient, TempDir) {
    let db_dir = TempDir::new().unwrap();
    let db_path = db_dir.path().join("test.db");
    let database = db::initialize(&db_path).await.unwrap();
    schema::run_pending(&database).await.unwrap();
    let client =
        CoreClient::from_connection(database.clone(), db_dir.path().to_path_buf()).unwrap();
    (database, client, db_dir)
}

/// Helper: create a single user profile for testing.
async fn create_profile(db: &mercury_cortex_core::SurrealDb) {
    db.query(
        "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
         created_at = time::now(), updated_at = time::now()",
    )
    .bind(("name", "Test User"))
    .bind(("email", "test@example.com"))
    .bind(("agent_name", "agent-test"))
    .bind(("type", "personal"))
    .await
    .unwrap();
}

#[tokio::test]
async fn test_project_creates_mercury_cortex_directory() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let mc_dir = project_root.path().join(".mercury-cortex");
    assert!(mc_dir.exists(), ".mercury-cortex directory should exist");
    assert!(
        mc_dir.join("config.json").exists(),
        "config.json should exist"
    );
    assert!(mc_dir.join(".mcignore").exists(), ".mcignore should exist");
    Ok(())
}

#[tokio::test]
async fn test_config_json_has_required_fields() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let config_path = project_root.path().join(".mercury-cortex/config.json");
    let content = fs::read_to_string(&config_path)?;

    assert!(
        content.contains("\"version\": \"1\""),
        "config should have version field"
    );
    assert!(
        content.contains("\"project_id\":"),
        "config should have project_id field"
    );
    Ok(())
}

#[tokio::test]
async fn test_mcignore_has_default_entries() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let mcignore_path = project_root.path().join(".mercury-cortex/.mcignore");
    let content = fs::read_to_string(&mcignore_path)?;

    for entry in &["target", "build", ".git", ".vscode", ".DS_Store"] {
        assert!(
            content.contains(entry),
            ".mcignore should contain '{entry}'"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_project_registers_in_database() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let projects: Vec<surrealdb::types::Value> = db
        .query(
            "SELECT name, slug, root_path, owner_id, \
                 created_at, updated_at FROM projects",
        )
        .await?
        .take(0)?;

    assert_eq!(projects.len(), 1, "should have exactly one project record");

    let obj = projects[0].as_object().unwrap();
    assert!(obj.contains_key("name"), "should have name field");
    assert!(obj.contains_key("slug"), "should have slug field");
    assert!(obj.contains_key("root_path"), "should have root_path field");
    assert!(obj.contains_key("owner_id"), "should have owner_id field");
    assert!(
        obj.contains_key("created_at"),
        "should have created_at field"
    );
    assert!(
        obj.contains_key("updated_at"),
        "should have updated_at field"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_is_idempotent() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;
    project::initialize_project(project_root.path(), &client).await?;

    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM projects").await?.take(0)?;

    assert_eq!(
        projects.len(),
        1,
        "should still have exactly one project record after re-run"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_fails_without_profile() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (_db, client, _db_dir) = setup_db(project_root.path()).await;

    let result = project::initialize_project(project_root.path(), &client).await;

    assert!(result.is_err(), "should fail without a user profile");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.to_lowercase().contains("profile") || msg.to_lowercase().contains("user"),
        "error should mention profile: {msg}"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_fails_on_nonexistent_root() -> anyhow::Result<()> {
    let (db, client, _db_dir) = setup_db(Path::new("/tmp")).await;
    create_profile(&db).await;

    let nonexistent = Path::new("/tmp/mercury_cortex_test_nonexistent_dir_xyzzy");
    let result = project::initialize_project(nonexistent, &client).await;

    assert!(result.is_err(), "should fail on nonexistent path");
    Ok(())
}

#[tokio::test]
async fn test_mcignore_preserves_user_rules() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    let mc_dir = project_root.path().join(".mercury-cortex");
    fs::create_dir_all(&mc_dir)?;

    let mcignore_path = mc_dir.join(".mcignore");
    let user_content = "# Custom rules\nnode_modules\n.env\n";
    fs::write(&mcignore_path, user_content)?;

    project::initialize_project(project_root.path(), &client).await?;

    let content = fs::read_to_string(&mcignore_path)?;
    assert!(
        content.contains("node_modules"),
        "should preserve user rule: node_modules"
    );
    assert!(content.contains(".env"), "should preserve user rule: .env");
    assert!(
        content.contains("target"),
        "should append missing default: target"
    );
    assert!(
        content.contains(".git"),
        "should append missing default: .git"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_id_stays_same_across_runs() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let config_path = project_root.path().join(".mercury-cortex/config.json");
    let first_run = fs::read_to_string(&config_path)?;

    project::initialize_project(project_root.path(), &client).await?;

    let second_run = fs::read_to_string(&config_path)?;
    assert_eq!(
        first_run, second_run,
        "project_id should remain stable across re-runs"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_id_matches_db_record_id() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    project::initialize_project(project_root.path(), &client).await?;

    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT VALUE id FROM projects").await?.take(0)?;

    let db_id = projects.first().context("No project record found")?;

    let db_id_str = mercury_cortex_core::util::record_thing_to_string(db_id)
        .context("project record id should be a RecordId")?;

    let config_path = project_root.path().join(".mercury-cortex/config.json");
    let config = fs::read_to_string(&config_path)?;

    assert!(
        config.contains(&db_id_str),
        "config.json project_id should match the DB record ID"
    );
    Ok(())
}

/// Extract the `project_id` value from a config.json string.
fn extract_project_id(content: &str) -> Option<String> {
    let key = "\"project_id\": \"";
    let start = content.find(key)?;
    let value_start = start + key.len();
    let end = content[value_start..].find('"')?;
    Some(content[value_start..value_start + end].to_string())
}

#[tokio::test]
async fn test_project_moved_updates_root_path() -> anyhow::Result<()> {
    let dir_old = TempDir::new()?;
    let dir_new = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(dir_old.path()).await;
    create_profile(&db).await;

    // Create project at the old location
    project::initialize_project(dir_old.path(), &client).await?;

    // Read the generated config to capture the project identity
    let config_old = dir_old.path().join(".mercury-cortex/config.json");
    let config_content = fs::read_to_string(&config_old)?;
    let project_id =
        extract_project_id(&config_content).context("config.json missing project_id")?;

    // Simulate move: place same config at new location
    let mc_dir_new = dir_new.path().join(".mercury-cortex");
    fs::create_dir_all(&mc_dir_new)?;
    fs::write(mc_dir_new.join("config.json"), &config_content)?;

    // Initialize at the new location
    project::initialize_project(dir_new.path(), &client).await?;

    // Should still be exactly one project record
    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM projects").await?.take(0)?;
    assert_eq!(projects.len(), 1, "should not create a duplicate record");

    // The root_path should point to the new location
    let expected = dir_new.path().to_string_lossy().to_string();
    let stored_paths: Vec<String> = db
        .query("SELECT VALUE root_path FROM projects")
        .await?
        .take(0)?;
    assert_eq!(
        stored_paths,
        vec![expected],
        "root_path should be updated to new location"
    );

    // Config at new location should have the same project_id
    let config_new = dir_new.path().join(".mercury-cortex/config.json");
    let new_content = fs::read_to_string(&config_new)?;
    let new_pid = extract_project_id(&new_content);
    assert_eq!(
        new_pid.as_deref(),
        Some(project_id.as_str()),
        "project_id should remain stable after move"
    );
    Ok(())
}

/// Re-running `mercury-cortex project` should reconcile a duplicate record
/// left behind by a stale DB view instead of bailing with an identity conflict.
#[tokio::test]
async fn test_project_reconciles_duplicate_root_record() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    // First run registers the canonical project and writes config.json
    project::initialize_project(project_root.path(), &client).await?;

    let config_path = project_root.path().join(".mercury-cortex/config.json");
    let config_before = fs::read_to_string(&config_path)?;
    let canonical_id =
        extract_project_id(&config_before).context("config.json missing project_id")?;

    let root_path = project_root.path().to_string_lossy().to_string();

    // Simulate the stale-DB-view duplicate: a second record at the same root.
    // The v005 `unique_root_path` index normally prevents a second record with
    // the same root_path, so drop it first to reproduce the pre-v005 state the
    // reconcile safety net was added for.
    db.query("REMOVE INDEX unique_root_path ON TABLE projects")
        .await?;
    let owner_id: Vec<surrealdb::types::Value> = db
        .query("SELECT VALUE id FROM users LIMIT 1")
        .await?
        .take(0)?;
    let owner = owner_id.first().cloned().context("no owner profile")?;
    db.query(
        "CREATE projects SET owner_id = $owner_id, name = 'mct1', slug = 'mct1', \
         root_path = $root_path, created_at = time::now(), updated_at = time::now()",
    )
    .bind(("owner_id", owner))
    .bind(("root_path", root_path.clone()))
    .await?
    .take::<Vec<surrealdb::types::Value>>(0)?;

    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM projects").await?.take(0)?;
    assert_eq!(projects.len(), 2, "precondition: duplicate record exists");

    // Re-running in the same directory should succeed and self-heal
    project::initialize_project(project_root.path(), &client).await?;

    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM projects").await?.take(0)?;
    assert_eq!(projects.len(), 1, "duplicate record should be removed");

    let config_after = fs::read_to_string(&config_path)?;
    assert_eq!(
        config_before, config_after,
        "config.json should be unchanged when the canonical project wins"
    );
    assert!(
        config_after.contains(&canonical_id),
        "canonical project_id should be preserved"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_identity_conflict_detected() -> anyhow::Result<()> {
    let dir_a = TempDir::new()?;
    let dir_b = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(dir_a.path()).await;
    create_profile(&db).await;

    // Create project A
    project::initialize_project(dir_a.path(), &client).await?;
    let config_a = dir_a.path().join(".mercury-cortex/config.json");

    // Create project B at a different location
    project::initialize_project(dir_b.path(), &client).await?;
    let config_b = dir_b.path().join(".mercury-cortex/config.json");

    // Simulate conflict: put project B's config in project A's directory
    let b_content = fs::read_to_string(&config_b)?;
    fs::write(&config_a, &b_content)?;

    // Initializing at dir_a with B's config should fail with a conflict
    let result = project::initialize_project(dir_a.path(), &client).await;
    assert!(result.is_err(), "should fail on identity conflict");
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.to_lowercase().contains("identity conflict"),
        "error should mention identity conflict: {msg}"
    );
    Ok(())
}

#[tokio::test]
async fn test_project_stale_config_creates_new() -> anyhow::Result<()> {
    let project_root = TempDir::new()?;
    let (db, client, _db_dir) = setup_db(project_root.path()).await;
    create_profile(&db).await;

    // Create project and capture its identity
    project::initialize_project(project_root.path(), &client).await?;
    let config_path = project_root.path().join(".mercury-cortex/config.json");
    let config_before = fs::read_to_string(&config_path)?;
    let old_pid = extract_project_id(&config_before).context("config.json missing project_id")?;

    // Simulate DB reset: delete all project records
    db.query("DELETE projects")
        .await?
        .take::<Vec<surrealdb::types::Value>>(0)?;

    // Re-initialize with the stale config
    project::initialize_project(project_root.path(), &client).await?;

    // Should have a new project record
    let projects: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM projects").await?.take(0)?;
    assert_eq!(projects.len(), 1, "should create one new project record");

    // The project_id should differ from the old stale one
    let config_after = fs::read_to_string(&config_path)?;
    let new_pid = extract_project_id(&config_after);
    assert!(new_pid.is_some(), "config should have a project_id");
    assert_ne!(
        new_pid.as_deref(),
        Some(old_pid.as_str()),
        "project_id should change after stale config is replaced"
    );
    Ok(())
}
