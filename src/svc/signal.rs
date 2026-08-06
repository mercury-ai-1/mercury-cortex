//! Process liveness, termination, and exit-waiting, cross-platform.
//!
//! The unix implementation wraps `libc::kill`/`waitpid` directly. Windows has
//! no POSIX signals, so it is built on `sysinfo`: liveness is a process-table
//! lookup and termination is `TerminateProcess`. The orchestrator in `stop`
//! only uses the platform-neutral [`request_termination`],
//! [`force_termination`], [`is_alive`], and [`wait_for_exit`].

use std::time::Duration;

use tokio::time::sleep;

use super::Error;

#[cfg(windows)]
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Returns `true` if a process with the given PID exists.
///
/// Unix probes with signal 0 (never a real signal). Windows looks the PID up
/// in a freshly refreshed process table.
pub fn is_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    is_alive_impl(pid)
}

#[cfg(unix)]
fn is_alive_impl(pid: u32) -> bool {
    // SAFETY: signal 0 never sends a real signal; kill(2) returns 0 when the
    // process exists and is signalable, -1 with ESRCH when it does not.
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn is_alive_impl(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
        ProcessRefreshKind::nothing(),
    );
    system.process(Pid::from_u32(pid)).is_some()
}

/// Send POSIX `sig` to the process with the given PID.
///
/// Unix-only: Windows has no signal numbers. On Windows use
/// [`request_termination`] or [`force_termination`] instead.
#[cfg(unix)]
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

/// Ask `pid` to terminate gracefully.
///
/// Returns `Ok(false)` when the process is already gone. Unix sends SIGTERM;
/// Windows terminates the process (there is no graceful analogue).
pub fn request_termination(pid: u32) -> Result<bool, Error> {
    #[cfg(unix)]
    {
        send_signal_tolerant(pid, libc::SIGTERM)
    }
    #[cfg(windows)]
    {
        terminate_tolerant(pid)
    }
}

/// Force `pid` to terminate without waiting for it to cooperate.
///
/// Returns `Ok(false)` when the process is already gone. Unix sends SIGKILL;
/// Windows terminates the process (same call as [`request_termination`]).
pub fn force_termination(pid: u32) -> Result<bool, Error> {
    #[cfg(unix)]
    {
        send_signal_tolerant(pid, libc::SIGKILL)
    }
    #[cfg(windows)]
    {
        terminate_tolerant(pid)
    }
}

#[cfg(unix)]
fn send_signal_tolerant(pid: u32, sig: i32) -> Result<bool, Error> {
    match send_signal(pid, sig) {
        Ok(()) => Ok(true),
        Err(Error::Io(e)) if e.raw_os_error() == Some(libc::ESRCH) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(windows)]
fn terminate_tolerant(pid: u32) -> Result<bool, Error> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
        ProcessRefreshKind::nothing(),
    );
    match system.process(Pid::from_u32(pid)) {
        Some(process) => {
            if process.kill() {
                Ok(true)
            } else {
                Err(Error::Io(std::io::Error::other(
                    "failed to terminate process",
                )))
            }
        }
        None => Ok(false),
    }
}

/// Poll until the process exits or `timeout` elapses.
///
/// On unix this uses `waitpid(WNOHANG)` so a terminated child is both detected
/// and reaped (a zombie still answers a `kill(pid, 0)` probe, so liveness alone
/// cannot tell a dead process from a running one). Windows has no zombies, so
/// liveness polling is definitive there. Returns `Ok(true)` when the process is
/// gone, `Ok(false)` on timeout.
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
    process_is_gone_impl(pid)
}

#[cfg(unix)]
fn process_is_gone_impl(pid: u32) -> bool {
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

#[cfg(windows)]
fn process_is_gone_impl(pid: u32) -> bool {
    !is_alive(pid)
}
