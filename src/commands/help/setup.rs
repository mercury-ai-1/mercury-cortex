//! Help text for `setup` and `migration`.

pub const SETUP_ABOUT: &str = "Initialize the Mercury Cortex environment";
pub const SETUP_LONG: &str = "\
Creates the Mercury Cortex data directories, connects to the SurrealKV
database, and applies any pending schema migrations.

Safe to run repeatedly: every step is idempotent, and applied migrations are
tracked so nothing runs twice. On a fresh machine this fully initializes the
environment; on later runs it verifies the existing setup.";
pub const SETUP_EXAMPLES: &str = "\
Examples:
  mercury-cortex setup";

pub const MIGRATION_ABOUT: &str = "Apply pending database migrations";
pub const MIGRATION_LONG: &str = "\
Applies schema migrations to an already-initialized environment without
re-running the full setup flow.

Errors if no database exists yet; run `mercury-cortex setup` first.";
pub const MIGRATION_EXAMPLES: &str = "\
Examples:
  mercury-cortex migration";
