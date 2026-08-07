//! Database reset: clear data from one or all schema tables.
//!
//! The transport-agnostic domain logic lives in the core `db::reset`
//! module; this module only provides the interactive terminal wrapper
//! ([`run`]) on top of the `DatabaseClient` facade.

use mercury_cortex_core::client::{CoreClient, ResetMode};

/// Interactive entry point for `mercury-cortex db reset`.
///
/// Refuses to run while the daemon holds the database lock, connects to the
/// database, lets the user pick tables (or all), shows a summary, and asks
/// for confirmation before clearing anything.
pub async fn run() -> Result<(), anyhow::Error> {
    let paths = CoreClient::paths()?;
    if !paths.db_path.exists() {
        anyhow::bail!(
            "No database found at {}. Run `mercury-cortex setup` first.",
            paths.db_path.display()
        );
    }

    let client = CoreClient::open()?;
    let database = client.database();

    if database.lock_is_held()? {
        anyhow::bail!(
            "Database is locked by a running process ({}). \
             Stop it before resetting the database.",
            paths.db_path.join("LOCK").display()
        );
    }

    let tables = database.list_resettable_tables().await?;
    if tables.is_empty() {
        println!("No schema tables found; nothing to reset.");
        return Ok(());
    }

    let mode = prompt_mode(&tables)?;
    let targets: Vec<String> = match &mode {
        ResetMode::All => tables.clone(),
        ResetMode::Selected(selected) => selected.clone(),
    };

    if targets.is_empty() {
        println!("No tables selected; nothing to reset.");
        return Ok(());
    }

    let counts = database.table_counts(&targets).await?;

    println!();
    println!("Reset summary:");
    for (table, count) in &counts {
        println!("  {table:<16} {count} record(s)");
    }
    println!();

    let confirmed = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Irreversible: clear this data?")
        .default(false)
        .interact()?;
    if !confirmed {
        println!("Aborted.");
        return Ok(());
    }

    let summary = database.reset(mode).await?;
    let total: u64 = summary.cleared.iter().map(|(_, n)| n).sum();
    println!(
        "Reset complete. Cleared {total} record(s) across {} table(s).",
        summary.cleared.len()
    );

    Ok(())
}

/// Let the user choose between clearing everything or specific tables.
fn prompt_mode(tables: &[String]) -> Result<ResetMode, anyhow::Error> {
    use dialoguer::theme::ColorfulTheme;
    use dialoguer::{MultiSelect, Select};

    let options = ["Clear all tables", "Choose tables"];
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Reset mode")
        .items(options)
        .default(0)
        .interact()?;

    if idx == 0 {
        return Ok(ResetMode::All);
    }

    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Tables to clear (space to select, enter to confirm)")
        .items(tables)
        .interact()?
        .into_iter()
        .map(|i| tables[i].clone())
        .collect();

    Ok(ResetMode::Selected(selected))
}
