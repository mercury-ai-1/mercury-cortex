//! Thin `libc` wrappers for sending signals and probing process liveness.

use std::time::Duration;

use tokio::time::sleep;

use super::Error;

/// Returns `true` if a process with the given PID exists (signal 0 probe).
pub fn is_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // SAFETY: signal 0 never sends a real signal; kill(2) returns 0 when the
    // process exists and is signalable, -1 with ESRCH when it does not.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Send `sig` to the process with the given PID.
pub fn send_signal(pid: u32, sig: i32) -> Result<(), Error> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(Error::InvalidPid(pid.to_string()));
    }
    // SAFETY: kill(2) takes a pid and a signal number; the syscall returns
    // 0 on success, -1 on error (ESRCH, EPERM, ...).
    let ret = unsafe { libc::kill(pid as i32, sig) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::Io(std::io::Error::last_os_error()))
    }
}

/// Poll until the process exits or `timeout` elapses.
///
/// Uses `waitpid(WNOHANG)` so a terminated child is both detected and reaped
/// (a zombie still answers a `kill(pid, 0)` probe, so liveness alone cannot
/// tell a dead process from a running one). Returns `Ok(true)` when the
/// process is gone, `Ok(false)` on timeout.
///
/// # Limitation: foreign processes
///
/// For processes this process does not own (the production case, e.g. stopping
/// a separate `mcp serve`), `waitpid` returns `ECHILD` and detection falls back
/// to `kill(pid, 0)` probing, so a foreign process that exited but whose own
/// parent has not yet reaped it (a foreign zombie) is still reported as alive.
/// The SIGKILL escalation in the stop orchestrator mitigates this.
pub async fn wait_for_exit(pid: u32, timeout: Duration) -> Result<bool, Error> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if process_is_gone(pid) {
            return Ok(true);
        }
        if tokio::time::Instant::now() >= deadline {
            return Ok(false);
        }
        sleep(Duration::from_millis(100)).await;
    }
}

/// Returns `true` when the process has fully exited (including reaping it if
/// it is a zombie child of ours).
fn process_is_gone(pid: u32) -> bool {
    // SAFETY: waitpid(WNOHANG) is a non-blocking probe. It returns `pid` when
    // the child has terminated (reaping it), 0 while it still runs, and -1
    // with ECHILD when `pid` is not our child (never was, or already reaped).
    // Because this reaps the child, do not also call `Child::wait()` or
    // `Child::try_wait()` for the same pid afterwards, or it will return ECHILD.
    let ret = unsafe { libc::waitpid(pid as i32, std::ptr::null_mut(), libc::WNOHANG) };
    if ret == pid as i32 {
        return true;
    }
    if ret == 0 {
        return false;
    }
    !is_alive(pid)
}
