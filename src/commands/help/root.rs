//! Root CLI description and shared global-flag help text.

/// One-sentence root summary shown in `-h`.
pub const ROOT_ABOUT: &str = "Local-first knowledge management and development companion";

/// Root description shown in `--help`.
pub const ROOT_LONG: &str = "\
Mercury Cortex indexes your files and project metadata into a local knowledge
graph you can search and reuse. It ships a CLI, a background daemon, and an MCP
server for AI clients.

Subcommands are grouped by concern: setup and migration maintain the
environment, db manages backups and resets, mcp runs the MCP server, project
initializes projects, profile manages your user profile, daemon runs the
background process, and version reports installation details. The global
flags below apply to every subcommand.";

/// Worked invocations shown at the end of root `--help`.
pub const ROOT_EXAMPLES: &str = "\
Examples:
  mercury-cortex setup
  mercury-cortex mcp serve
  mercury-cortex db backup";

/// Detail for `-v/--verbose`, shown in `--help`.
pub const VERBOSE_LONG: &str = "\
Enable verbose (debug) logging.

Sets the log level to debug for the mercury_cortex crates.";

/// Detail for `--json`, shown in `--help`.
pub const JSON_LONG: &str = "\
Output in JSON format for machine-readable consumption.

Commands that support structured output (e.g. setup, version) print a JSON
document instead of human-readable text.";

/// Detail for `--log-format`, shown in `--help`.
pub const LOG_FORMAT_LONG: &str = "\
Log output format: \"text\" (default) or \"json\".

The default can be changed by setting MERCURY_LOG_FORMAT.";
