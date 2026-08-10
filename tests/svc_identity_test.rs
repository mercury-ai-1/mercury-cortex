#![cfg(unix)]

use std::process::Command;
use std::time::{Duration, Instant};

use mercury_cortex::svc::{ServiceIdentity, verify_process};

/// A live child whose command line we control.
fn spawn_sleep() -> std::process::Child {
    Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn sleep")
}

/// Poll `verify_process` until it matches or `timeout` elapses.
///
/// A freshly spawned child may still be between `fork()` and `exec()` when
/// probed; on Linux, `/proc/PID/cmdline` then shows the parent's argv rather
/// than the pattern, so a single immediate probe can race CI. Wait for the
/// child to be verified instead of assuming exec already happened.
fn wait_until_verified(pid: u32, ident: &ServiceIdentity<'_>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if verify_process(pid, ident).unwrap_or(false) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn verify_process_matches_known_pattern() {
    let mut child = spawn_sleep();
    let ident = ServiceIdentity {
        name: "test",
        command_pattern: "sleep",
    };
    assert!(
        wait_until_verified(child.id(), &ident, Duration::from_secs(5)),
        "child command line never matched 'sleep' within 5s"
    );
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
