#![cfg(unix)]

use std::process::Command;
use std::time::Duration;

use tempfile::TempDir;

use mercury_cortex::svc::{PidFile, ServiceIdentity, StopOutcome, is_alive, stop, wait_for_exit};

const GRACE: Duration = Duration::from_secs(5);

/// Spawn `sleep 60` whose argv0 is `<marker>` so the scan matches only it.
fn spawn_marked(marker: &str) -> std::process::Child {
    Command::new("bash")
        .arg("-c")
        .arg(format!("exec -a {marker} sleep 60"))
        .spawn()
        .expect("spawn marked sleep")
}

fn ident(marker: &str) -> ServiceIdentity<'_> {
    ServiceIdentity {
        name: "test",
        command_pattern: marker,
    }
}

#[test]
fn stop_no_pidfile_returns_already_stopped() {
    let tmp = TempDir::new().unwrap();
    // Marker matches nothing → no scan results.
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&ident("mctest-none-1"), tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::AlreadyStopped);
}

#[test]
fn stop_stale_pid_removes_file() {
    let tmp = TempDir::new().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(999_999_999).unwrap();
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&ident("mctest-none-2"), tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::StalePidRemoved);
    assert!(!pf.path().exists(), "stale PID file must be removed");
}

#[test]
fn stop_foreign_pid_returns_identity_mismatch() {
    let tmp = TempDir::new().unwrap();
    // A live but foreign process: `sleep` does not match "mercury-cortex mcp serve".
    let mut child = Command::new("sleep").arg("60").spawn().unwrap();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(child.id()).unwrap();
    let foreign = ServiceIdentity {
        name: "test",
        command_pattern: "mercury-cortex mcp serve",
    };
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&foreign, tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::IdentityMismatch);
    assert!(
        pf.path().exists(),
        "PID file must be retained on identity mismatch"
    );
    assert!(is_alive(child.id()), "foreign process must not be killed");
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stop_terminates_matching_process() {
    let tmp = TempDir::new().unwrap();
    let mut child = spawn_marked("mctest-one");
    let pid = child.id();
    let pf = PidFile::new(tmp.path(), "test");
    pf.write(pid).unwrap();
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&ident("mctest-one"), tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::Stopped);
    assert!(!pf.path().exists(), "PID file must be removed after stop");
    let exited = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(wait_for_exit(pid, GRACE))
        .unwrap();
    assert!(exited, "matching process must exit after stop");
    let _ = child.wait();
}

#[test]
fn stop_outcome_messages_are_distinct() {
    let variants = [
        StopOutcome::AlreadyStopped,
        StopOutcome::StalePidRemoved,
        StopOutcome::Stopped,
        StopOutcome::ForceKilled,
        StopOutcome::IdentityMismatch,
        StopOutcome::Failed("boom".into()),
    ];
    let messages: Vec<String> = variants.iter().map(|v| v.message()).collect();
    for m in &messages {
        assert!(!m.trim().is_empty(), "message must not be empty: {m:?}");
    }
    let unique: std::collections::HashSet<&String> = messages.iter().collect();
    assert_eq!(
        unique.len(),
        variants.len(),
        "messages must be distinct: {messages:?}"
    );
}

#[test]
fn stop_ignores_stop_invocations() {
    let tmp = TempDir::new().unwrap();
    // A wrapper shell whose argv looks like `<service> stop` (e.g. running
    // `sh -c "mercury-cortex daemon stop"`) contains the service pattern but
    // is NOT the service; it must never be signalled.
    let mut child = Command::new("bash")
        .arg("-c")
        .arg("exec -a 'mctest-excl stop' sleep 60")
        .spawn()
        .unwrap();
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&ident("mctest-excl"), tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::AlreadyStopped);
    assert!(
        is_alive(child.id()),
        "stop-invocation process must not be killed"
    );
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn stop_scans_for_additional_processes() {
    let tmp = TempDir::new().unwrap();
    // Two matching processes with NO pid file; the scan must find and stop both.
    // Unique markers so this test's scan never touches other tests' processes.
    let mut a = spawn_marked("mctest-scan-a");
    let mut b = spawn_marked("mctest-scan-b");
    let outcome = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(stop(&ident("mctest-scan"), tmp.path()))
        .unwrap();
    assert_eq!(outcome, StopOutcome::Stopped);
    assert!(!is_alive(a.id()), "scanned process A must be stopped");
    assert!(!is_alive(b.id()), "scanned process B must be stopped");
    let _ = a.wait();
    let _ = b.wait();
}
