//! The `mercury-cortex db` command group — database maintenance operations.
//!
//! Each subcommand routes to the `DatabaseClient` facade; this module only
//! formats and prints results. `reset` keeps its interactive prompt layer in
//! `reset.rs`, `migration` applies schema migrations.

mod export;
pub(super) mod migration;
mod reset;

use std::path::PathBuf;

use clap::Subcommand;

use mercury_cortex_core::client::CoreClient;

use super::db::reset::run as run_reset;
use super::help;

/// Database maintenance operations — backups, restores, and table resets.
#[derive(Subcommand)]
pub enum DbCommand {
    /// Create a timestamped backup of the `SurrealKV` database.
    #[command(
        about = help::BACKUP_ABOUT,
        long_about = help::BACKUP_LONG,
        after_long_help = help::BACKUP_EXAMPLES,
    )]
    Backup,
    /// List available database backups.
    #[command(
        about = help::LIST_ABOUT,
        long_about = help::LIST_LONG,
        after_long_help = help::LIST_EXAMPLES,
    )]
    List,
    /// Restore the database from a backup.
    #[command(
        about = help::RESTORE_ABOUT,
        long_about = help::RESTORE_LONG,
        after_long_help = help::RESTORE_EXAMPLES,
    )]
    Restore {
        /// Path to the backup directory to restore from.
        #[arg(long_help = help::RESTORE_PATH_LONG)]
        path: PathBuf,
    },
    /// Clear data from one or all database tables.
    #[command(
        about = help::RESET_ABOUT,
        long_about = help::RESET_LONG,
        after_long_help = help::RESET_EXAMPLES,
    )]
    Reset,
    /// Export table data to JSON files.
    #[command(
        about = help::EXPORT_ABOUT,
        long_about = help::EXPORT_LONG,
        after_long_help = help::EXPORT_EXAMPLES,
    )]
    Export {
        #[command(flatten)]
        args: export::ExportArgs,
    },
}

/// Execute a database subcommand (backup, list, restore, reset, migration).
pub async fn run(command: DbCommand) -> Result<(), anyhow::Error> {
    match command {
        DbCommand::Backup => {
            let client = CoreClient::open()?;
            let result = client.database().backup()?;
            println!("Backup created: {}", result.path.display());
            println!("  Size: {} bytes", result.size);
            Ok(())
        }
        DbCommand::List => {
            let client = CoreClient::open()?;
            let list = client.database().list_backups()?;
            if list.missing {
                println!("No backups found (directory does not exist).");
            } else if list.entries.is_empty() {
                println!("No backups found in {}", list.dir.display());
            } else {
                println!("Available backups ({}):", list.dir.display());
                for entry in list.entries {
                    println!("  {}  {} bytes", entry.name, entry.size);
                }
            }
            Ok(())
        }
        DbCommand::Restore { path } => {
            let client = CoreClient::open()?;
            let result = client.database().restore(&path)?;
            println!("Database restored from: {}", result.backup_path.display());
            println!("  To: {}", result.db_path.display());
            Ok(())
        }
        DbCommand::Reset => run_reset().await,
        DbCommand::Export { args } => export::run(args).await,
    }
}
