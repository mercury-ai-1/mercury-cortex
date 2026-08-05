use std::process::Command;

fn help_output(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_mercury-cortex"))
        .args(args)
        .arg("--help")
        .output()
        .expect("binary should run");
    assert!(output.status.success(), "`{args:?} --help` should exit 0");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn short_help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_mercury-cortex"))
        .args(args)
        .arg("-h")
        .output()
        .expect("binary should run");
    assert!(output.status.success(), "`{args:?} -h` should exit 0");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Every command and subcommand that must have two-tier help.
const COMMAND_PATHS: &[&[&str]] = &[
    &[],
    &["setup"],
    &["migration"],
    &["profile"],
    &["mcp"],
    &["mcp", "serve"],
    &["mcp", "stop"],
    &["project"],
    &["daemon"],
    &["daemon", "serve"],
    &["daemon", "stop"],
    &["db"],
    &["db", "backup"],
    &["db", "list"],
    &["db", "restore"],
    &["db", "reset"],
    &["db", "export"],
    &["version"],
];

#[test]
fn every_command_has_help() {
    for path in COMMAND_PATHS {
        let out = help_output(path);
        assert!(
            !out.trim().is_empty(),
            "`{path:?} --help` should be non-empty"
        );
    }
}

#[test]
fn short_help_is_shorter_than_long_help() {
    for path in COMMAND_PATHS {
        let short = short_help(path).len();
        let long = help_output(path).len();
        assert!(
            long > short,
            "`{path:?}` --help ({long} chars) must be longer than -h ({short} chars)"
        );
    }
}

#[test]
fn short_help_has_no_examples_section() {
    for path in COMMAND_PATHS {
        let short = short_help(path);
        assert!(
            !short.contains("Examples:"),
            "`{path:?} -h` must not contain an Examples section:\n{short}"
        );
    }
}

#[test]
fn long_help_includes_examples() {
    for path in COMMAND_PATHS {
        let long = help_output(path);
        assert!(
            long.contains("Examples:"),
            "`{path:?} --help` should include an Examples section:\n{long}"
        );
    }
}

#[test]
fn root_help_lists_all_commands() {
    let root = help_output(&[]);
    for cmd in [
        "setup",
        "migration",
        "profile",
        "mcp",
        "project",
        "daemon",
        "db",
        "version",
    ] {
        assert!(root.contains(cmd), "root help should list `{cmd}`");
    }
}

#[test]
fn db_help_lists_subcommands() {
    let db_help = help_output(&["db"]);
    for sub in ["backup", "list", "restore", "reset", "export"] {
        assert!(db_help.contains(sub), "db help should list `{sub}`");
    }
}

#[test]
fn mcp_help_lists_serve_and_stop() {
    let mcp_help = help_output(&["mcp"]);
    assert!(mcp_help.contains("serve"), "mcp help should list serve");
    assert!(mcp_help.contains("stop"), "mcp help should list stop");
}

#[test]
fn daemon_help_lists_serve_and_stop() {
    let daemon_help = help_output(&["daemon"]);
    assert!(
        daemon_help.contains("serve"),
        "daemon help should list serve"
    );
    assert!(daemon_help.contains("stop"), "daemon help should list stop");
}

#[test]
fn root_help_shows_root_description() {
    let root = help_output(&[]);
    assert!(
        root.contains("knowledge"),
        "root help should show the root description"
    );
}
