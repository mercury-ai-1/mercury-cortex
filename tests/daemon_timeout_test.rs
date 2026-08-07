#![cfg(unix)]

//! Graceful-shutdown semantics tests.
//!
//! These assert SIGTERM-based graceful stop, PID-file cleanup on Drop, and a
//! clean (success) exit; all of which are unix-only. Windows termination is a
//! hard `TerminateProcess` with no graceful shutdown analogue, so the guarded
//! assertions cannot hold there.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const EXIT_TIMEOUT: Duration = Duration::from_secs(60);

/// Ensure a spawned daemon is always killed and reaped, even on panic.
struct ChildGuard(Option<Child>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

#[tokio::test]
async fn shutdown_timeout_zero_fails_fast() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();
    unsafe { std::env::set_var("HOME", home) };

    let bin = env!("CARGO_BIN_EXE_mercury-cortex");
    let output = Command::new(bin)
        .args(["daemon", "--shutdown-timeout", "0", "serve"])
        .env("HOME", home)
        .output()?;

    assert!(!output.status.success(), "must exit non-zero");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--shutdown-timeout must be at least 1 second"),
        "stderr: {stderr}"
    );

    let socket = home.join(".mercury/cortex/runtime.sock");
    assert!(!socket.exists(), "must not leave the socket behind");
    let pidfile = home.join(".mercury/cortex/daemon.pid");
    assert!(!pidfile.exists(), "must not write a PID file");
    Ok(())
}

#[tokio::test]
async fn shutdown_timeout_over_600_clamps_with_warn() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();
    unsafe { std::env::set_var("HOME", home) };

    let bin = env!("CARGO_BIN_EXE_mercury-cortex");
    let mut child = Command::new(bin)
        .args(["daemon", "--shutdown-timeout", "700", "serve"])
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    // `Child` is not `Clone`, so take the stderr handle and move the child
    // into the guard; the guard owns it and reaps it on drop, while the test
    // polls its field with non-blocking `try_wait()` for a bounded exit wait.
    let stderr_handle = child.stderr.take();
    let pid = child.id();
    let mut guard = ChildGuard(Some(child));

    // Prove startup by waiting for the PID file (closest available marker to
    // signal-registration, matching daemon_stop_integration_test.rs); the cap
    // warning is emitted before the PID file appears, so its arrival implies
    // the warn.
    let pidfile = home.join(".mercury/cortex/daemon.pid");
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while !pidfile.exists() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        pidfile.exists(),
        "daemon did not write its PID file in time"
    );

    // SIGTERM (not child.kill(), which is SIGKILL) to exercise the graceful
    // shutdown path; mirrors core's runtime_signal_test.
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()?;
    assert!(status.success(), "kill -TERM must succeed");

    // Bounded, non-blocking exit wait. `try_wait` reaps on success, so the
    // guard's eventual `wait()` in Drop just sees ECHILD and is ignored.
    let deadline = Instant::now() + EXIT_TIMEOUT;
    let exit_status = loop {
        if let Some(status) = guard.0.as_mut().and_then(|c| c.try_wait().unwrap_or(None)) {
            break status;
        }
        if Instant::now() >= deadline {
            anyhow::bail!("daemon did not exit within {EXIT_TIMEOUT:?}");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert!(exit_status.success(), "daemon should exit cleanly");

    // Release the guard before reading stderr to EOF.
    drop(guard);

    let mut stderr = String::new();
    if let Some(mut r) = stderr_handle {
        r.read_to_string(&mut stderr)?;
    }
    assert!(stderr.contains("capped at 600 seconds"), "stderr: {stderr}");
    assert!(
        stderr.contains("--shutdown-timeout capped at 600"),
        "stderr: {stderr}"
    );

    Ok(())
}
