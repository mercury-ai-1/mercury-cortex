use std::path::Path;

use tempfile::TempDir;

use mercury_cortex_core::db;
use mercury_cortex_core::schema;

/// Helper: connect to a temporary database and run all pending migrations.
async fn run_setup_in(tmp: &Path) -> anyhow::Result<mercury_cortex_core::SurrealDb> {
    let db_path = tmp.join("mercury_cortex_global_knowledge.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;
    Ok(db)
}

#[tokio::test]
async fn test_setup_creates_database_and_schema() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    run_setup_in(tmp.path()).await?;
    let db_path = tmp.path().join("mercury_cortex_global_knowledge.db");
    assert!(db_path.exists(), "database directory should exist");
    Ok(())
}

#[tokio::test]
async fn test_setup_is_idempotent() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("mercury_cortex_global_knowledge.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;
    schema::run_pending(&db).await?;
    let info: surrealdb::types::Value = db.query("INFO FOR DB").await?.take(0)?;
    let info_str = format!("{info:?}");
    assert!(
        info_str.contains("users"),
        "schema should remain after second run"
    );
    Ok(())
}

#[tokio::test]
async fn test_db_initialize_connects() -> surrealdb::Result<()> {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("test_connect.db");
    let db = db::initialize(&db_path).await?;
    db.query("RETURN 1").await?;
    Ok(())
}

#[tokio::test]
async fn test_schema_defines_all_tables() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("schema_test.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;

    let expected = [
        "users",
        "projects",
        "file_data",
        "owns",
        "contains",
        "imports",
        "calls",
        "depends_on",
        "part_of_pattern",
    ];

    let info: surrealdb::types::Value = db.query("INFO FOR DB").await?.take(0)?;
    let info_str = format!("{info:?}");
    for table in &expected {
        assert!(
            info_str.contains(table),
            "table '{table}' should be defined in database"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_verify_schema_passes_after_setup() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("verify_ok.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;
    schema::verify_schema(&db).await?;
    Ok(())
}

#[tokio::test]
async fn test_verify_schema_fails_on_empty_db() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("verify_fail.db");
    let db = db::initialize(&db_path).await?;
    let result = schema::verify_schema(&db).await;
    assert!(
        result.is_err(),
        "verify_schema should fail on empty database"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("missing"),
        "error should mention missing tables"
    );
    Ok(())
}

#[tokio::test]
async fn test_verify_schema_idempotent() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("verify_idem.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;
    schema::verify_schema(&db).await?;
    schema::verify_schema(&db).await?;
    Ok(())
}

#[tokio::test]
async fn test_migration_tracks_applied() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("migration_track.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;

    let applied: Vec<i64> = db
        .query("SELECT VALUE version FROM _migrations ORDER BY version")
        .await?
        .take(0)?;
    assert_eq!(
        applied.len(),
        mercury_cortex_core::schema::migration::registry::all_migrations().len(),
        "all migrations should be recorded"
    );
    Ok(())
}

#[tokio::test]
async fn test_migration_does_not_reapply() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db_path = tmp.path().join("migration_reapply.db");
    let db = db::initialize(&db_path).await?;
    schema::run_pending(&db).await?;
    schema::run_pending(&db).await?;

    let applied: Vec<i64> = db
        .query("SELECT VALUE version FROM _migrations ORDER BY version")
        .await?
        .take(0)?;
    assert_eq!(
        applied.len(),
        mercury_cortex_core::schema::migration::registry::all_migrations().len(),
        "migrations should not be duplicated after re-run"
    );
    Ok(())
}
