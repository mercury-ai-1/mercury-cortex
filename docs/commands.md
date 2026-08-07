# Commands Reference

Complete reference for all `mercury-cortex` CLI commands and flags.

## Global Flags

These flags apply to every command:

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Print help for the current command (use `--help` for detailed output) |
| `-v`, `--verbose` | Enable debug logging for the `mercury_cortex` crates |
| `--json` | Output in JSON format (machine-readable) |
| `--log-format <FORMAT>` | Log format: `text` (default) or `json`. Also set via `MERCURY_LOG_FORMAT` |

---

## `mercury-cortex setup`

Initialize the Mercury Cortex environment.

Creates the data directories, connects to the SurrealKV database, and applies any pending schema migrations. Safe to run repeatedly, since every step is idempotent and applied migrations are tracked so nothing runs twice.

On a fresh machine this fully initializes the environment; on later runs it verifies the existing setup.

```bash
mercury-cortex setup
```

---

## `mercury-cortex migration`

Apply pending database migrations.

Applies schema migrations to an already-initialized environment without re-running the full setup flow. Errors if no database exists yet; run `mercury-cortex setup` first.

```bash
mercury-cortex migration
```

---

## `mercury-cortex profile`

Create or update your user profile.

Walks you through creating or updating your Mercury Cortex user profile interactively, including your name, email, and agent name.

```bash
mercury-cortex profile
```

---

## `mercury-cortex project`

Initialize the current directory as a Mercury Cortex project.

Sets up the current directory as a Mercury Cortex project: creates the `.mercury-cortex/` structure and registers the project in the global database.

```bash
cd my-project
mercury-cortex project
```

---

## `mercury-cortex mcp`

Manage the Model Context Protocol (MCP) server.

The server runs over stdio and holds the database lock while it is running.

### `mercury-cortex mcp serve`

Start the MCP server over stdio.

Starts the MCP server over standard input/output with its own runtime. Only one such process may run at a time because SurrealKV uses exclusive file locking.

The server answers the initialize handshake and tool listings immediately; tool calls block until the knowledge engine is ready. Stops on EOF or SIGTERM/SIGINT/SIGHUP.

```bash
mercury-cortex mcp serve
```

### `mercury-cortex mcp stop`

Stop all running MCP servers.

Stops every running `mercury-cortex mcp serve` process, releasing the database lock so commands like `db reset` and `db backup` can run. Only processes whose command line matches the MCP server are touched; unrelated processes are never signalled.

```bash
mercury-cortex mcp stop
```

---

## `mercury-cortex daemon`

Start and manage the Mercury Cortex daemon (IPC server).

Starts the Mercury Cortex runtime and IPC server in a single long-lived process. The daemon owns the database and knowledge engine; commands such as `profile` and `project` connect to it over IPC. Runs until it receives a shutdown signal (SIGINT, SIGTERM, or SIGHUP).

```bash
mercury-cortex daemon
```

### `mercury-cortex daemon serve`

Start the daemon (IPC server). This is the default when no subcommand is given.

```bash
mercury-cortex daemon serve
```

#### Options

| Flag | Description |
|------|-------------|
| `--shutdown-timeout <SECONDS>` | How long to wait for graceful shutdown before exiting (default: 30, max: 600) |

### `mercury-cortex daemon stop`

Stop a running daemon.

Stops the running `mercury-cortex daemon` process, releasing the database lock so commands like `db reset` and `db backup` can run. Only processes whose command line matches the daemon are touched; unrelated processes are never signalled.

```bash
mercury-cortex daemon stop
```

---

## `mercury-cortex db`

Database maintenance: create timestamped backups, list them, restore the database from a backup, reset table data, and export tables to JSON. Backup, restore, and reset refuse to run while the daemon or MCP server holds the database lock.

```bash
mercury-cortex db backup
mercury-cortex db reset
mercury-cortex db export --all
```

### `mercury-cortex db backup`

Create a timestamped backup of the database.

Copies the SurrealKV database directory into `~/.mercury/cortex/backups/` with a timestamped name. Refuses to run while the database lock is held, since a directory copy of a live database is not guaranteed consistent.

```bash
mercury-cortex db backup
```

### `mercury-cortex db list`

List available database backups.

Lists the timestamped backups under `~/.mercury/cortex/backups/` together with their sizes.

```bash
mercury-cortex db list
```

### `mercury-cortex db restore`

Restore the database from a backup.

Replaces the current database with a copy of the given backup directory. Refuses to run while the database lock is held. The current database is removed and replaced.

```bash
mercury-cortex db restore <BACKUP_DIR>
```

#### Arguments

| Argument | Description |
|----------|-------------|
| `<BACKUP_DIR>` | Path to the backup directory to restore from (e.g. one listed by `db list`) |

### `mercury-cortex db reset`

Clear data from one or all database tables.

Clears all records from one or more schema tables. Prompts for confirmation before deleting anything. Refuses to run while the database lock is held.

```bash
mercury-cortex db reset
```

### `mercury-cortex db export`

Export table data to JSON files.

Exports one or more tables to `<table>.json` files in the output directory. Use `--table` to pick tables, `--all` for every table, or run without either flag to choose interactively. Output is deterministic (rows sorted by record id, keys in database order) so exports can be reviewed and committed.

```bash
mercury-cortex db export
mercury-cortex db export --all
mercury-cortex db export --table projects --table file_data
mercury-cortex db export --all --project-id projects:p1
mercury-cortex db export --list-tables
```

#### Options

| Flag | Description |
|------|-------------|
| `--out <DIR>` | Output directory for `<table>.json` files. Created if missing; existing files are overwritten. Defaults to the current directory. |
| `--table <NAME>` | Export only the named table(s). Repeatable. Skips the interactive selection menu. Mutually exclusive with `--all`. |
| `--all` | Export every table present in the database. Skips the interactive selection menu. Mutually exclusive with `--table`. |
| `--project-id <ID>` | Only export rows whose `project_id` equals this record id (e.g. `projects:p1`). Applied to every exported table that has a `project_id` field; tables without one are exported unfiltered. |
| `--list-tables` | Print the names of the tables present in the database (excluding `_`-prefixed internal tables) and exit. |

---

## `mercury-cortex version`

Show the installed Mercury Cortex version and installation paths.

Prints the installed version together with the binary and data directory paths. Use `--json` for machine-readable output.

```bash
mercury-cortex version
```
