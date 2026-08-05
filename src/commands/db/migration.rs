//! The `mercury-cortex migration` command handler.
//!
//! Applies any pending migrations on an existing database without re-running
//! the full setup flow.  Exits early with an error if no database exists yet.

use mercury_cortex_core::client::CoreClient;

/// Apply all pending database migrations.
///
/// Checks that the database already exists (bails with a hint to run `setup`
/// first), connects, and delegates to `DatabaseClient::migrate`.
pub async fn run() -> Result<(), anyhow::Error> {
    let paths = CoreClient::paths()?;
    if !paths.db_path.exists() {
        anyhow::bail!("No database found. Run `mercury-cortex setup` first.");
    }

    println!("Checking for pending migrations...");
    let client = CoreClient::open()?;
    client
        .database()
        .migrate(|display_name| {
            println!("  ✓ Applied migration: {display_name}");
        })
        .await?;
    println!("✓ All migrations applied");
    Ok(())
}
