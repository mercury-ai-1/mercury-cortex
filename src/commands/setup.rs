//! The `mercury-cortex setup` command handler.
//!
//! Detects the current environment state (fresh / partial / ready), creates
//! directories if needed, connects to the database, runs pending migrations,
//! and verifies the resulting schema.

use std::path::Path;

use serde_json::json;

use mercury_cortex_core::client::CoreClient;

/// Run the full setup flow.
///
/// Safe to call multiple times; uses `IF NOT EXISTS` throughout and tracks
/// applied migrations in a `_migrations` table.
pub async fn run(json: bool) -> Result<(), anyhow::Error> {
    let paths = CoreClient::paths()?;
    let data_dir = paths.data_dir;
    let db_path = paths.db_path;

    let state = detect_state(&data_dir, &db_path);
    if json {
        let status = match state {
            State::Fresh => "fresh",
            State::Partial => "partial",
            State::Ready => "ready",
        };
        eprintln!("{}", json!({ "status": status, "phase": "detect" }));
    } else {
        match state {
            State::Fresh => {
                println!("Setting up Mercury Cortex for the first time...");
                std::fs::create_dir_all(&data_dir)?;
            }
            State::Partial => {
                println!("Completing Mercury Cortex setup...");
                std::fs::create_dir_all(&data_dir)?;
            }
            State::Ready => {
                println!("Mercury Cortex is already configured – verifying setup...");
            }
        }
    }

    let client = CoreClient::open()?;

    if json {
        eprintln!("{}", json!({ "status": "running", "phase": "migrations" }));
    }
    client
        .database()
        .migrate(|display_name| {
            if !json {
                println!("  ✓ Applied migration: {display_name}");
            }
        })
        .await?;
    client.database().verify_schema().await?;

    if json {
        println!(
            "{}",
            json!({
                "status": "ok",
                "message": "Mercury Cortex setup complete",
                "db_path": db_path.display().to_string()
            })
        );
    } else {
        match state {
            State::Fresh => println!("✓ Mercury Cortex setup complete – {}", db_path.display()),
            _ => println!("✓ Mercury Cortex is ready – {}", db_path.display()),
        }
    }
    Ok(())
}

enum State {
    Fresh,
    Partial,
    Ready,
}

fn detect_state(data_dir: &Path, db_path: &Path) -> State {
    if db_path.exists() {
        State::Ready
    } else if data_dir.exists() {
        State::Partial
    } else {
        State::Fresh
    }
}
