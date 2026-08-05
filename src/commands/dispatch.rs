//! CLI command definitions and dispatch.
//!
//! Each subcommand lives in its own file under this module.
//! [`Commands`] is the top-level enum consumed by `main.rs`.

use clap::Subcommand;

use super::{daemon, db, help, mcp, profile, project, setup, version};

/// All top-level subcommands for the `mercury-cortex` CLI.
#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = help::SETUP_ABOUT,
        long_about = help::SETUP_LONG,
        after_long_help = help::SETUP_EXAMPLES,
    )]
    Setup,
    #[command(
        about = help::MIGRATION_ABOUT,
        long_about = help::MIGRATION_LONG,
        after_long_help = help::MIGRATION_EXAMPLES,
    )]
    Migration,
    #[command(
        about = help::PROFILE_ABOUT,
        long_about = help::PROFILE_LONG,
        after_long_help = help::PROFILE_EXAMPLES,
    )]
    Profile,
    #[command(
        about = help::MCP_ABOUT,
        long_about = help::MCP_LONG,
        after_long_help = help::MCP_EXAMPLES,
    )]
    MCP {
        #[command(subcommand)]
        command: mcp::McpCommand,
    },
    #[command(
        about = help::PROJECT_ABOUT,
        long_about = help::PROJECT_LONG,
        after_long_help = help::PROJECT_EXAMPLES,
    )]
    Project,
    #[command(
        about = help::DAEMON_ABOUT,
        long_about = help::DAEMON_LONG,
        after_long_help = help::DAEMON_EXAMPLES,
    )]
    Daemon {
        #[command(flatten)]
        args: daemon::DaemonArgs,
        #[command(subcommand)]
        command: Option<daemon::DaemonCommand>,
    },
    #[command(
        about = help::DB_ABOUT,
        long_about = help::DB_LONG,
        after_long_help = help::DB_EXAMPLES,
    )]
    Db {
        #[command(subcommand)]
        command: db::DbCommand,
    },
    #[command(
        about = help::VERSION_ABOUT,
        long_about = help::VERSION_LONG,
        after_long_help = help::VERSION_EXAMPLES,
    )]
    Version,
}

/// Route a parsed subcommand to its handler.
pub async fn dispatch(command: Commands, json: bool) -> Result<(), anyhow::Error> {
    match command {
        Commands::Setup => setup::run(json).await,
        Commands::Migration => db::migration::run().await,
        Commands::Profile => profile::run(json).await,
        Commands::MCP { command } => mcp::run(command).await,
        Commands::Project => project::run().await,
        Commands::Daemon { args, command } => daemon::run(args, command).await,
        Commands::Db { command } => db::run(command).await,
        Commands::Version => version::run(json),
    }
}
