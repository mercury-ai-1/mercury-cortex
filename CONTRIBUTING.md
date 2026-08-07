# Contributing to Mercury Cortex

Thank you for your interest in contributing!

## Getting Started

### Prerequisites

- Rust 1.85+ (edition 2024)
- The `mercury-cortex-core` library cloned as a sibling directory

### Clone

```bash
git clone https://github.com/mercury-ai-1/mercury-cortex.git
git clone https://github.com/mercury-ai-1/mercury-cortex-core.git ../mercury-cortex-core
```

### Build

```bash
cargo build
```

### Test

```bash
cargo test
cargo clippy -- -D warnings
```

### Cross-repo Testing

This crate depends on `mercury-cortex-core` via path dependency. Core changes should be tested together:

```bash
cargo test --manifest-path ../mercury-cortex-core/Cargo.toml
cargo test
```

## Project Structure

```
src/
  commands/       CLI subcommand handlers
    help/         Externalized help text for each command
  mcp/            MCP server implementation
    tools/        Tool handlers called by AI assistants
    prompts/      Workflow prompt handlers
  ipc/            Unix socket IPC client/server/protocol
  svc/            Process service management (pidfile, identity, signal, stop)
tests/            Integration tests (26 test files)
```

## Development Workflow

1. Create a branch from `main`
2. Make your changes
3. Run `cargo test` and `cargo clippy -- -D warnings`
4. Ensure help text is updated if you added or changed a command
5. Submit a pull request

## Help Text

Every command and subcommand has externalized help text in `src/commands/help/`. When adding or changing a command:

1. Add or update the help text in the appropriate help module
2. Add or update the corresponding help test in `tests/help_audit_test.rs`

Tests enforce that help text exists for all commands; the build will fail without it.

## Testing

- **Unit tests**: Inline `#[cfg(test)]` modules within `src/` files
- **Integration tests**: The `tests/` directory contains 26 test files
- **CLI tests**: Use `std::process::Command` to run the binary and assert on output
- **IPC tests**: Use temporary directories and temp SurrealDB instances
- **MCP tests**: Use the `create_test_context()` helper for isolated contexts

## Commit Messages

Use [conventional commits](https://www.conventionalcommits.org/):

- `feat:` for new features
- `fix:` for bug fixes
- `refactor:` for code restructuring without behavior change
- `test:` for adding or updating tests
- `docs:` for documentation changes
- `chore:` for maintenance tasks

## Code Style

- Follow existing code patterns in the crate
- Keep functions focused and small
- Name variables and functions clearly; prefer explicit over clever
- Use `thiserror` for error types

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
