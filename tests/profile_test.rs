use std::path::Path;

use surrealdb::types::Value;
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
async fn test_create_profile() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db = run_setup_in(tmp.path()).await?;

    db.query(
        "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
         created_at = time::now(), updated_at = time::now()",
    )
    .bind(("name", "Alice".to_string()))
    .bind(("email", "alice@example.com".to_string()))
    .bind(("agent_name", "agent-alice".to_string()))
    .bind(("type", "personal".to_string()))
    .await?;

    let profiles: Vec<surrealdb::types::Value> = db
        .query("SELECT name, email, type FROM users")
        .await?
        .take(0)?;

    assert_eq!(profiles.len(), 1, "should have one profile");

    let obj = profiles[0].as_object().expect("should be an object");
    let name = obj.get("name").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });
    let email = obj.get("email").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });
    let ptype = obj.get("type").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });

    assert_eq!(name, Some("Alice"), "name should match");
    assert_eq!(email, Some("alice@example.com"), "email should match");
    assert_eq!(ptype, Some("personal"), "type should match");
    Ok(())
}

#[tokio::test]
async fn test_duplicate_email_rejected() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db = run_setup_in(tmp.path()).await?;

    db.query(
        "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
         created_at = time::now(), updated_at = time::now()",
    )
    .bind(("name", "Alice".to_string()))
    .bind(("email", "alice@example.com".to_string()))
    .bind(("agent_name", "agent-alice".to_string()))
    .bind(("type", "personal".to_string()))
    .await?;

    let second_result = db
        .query(
            "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
             created_at = time::now(), updated_at = time::now()",
        )
        .bind(("name", "Bob".to_string()))
        .bind(("email", "alice@example.com".to_string()))
        .bind(("agent_name", "agent-bob".to_string()))
        .bind(("type", "organization".to_string()))
        .await;

    if let Err(e) = second_result {
        let msg = e.to_string();
        assert!(
            msg.contains("unique") || msg.contains("duplicate") || msg.contains("already exists"),
            "unexpected error on duplicate email: {msg}"
        );
    }

    let profiles: Vec<Value> = db.query("SELECT * FROM users").await?.take(0)?;
    assert_eq!(
        profiles.len(),
        1,
        "duplicate email should not create a second record"
    );
    Ok(())
}

#[tokio::test]
async fn test_profile_is_empty_initially() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db = run_setup_in(tmp.path()).await?;

    let profiles: Vec<surrealdb::types::Value> = db.query("SELECT * FROM users").await?.take(0)?;

    assert!(profiles.is_empty(), "no profiles should exist initially");
    Ok(())
}

#[tokio::test]
async fn test_update_profile() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db = run_setup_in(tmp.path()).await?;

    db.query(
        "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
         created_at = time::now(), updated_at = time::now()",
    )
    .bind(("name", "Alice".to_string()))
    .bind(("email", "alice@example.com".to_string()))
    .bind(("agent_name", "agent-alice".to_string()))
    .bind(("type", "personal".to_string()))
    .await?;

    let profiles: Vec<surrealdb::types::Value> =
        db.query("SELECT * FROM users LIMIT 1").await?.take(0)?;

    let id = profiles[0]
        .as_object()
        .and_then(|o| o.get("id"))
        .cloned()
        .expect("profile should have an id");

    db.query("UPDATE $id SET name = $name, email = $email, type = $type, updated_at = time::now()")
        .bind(("id", id))
        .bind(("name", "Alice Updated".to_string()))
        .bind(("email", "alice@newdomain.com".to_string()))
        .bind(("type", "organization".to_string()))
        .await?;

    let profiles: Vec<surrealdb::types::Value> = db
        .query("SELECT name, email, type FROM users")
        .await?
        .take(0)?;

    let obj = profiles[0].as_object().expect("should be an object");
    let name = obj.get("name").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });
    let email = obj.get("email").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });
    let ptype = obj.get("type").and_then(|v| {
        if let Value::String(s) = v {
            Some(s.as_str())
        } else {
            None
        }
    });

    assert_eq!(name, Some("Alice Updated"), "name should be updated");
    assert_eq!(
        email,
        Some("alice@newdomain.com"),
        "email should be updated"
    );
    assert_eq!(ptype, Some("organization"), "type should be updated");
    Ok(())
}

#[tokio::test]
async fn test_only_one_profile() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let db = run_setup_in(tmp.path()).await?;

    for i in 0..3 {
        db.query(
            "CREATE users SET name = $name, email = $email, agent_name = $agent_name, type = $type, \
             created_at = time::now(), updated_at = time::now()",
        )
        .bind(("name", format!("User {i}")))
        .bind(("email", format!("user{i}@example.com")))
        .bind(("agent_name", format!("agent-user{i}")))
        .bind(("type", "personal".to_string()))
        .await?;
    }

    let result: Vec<Value> = db
        .query("SELECT count() FROM users GROUP ALL")
        .await?
        .take(0)?;

    let count = result
        .first()
        .and_then(|v| v.as_object())
        .and_then(|o| o.get("count"))
        .and_then(|v| {
            if let Value::Number(n) = v {
                n.to_int()
            } else {
                None
            }
        })
        .unwrap_or(0);

    assert_eq!(count, 3, "should have 3 profiles when multiple are created");
    Ok(())
}
