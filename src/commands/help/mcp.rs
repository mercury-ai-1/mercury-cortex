//! Help text for `mcp`, `mcp serve`, and `mcp stop`.

pub const MCP_ABOUT: &str = "Manage the Model Context Protocol (MCP) server";
pub const MCP_LONG: &str = "\
Runs and manages the MCP server, which exposes Mercury Cortex tools and
knowledge to MCP clients such as OpenCode and Claude.

The server runs over stdio and holds the database lock while it is running.";
pub const MCP_EXAMPLES: &str = "\
Examples:
  mercury-cortex mcp serve
  mercury-cortex mcp stop";

pub const SERVE_ABOUT: &str = "Start the MCP server over stdio";
pub const SERVE_LONG: &str = "\
Starts the MCP server over standard input/output with its own runtime. Only
one such process may run at a time because SurrealKV uses exclusive file
locking.

The server answers the initialize handshake and tool listings immediately;
tool calls block until the knowledge engine is ready. Stops on EOF or
SIGTERM/SIGINT/SIGHUP.";
pub const SERVE_EXAMPLES: &str = "\
Examples:
  mercury-cortex mcp serve";

pub const STOP_ABOUT: &str = "Stop all running MCP servers";
pub const STOP_LONG: &str = "\
Stops every running `mercury-cortex mcp serve` process, releasing the database
lock so commands like `db reset` and `db backup` can run.

Only processes whose command line matches the MCP server are touched; unrelated
processes are never signalled.";
pub const STOP_EXAMPLES: &str = "\
Examples:
  mercury-cortex mcp stop";
