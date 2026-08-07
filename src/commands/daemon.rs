//! The `mercury-cortex daemon` command handler.
//!
//! Starts the Mercury Cortex runtime and IPC server in a single long-lived
//! process. Other processes (e.g. `mcp serve`) connect to the daemon via IPC.

use clap::{Args, Subcommand};

use super::help;
use crate::ipc;
use crate::svc::{PidFileGuard, ServiceIdentity, stop as svc_stop};
use mercury_cortex_core::db;
use mercury_cortex_core::runtime::status::RuntimePhase;
use mercury_cortex_core::runtime::{Runtime, RwLockExt, wait_shutdown_signal};

/// Arguments for serving the daemon.
#[derive(Debug, Args)]
pub struct DaemonArgs {
    /// Graceful shutdown timeout in seconds.
    #[arg(long, long_help = help::SHUTDOWN_TIMEOUT_LONG)]
    pub shutdown_timeout: Option<u64>,
}

/// The `daemon` subcommands.
#[derive(Debug, Subcommand)]
pub enum DaemonCommand {
    /// Start the daemon (default when no subcommand is given).
    #[command(
        about = help::DAEMON_SERVE_ABOUT,
        long_about = help::DAEMON_SERVE_LONG,
        after_long_help = help::DAEMON_SERVE_EXAMPLES,
    )]
    Serve,
    /// Stop a running daemon.
    #[command(
        about = help::DAEMON_STOP_ABOUT,
        long_about = help::DAEMON_STOP_LONG,
        after_long_help = help::DAEMON_STOP_EXAMPLES,
    )]
    Stop,
}

/// Start the Mercury Cortex daemon.
///
/// Creates a single Runtime (owning the database and knowledge engine), then
/// starts the IPC server on the configured platform endpoint (Unix socket or
/// TCP loopback).  The daemon runs until it receives a shutdown signal.
pub async fn run(args: DaemonArgs, command: Option<DaemonCommand>) -> Result<(), anyhow::Error> {
    match command {
        Some(DaemonCommand::Serve) | None => run_serve(args).await,
        Some(DaemonCommand::Stop) => stop().await,
    }
}

async fn run_serve(args: DaemonArgs) -> Result<(), anyhow::Error> {
    let shutdown_timeout = args.shutdown_timeout.unwrap_or(30);
    if shutdown_timeout == 0 {
        anyhow::bail!("--shutdown-timeout must be at least 1 second");
    }
    let shutdown_timeout = if shutdown_timeout > 600 {
        tracing::warn!(
            given = shutdown_timeout,
            capped = 600,
            "--shutdown-timeout capped at 600 seconds"
        );
        600
    } else {
        shutdown_timeout
    };

    let rt = Runtime::new()
        .await
        .map_err(|e| anyhow::anyhow!("Mercury Cortex failed to start: {e}"))?;
    let phase = rt.ctx.status.read_result()?.phase;
    if phase == RuntimePhase::Failed {
        let reactor = rt.ctx.status.read_result()?;
        match reactor.error {
            Some(ref err) => {
                anyhow::bail!(
                    "Mercury Cortex failed to start\n  Error: {}\n  Recovery: {}",
                    err.message,
                    err.recovery
                );
            }
            None => anyhow::bail!("Mercury Cortex failed to start (unknown error)"),
        }
    }

    // Start IPC server for other processes to communicate.
    let ipc_handle = ipc::server::start(rt.ctx.clone()).await?;

    // Advertise this process so `daemon stop` can find it. Removed on Drop.
    let data_dir = db::data_dir()?;
    let _pid_guard = PidFileGuard::acquire(&data_dir, "daemon")?;

    eprintln!();
    eprintln!("Mercury Cortex Daemon");
    eprintln!(
        "  Socket: {}",
        ipc::net::Endpoint::from_socket_path(&rt.ctx.config.socket_path).display()
    );
    eprintln!();

    // Wait for a termination signal. The runtime installs its own
    // SIGINT/SIGTERM/SIGHUP handlers that stop the engine; our listener
    // exists purely so this process actually exits (the runtime handler
    // alone leaves the daemon running forever).
    wait_shutdown_signal().await;
    tracing::info!("shutdown signal received; stopping daemon");

    // Give the engine time to stop gracefully, capped at shutdown_timeout.
    // Idempotent with the runtime's own trigger_shutdown.
    tokio::time::timeout(
        std::time::Duration::from_secs(shutdown_timeout),
        rt.shutdown(),
    )
    .await
    .ok();

    // Explicitly cancel the IPC accept loop; `shutdown_background()` in main
    // would also abort it, but this makes the shutdown self-contained. Killing
    // in-flight requests is fine; the engine is already stopped.
    ipc_handle.abort();

    Ok(())
}

/// Stop the running daemon.
async fn stop() -> Result<(), anyhow::Error> {
    let data_dir = db::data_dir()?;
    let ident = ServiceIdentity {
        name: "daemon",
        command_pattern: "mercury-cortex daemon",
    };
    let outcome = svc_stop(&ident, &data_dir).await?;
    println!("{}", outcome.message());
    Ok(())
}
