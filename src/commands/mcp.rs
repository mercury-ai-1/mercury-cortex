use super::help;

use clap::Subcommand;

use mercury_cortex_core::db;
use mercury_cortex_core::runtime::status::RuntimePhase;
use mercury_cortex_core::runtime::{Runtime, RwLockExt, wait_shutdown_signal};
use rmcp::serve_server;

use crate::mcp::context::{LazyContext, McpContext};
use crate::mcp::handler::McpHandler;
use crate::mcp::transport::mcp_stdio;
use crate::svc::{PidFileGuard, ServiceIdentity, stop as svc_stop};

#[derive(Subcommand)]
pub enum McpCommand {
    #[command(
        about = help::SERVE_ABOUT,
        long_about = help::SERVE_LONG,
        after_long_help = help::SERVE_EXAMPLES,
    )]
    Serve,
    #[command(
        about = help::STOP_ABOUT,
        long_about = help::STOP_LONG,
        after_long_help = help::STOP_EXAMPLES,
    )]
    Stop,
}

/// Start the MCP server over stdio.
///
/// Spawns its own Runtime — only one such process may run at a time because
/// `SurrealKV` uses exclusive file locking.  The MCP server starts
/// *immediately* so the initialize handshake, `ping`, and `tools/list`
/// respond instantly.  Tool calls block on [`LazyContext::get`] until the
/// engine is ready.
pub async fn run(command: McpCommand) -> Result<(), anyhow::Error> {
    match command {
        McpCommand::Serve => serve_direct().await,
        McpCommand::Stop => stop().await,
    }
}

async fn serve_direct() -> Result<(), anyhow::Error> {
    let lazy = LazyContext::new();
    let handler = McpHandler { ctx: lazy.clone() };
    let transport = mcp_stdio();

    let mcp_handle = tokio::spawn(async move {
        match serve_server(handler, transport).await {
            Ok(service) => {
                if let Err(e) = service.waiting().await {
                    tracing::error!("MCP server task failed: {e}");
                }
            }
            Err(e) => {
                tracing::error!("MCP server initialization failed: {e}");
            }
        }
    });

    let rt = Runtime::new()
        .await
        .map_err(|e| anyhow::anyhow!("Mercury Cortex failed to start: {e}"))?;
    let phase = rt.ctx.status.read_result()?.phase;
    if phase == RuntimePhase::Failed {
        let reactor = rt.ctx.status.read_result()?;
        match reactor.error {
            Some(ref err) => anyhow::bail!(
                "Mercury Cortex failed to start\n  Error: {}\n  Recovery: {}",
                err.message,
                err.recovery
            ),
            None => anyhow::bail!("Mercury Cortex failed to start (unknown error)"),
        }
    }

    let ctx_engine = rt.engine()?;
    let ctx = McpContext::new(ctx_engine, rt.ctx.clone());
    lazy.set(ctx);

    // Advertise this process so `mcp stop` can find it. Removed on Drop.
    let _pid_guard = PidFileGuard::acquire(&db::data_dir()?, "mcp")?;

    // Wait for either the stdio session to end or a termination signal.
    // The runtime installs its own SIGINT/SIGTERM/SIGHUP handlers that stop
    // the engine; our listener exists purely so this process actually exits
    // (the runtime handler alone leaves `mcp_handle` blocking forever).
    tokio::select! {
        _ = mcp_handle => {
            tracing::info!("MCP stdio session ended");
        }
        _ = wait_shutdown_signal() => {
            tracing::info!("shutdown signal received; stopping MCP server");
        }
    }

    // Idempotent with the runtime's own trigger_shutdown (guards on
    // Stopping/Stopped phase), so racing it is harmless.
    rt.shutdown().await;

    Ok(())
}

/// Stop all running `mcp serve` processes.
async fn stop() -> Result<(), anyhow::Error> {
    let data_dir = db::data_dir()?;
    let ident = ServiceIdentity {
        name: "mcp",
        command_pattern: "mercury-cortex mcp serve",
    };
    let outcome = svc_stop(&ident, &data_dir).await?;
    println!("{}", outcome.message());
    Ok(())
}
