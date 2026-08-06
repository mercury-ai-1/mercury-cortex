#![cfg(unix)]

use std::process::Command;
use std::time::Duration;

use mercury_cortex::svc::{is_alive, send_signal, wait_for_exit};

#[test]
fn is_alive_returns_true_for_self() {
    assert!(is_alive(std::process::id()));
}

#[test]
fn is_alive_returns_false_for_dead_pid() {
    assert!(!is_alive(999_999_999));
}

#[test]
fn send_signal_zero_to_self_succeeds() {
    // signal 0 is a liveness probe, not a real signal
    send_signal(std::process::id(), 0).unwrap();
}

#[test]
fn send_signal_to_dead_pid_errors() {
    assert!(send_signal(999_999_999, 15).is_err());
}

#[tokio::test]
async fn wait_for_exit_returns_true_for_already_dead_pid() {
    assert!(
        wait_for_exit(999_999_999, Duration::from_secs(1))
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn wait_for_exit_returns_false_when_still_alive() {
    let mut child = Command::new("sleep").arg("5").spawn().expect("spawn sleep");
    let alive = wait_for_exit(child.id(), Duration::from_millis(200))
        .await
        .unwrap();
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        !alive,
        "sleep is still running, should not be reported exited"
    );
}

#[tokio::test]
async fn wait_for_exit_detects_kill() {
    let mut child = Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    child.kill().unwrap();
    let exited = wait_for_exit(pid, Duration::from_secs(5)).await.unwrap();
    let _ = child.wait();
    assert!(exited, "killed sleep should be reported exited");
}
