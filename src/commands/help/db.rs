//! Help text for `db`, `db backup`, `db list`, `db restore`, `db reset`,
//! and `db export`.

pub const DB_ABOUT: &str = "Manage database backups, restores, resets, and exports";
pub const DB_LONG: &str = "\
Database maintenance: create timestamped backups, list them, restore the
database from a backup, reset table data, and export tables to JSON.

Backup, restore, and reset refuse to run while the daemon or MCP server holds
the database lock.";
pub const DB_EXAMPLES: &str = "\
Examples:
  mercury-cortex db backup
  mercury-cortex db reset
  mercury-cortex db export --all";

pub const BACKUP_ABOUT: &str = "Create a timestamped backup of the database";
pub const BACKUP_LONG: &str = "\
Copies the SurrealKV database directory into `~/.mercury/cortex/backups/` with
a timestamped name. Refuses to run while the database lock is held, since a
directory copy of a live database is not guaranteed consistent.";
pub const BACKUP_EXAMPLES: &str = "\
Examples:
  mercury-cortex db backup";

pub const LIST_ABOUT: &str = "List available database backups";
pub const LIST_LONG: &str = "\
Lists the timestamped backups under `~/.mercury/cortex/backups/` together with
their sizes.";
pub const LIST_EXAMPLES: &str = "\
Examples:
  mercury-cortex db list";

pub const RESTORE_ABOUT: &str = "Restore the database from a backup";
pub const RESTORE_LONG: &str = "\
Replaces the current database with a copy of the given backup directory.

Refuses to run while the database lock is held, since replacing a live
database directory would corrupt the running process's state. The current
database is removed and replaced.";
pub const RESTORE_EXAMPLES: &str = "\
Examples:
  mercury-cortex db restore ~/.mercury/cortex/backups/mercury_cortex_global_knowledge.db.1722547200";

pub const RESTORE_PATH_LONG: &str = "\
Path to the backup directory to restore from, e.g. one of the directories
listed by `mercury-cortex db list`.";

pub const RESET_ABOUT: &str = "Clear data from one or all database tables";
pub const RESET_LONG: &str = "\
Clears all records from one or more schema tables. Prompts for confirmation
before deleting anything.

Refuses to run while the database lock is held by a running process.";
pub const RESET_EXAMPLES: &str = "\
Examples:
  mercury-cortex db reset";

pub const EXPORT_ABOUT: &str = "Export table data to JSON files";
pub const EXPORT_LONG: &str = "\
Exports one or more tables to `<table>.json` files in the output directory.
Use `--table` to pick tables, `--all` for every table, or run without either
flag to choose interactively.

With `--project-id`, rows are filtered to the given project on every table
that has a `project_id` field; other tables are exported unfiltered.

Output is deterministic (rows sorted by record id, keys in database order) so
exports can be reviewed and committed.";
pub const EXPORT_EXAMPLES: &str = "\
Examples:
  mercury-cortex db export
  mercury-cortex db export --all
  mercury-cortex db export --table projects --table file_data
  mercury-cortex db export --all --project-id projects:p1
  mercury-cortex db export --list-tables";
pub const EXPORT_OUT_LONG: &str = "\
Directory to write `<table>.json` files to. Created if missing; existing
files are overwritten. Defaults to the current directory.";
pub const EXPORT_TABLE_LONG: &str = "\
Export only the named table(s). Repeatable. Skips the interactive selection
menu. Mutually exclusive with `--all`.";
pub const EXPORT_ALL_LONG: &str = "\
Export every table present in the database. Skips the interactive selection
menu. Mutually exclusive with `--table`.";
pub const EXPORT_PROJECT_ID_LONG: &str = "\
Only export rows whose `project_id` equals this record id (e.g.
`projects:p1`). Applied to every exported table that has a `project_id`
field; tables without one are exported unfiltered.";
pub const EXPORT_LIST_TABLES_LONG: &str = "\
Print the names of the tables present in the database (excluding `_`-prefixed
internal tables) and exit.";
