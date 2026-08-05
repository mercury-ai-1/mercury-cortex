# Security Policy

## Reporting Vulnerabilities

Please report security vulnerabilities through [GitHub Issues](https://github.com/mercury-ai-1/mercury-cortex/issues) or email if provided.

Do NOT disclose vulnerabilities publicly until a fix is available.

## Scope

This security policy covers `mercury-cortex` (the CLI and MCP server application).

For `mercury-cortex-core` library security, see the [core repository](https://github.com/mercury-ai-1/mercury-cortex-core).

## Threat Model

### Database Access

- SurrealDB uses `kv-surrealkv` with exclusive file locking
- Only one process may access the database at a time
- Database resides in `~/.mercury/` (user's home directory)

### Unix Socket IPC

- Daemon IPC uses Unix domain sockets in the data directory
- Socket permissions are restricted to the file system user
- PID files prevent multiple instances and enable clean shutdown

### Temp File Staging

- AI-generated metadata is staged in `.mercury-cortex/temp/`
- Files are imported once and the temp directory is cleaned up
- Path traversal in metadata is guarded by `join_within_root()`

### `.mcignore` Patterns

- Exclusion patterns prevent indexing of sensitive directories
- Patterns follow gitignore syntax
- Default exclusions: `target`, `build`, `.env`, `.git`

### Process Management

- `mcp stop` and `daemon stop` use process identity verification
- Signal handling uses `libc` wrappers (SIGTERM, SIGINT, SIGHUP)
- Graceful shutdown with configurable timeout (1-600 seconds)

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
| < 0.1   | No        |

Only the latest version receives security updates.
