//! End-to-end tests for the `db export` CLI command.
//!
//! Each test seeds a throwaway database under `$home/.mercury/cortex` and runs
//! the real binary with `HOME` pointed at that directory (mirrors the other
//! CLI integration tests).

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use mercury_cortex_core::db;
use mercury_cortex_core::schema;

/// Seed a throwaway database under `$home/.mercury/cortex`.
///
/// Records satisfy the SCHEMAFULL migrations' required, non-nullable fields
/// (`projects.owner_id`, `users.agent_name`, `file_data.project_id`) so the
/// rows actually persist and `SELECT *` sees them.
async fn seed(home: &Path) {
    let db_path = home.join(".mercury/cortex/mercury_cortex_global_knowledge.db");
    let db = db::initialize(&db_path).await.unwrap();
    schema::run_pending(&db).await.unwrap();
    db.query("CREATE users:u1 SET name = 'U', email = 'u@example.com', type = '', agent_name = 'agent-u1', created_at = time::now(), updated_at = time::now()")
        .await
        .unwrap();
    db.query("CREATE projects:p1 SET owner_id = users:u1, name = 'P1', slug = 'p1', root_path = '/p1', created_at = time::now(), updated_at = time::now()")
        .await
        .unwrap();
    db.query("CREATE projects:p2 SET owner_id = users:u1, name = 'P2', slug = 'p2', root_path = '/p2', created_at = time::now(), updated_at = time::now()")
        .await
        .unwrap();
    db.query("CREATE file_data:f1 SET project_id = projects:p1, path = '/a.rs', indexed_at = time::now(), updated_at = time::now()")
        .await
        .unwrap();
    db.query("CREATE file_data:f2 SET project_id = projects:p2, path = '/b.rs', indexed_at = time::now(), updated_at = time::now()")
        .await
        .unwrap();
}

fn run(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mercury-cortex"))
        .args(args)
        .env("HOME", home)
        .output()
        .expect("binary should run")
}

#[tokio::test]
async fn export_single_table_via_flag() {
    let home = TempDir::new().unwrap();
    seed(home.path()).await;
    let out = home.path().join("out");
    let out_str = out.to_str().unwrap();

    let output = run(
        home.path(),
        &["db", "export", "--table", "projects", "--out", out_str],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let content = std::fs::read_to_string(out.join("projects.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn export_all_via_flag() {
    let home = TempDir::new().unwrap();
    seed(home.path()).await;
    let out = home.path().join("out");
    let out_str = out.to_str().unwrap();

    let output = run(home.path(), &["db", "export", "--all", "--out", out_str]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for t in ["projects", "file_data", "users"] {
        assert!(out.join(format!("{t}.json")).exists(), "missing {t}.json");
    }
}

#[tokio::test]
async fn list_tables_flag_prints_and_exits() {
    let home = TempDir::new().unwrap();
    seed(home.path()).await;

    let output = run(home.path(), &["db", "export", "--list-tables"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("projects"), "stdout: {stdout}");
    assert!(stdout.contains("file_data"), "stdout: {stdout}");
}

#[tokio::test]
async fn project_id_filter_via_flag() {
    let home = TempDir::new().unwrap();
    seed(home.path()).await;
    let out = home.path().join("out");
    let out_str = out.to_str().unwrap();

    let output = run(
        home.path(),
        &[
            "db", "export", "--table", "file_data", "--project-id",
            "projects:p1", "--out", out_str,
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let content = std::fs::read_to_string(out.join("file_data.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unknown_table_fails_nonzero() {
    let home = TempDir::new().unwrap();
    seed(home.path()).await;

    let output = run(home.path(), &["db", "export", "--table", "nonexistent"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nonexistent"), "stderr: {stderr}");
}

#[test]
fn all_and_table_flags_conflict() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["db", "export", "--table", "projects", "--all"]);
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn export_without_db_fails_gracefully() {
    let home = TempDir::new().unwrap();
    let output = run(home.path(), &["db", "export", "--table", "projects"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No database found"),
        "stderr: {stderr}"
    );
}
