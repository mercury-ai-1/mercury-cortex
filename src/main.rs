use clap::Parser;
use tracing_subscriber::EnvFilter;

use mercury_cortex::commands::help;

#[derive(Parser)]
#[command(
    name = "mercury-cortex",
    version,
    about = help::ROOT_ABOUT,
    long_about = help::ROOT_LONG,
    after_long_help = help::ROOT_EXAMPLES,
)]
struct Cli {
    #[command(subcommand)]
    command: mercury_cortex::commands::Commands,

    /// Enable verbose (debug) logging.
    #[arg(short, long, global = true, long_help = help::VERBOSE_LONG)]
    verbose: bool,

    /// Output in JSON format for machine-readable consumption.
    #[arg(long, global = true, long_help = help::JSON_LONG)]
    json: bool,

    /// Log output format: "text" (default) or "json".
    #[arg(
        long,
        global = true,
        default_value_t = String::from("text"),
        env = "MERCURY_LOG_FORMAT",
        long_help = help::LOG_FORMAT_LONG
    )]
    log_format: String,
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    let default_level = if cli.verbose {
        "mercury_cortex=debug,off"
    } else {
        "mercury_cortex=info,off"
    };
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| default_level.into()),
        );
    if cli.log_format == "json" {
        subscriber.json().init();
    } else {
        subscriber.init();
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Error: failed to start runtime: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };

    let result = rt.block_on(mercury_cortex::commands::dispatch(cli.command, cli.json));

    // Don't let Runtime drop block on in-flight blocking tasks (e.g. a
    // blocking stdin read that `mcp serve` leaves pending while a client
    // holds the pipe open). Exiting without waiting is safe here: spawned
    // work dies with the process.
    rt.shutdown_background();

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            // {e:#} prints the full anyhow context chain so the user sees
            // where the error originated.
            tracing::error!("Error: {e:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
