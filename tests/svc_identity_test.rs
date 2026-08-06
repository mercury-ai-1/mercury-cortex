#![cfg(unix)]

use std::process::Command;

use mercury_cortex::svc::{ServiceIdentity, verify_process};

/// A live child whose command line we control.
fn spawn_sleep() -> std::process::Child {
    Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn sleep")
}

#[test]
fn verify_process_matches_known_pattern() {
    let mut child = spawn_sleep();
    let ident = ServiceIdentity {
        name: "test",
        command_pattern: "sleep",
    };
    assert!(verify_process(child.id(), &ident).unwrap());
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn verify_process_rejects_dead_pid() {
    let ident = ServiceIdentity {
        name: "test",
        command_pattern: "sleep",
    };
    assert!(!verify_process(999_999_999, &ident).unwrap());
}

#[test]
fn verify_process_rejects_foreign_command() {
    let mut child = spawn_sleep();
    let ident = ServiceIdentity {
        name: "mcp",
        command_pattern: "mercury-cortex mcp serve",
    };
    assert!(!verify_process(child.id(), &ident).unwrap());
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn verify_process_missing_ps_returns_false() {
    let ident = ServiceIdentity {
        name: "test",
        command_pattern: "sleep",
    };
    // A huge PID makes `ps -p <pid>` exit non-zero → Ok(false), never an error.
    assert!(!verify_process(999_999_998, &ident).unwrap());
}
