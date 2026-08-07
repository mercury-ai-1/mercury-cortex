//! `mercury-cortex db export`: export table data to JSON files.

use std::path::PathBuf;

use anyhow::bail;
use clap::Args;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, MultiSelect, Select};
use mercury_cortex_core::client::{CoreClient, ExportFilter};

use crate::commands::help;

/// Arguments for `mercury-cortex db export`.
#[derive(Args)]
pub struct ExportArgs {
    /// Output directory for `<table>.json` files.
    #[arg(
        long,
        value_name = "DIR",
        default_value = ".",
        long_help = help::EXPORT_OUT_LONG
    )]
    pub out: PathBuf,

    /// Export only the named table(s); skips the interactive menu.
    #[arg(long, value_name = "NAME", long_help = help::EXPORT_TABLE_LONG)]
    pub table: Vec<String>,

    /// Export every table present in the database.
    #[arg(long, conflicts_with = "table", long_help = help::EXPORT_ALL_LONG)]
    pub all: bool,

    /// Filter rows on tables that have a `project_id` field.
    #[arg(long, value_name = "ID", long_help = help::EXPORT_PROJECT_ID_LONG)]
    pub project_id: Option<String>,

    /// List available tables and exit.
    #[arg(long, long_help = help::EXPORT_LIST_TABLES_LONG)]
    pub list_tables: bool,
}

pub async fn run(args: ExportArgs) -> Result<(), anyhow::Error> {
    let paths = CoreClient::paths()?;
    if !paths.db_path.exists() {
        bail!(
            "No database found at {}. Run `mercury-cortex setup` first.",
            paths.db_path.display()
        );
    }

    let client = CoreClient::open()?;
    let database = client.database();
    let tables = database.list_tables().await?;

    if args.list_tables {
        if tables.is_empty() {
            println!("No tables found.");
        } else {
            for table in &tables {
                println!("{table}");
            }
        }
        return Ok(());
    }

    let filters: Vec<ExportFilter> = match &args.project_id {
        Some(id) => vec![ExportFilter::record("project_id", id)?],
        None => Vec::new(),
    };

    let interactive = args.table.is_empty() && !args.all;
    let targets = if interactive {
        select_tables(&tables)?
    } else {
        resolve_targets(&tables, &args.table, args.all)?
    };

    if targets.is_empty() {
        println!("No tables selected; nothing to export.");
        return Ok(());
    }

    if interactive {
        print_summary(&database, &targets).await?;
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Export these tables?")
            .default(false)
            .interact()?;
        if !confirmed {
            println!("Aborted.");
            return Ok(());
        }
        println!("Exporting…");
    }

    let summary = database.export(&targets, &filters, &args.out).await?;

    for file in &summary.files {
        println!("{} ({} rows)", file.filename, file.rows);
    }
    for skip in &summary.skipped_filters {
        println!("{skip}");
    }
    println!(
        "wrote {} file(s) to {} in {}ms",
        summary.files.len(),
        args.out.display(),
        summary.duration_ms
    );

    Ok(())
}

/// Interactive table selection: "Export all tables" or "Choose tables",
/// mirroring `db reset`'s `prompt_mode`.
fn select_tables(tables: &[String]) -> Result<Vec<String>, anyhow::Error> {
    if tables.is_empty() {
        println!("No tables found; nothing to export.");
        return Ok(Vec::new());
    }
    let options = ["Export all tables", "Choose tables"];
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Export mode")
        .items(options)
        .default(0)
        .interact()?;

    if idx == 0 {
        return Ok(tables.to_vec());
    }

    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Tables to export (space to select, enter to confirm)")
        .items(tables)
        .interact()?
        .into_iter()
        .map(|i| tables[i].clone())
        .collect();
    Ok(selected)
}

/// Resolve targets for non-interactive `--table` / `--all` mode, failing on
/// unknown table names before anything is exported.
fn resolve_targets(
    tables: &[String],
    selected: &[String],
    all: bool,
) -> Result<Vec<String>, anyhow::Error> {
    if all {
        return Ok(tables.to_vec());
    }
    let mut unknown: Vec<&str> = selected
        .iter()
        .filter(|t| !tables.contains(t))
        .map(String::as_str)
        .collect();
    unknown.sort();
    if !unknown.is_empty() {
        bail!("unknown table(s): {}", unknown.join(", "));
    }
    Ok(selected.to_vec())
}

/// Print the pre-export "Export summary:" table with row counts (same layout
/// as `db reset`).
async fn print_summary(
    database: &mercury_cortex_core::client::DatabaseClient<'_>,
    targets: &[String],
) -> Result<(), anyhow::Error> {
    let counts = database.table_counts(targets).await?;
    println!();
    println!("Export summary:");
    for (table, count) in &counts {
        println!("  {table:<16} {count} record(s)");
    }
    println!();
    Ok(())
}
