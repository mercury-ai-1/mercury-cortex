//! Help text for `daemon`, `daemon serve`, `daemon stop`, and the
//! `--shutdown-timeout` argument.

pub const DAEMON_ABOUT: &str = "Start and manage the Mercury Cortex daemon (IPC server)";
pub const DAEMON_LONG: &str = "\
Starts the Mercury Cortex runtime and IPC server in a single long-lived
process. The daemon owns the database and knowledge engine;
commands such as `profile` and `project` connect to it over IPC.

Runs until it receives a shutdown signal (SIGINT, SIGTERM, or SIGHUP).";
pub const DAEMON_EXAMPLES: &str = "\
Examples:
  mercury-cortex daemon
  mercury-cortex daemon stop";

pub const DAEMON_SERVE_ABOUT: &str = "Start the daemon (IPC server)";
pub const DAEMON_SERVE_LONG: &str = "\
Starts the Mercury Cortex runtime and IPC server in a single long-lived
process. The daemon owns the database and knowledge engine;
commands such as `profile` and `project` connect to it over IPC.

Runs until it receives a shutdown signal (SIGINT, SIGTERM, or SIGHUP).";
pub const DAEMON_SERVE_EXAMPLES: &str = "\
Examples:
  mercury-cortex daemon";

pub const DAEMON_STOP_ABOUT: &str = "Stop a running daemon";
pub const DAEMON_STOP_LONG: &str = "\
Stops the running `mercury-cortex daemon` process, releasing the database
lock so commands like `db reset` and `db backup` can run.

Only processes whose command line matches the daemon are touched; unrelated
processes are never signalled.";
pub const DAEMON_STOP_EXAMPLES: &str = "\
Examples:
  mercury-cortex daemon stop";

pub const SHUTDOWN_TIMEOUT_LONG: &str = "\
How long to wait for a graceful shutdown before exiting. Defaults to 30
seconds.";
