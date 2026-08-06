//! Stop orchestration for service processes.

use std::path::Path;
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::Error;
use super::identity::{ServiceIdentity, command_line, verify_process};
use super::pidfile::PidFile;
use super::signal::{force_termination, is_alive, request_termination, wait_for_exit};

/// How long to wait for a graceful stop before escalating to a forced one.
pub const GRACE_PERIOD: Duration = Duration::from_secs(5);

/// What happened when `stop` ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopOutcome {
    /// PID file missing or process already gone — nothing to do.
    AlreadyStopped,
    /// PID file existed but pointed at a dead process; file removed.
    StalePidRemoved,
    /// Graceful termination requested, process exited within the grace period,
    /// PID file removed.
    Stopped,
    /// Graceful termination timed out; process was killed, PID file removed.
    ForceKilled,
    /// PID alive but did not match the service identity — refused to touch it.
    IdentityMismatch,
    /// Orchestration failed (I/O, wait, scan). Carries a message.
    Failed(String),
}

impl StopOutcome {
    /// A human-readable report of what happened.
    pub fn message(&self) -> String {
        match self {
            StopOutcome::AlreadyStopped => "No running service found (no PID file).".to_string(),
            StopOutcome::StalePidRemoved => {
                "Removed stale PID file (process was not running).".to_string()
            }
            StopOutcome::Stopped => "Service stopped (terminated).".to_string(),
            StopOutcome::ForceKilled => {
                "Service force-killed (graceful stop timed out).".to_string()
            }
            StopOutcome::IdentityMismatch => {
                "Refusing to stop: PID does not match a Mercury Cortex process.".to_string()
            }
            StopOutcome::Failed(msg) => format!("Failed to stop service: {msg}"),
        }
    }
}

/// Stop every live process matching `ident`, seeded by the service's PID file.
///
/// Never terminates a process unless its command line matches
/// `ident.command_pattern` — unrelated processes are always ignored.
pub async fn stop(ident: &ServiceIdentity<'_>, data_dir: &Path) -> Result<StopOutcome, Error> {
    let pidfile = PidFile::new(data_dir, ident.name);
    let self_pid = std::process::id();
    let pidfile_pid = pidfile.read()?;

    let mut outcome = StopOutcome::AlreadyStopped;

    if let Some(pid) = pidfile_pid {
        if !is_alive(pid) {
            pidfile.remove()?;
            outcome = StopOutcome::StalePidRemoved;
        } else if !verify_process(pid, ident)? {
            return Ok(StopOutcome::IdentityMismatch);
        } else {
            outcome = stop_process(pid, ident, &pidfile).await?;
            if outcome == StopOutcome::AlreadyStopped {
                pidfile.remove()?;
                outcome = StopOutcome::StalePidRemoved;
            }
        }
    }

    // Catch any additional matching processes (e.g. one `mcp serve` per
    // OpenCode session) even when no PID file exists. Exclude ourselves and
    // the pidfile PID, which the pidfile path already handled.
    for pid in find_matching_pids(ident)? {
        if pid == self_pid || pidfile_pid == Some(pid) {
            continue;
        }
        let this = stop_process(pid, ident, &pidfile).await?;
        if this != StopOutcome::AlreadyStopped {
            outcome = this;
        }
    }

    Ok(outcome)
}

/// Terminate `pid` (graceful request → grace period → kill) and remove the
/// PID file.
///
/// Identity is re-verified immediately before each termination so a PID
/// reused by an unrelated process between scan and signal is never touched.
async fn stop_process(
    pid: u32,
    ident: &ServiceIdentity<'_>,
    pidfile: &PidFile,
) -> Result<StopOutcome, Error> {
    if !is_alive(pid) {
        return Ok(StopOutcome::AlreadyStopped);
    }

    if !verify_process(pid, ident)? {
        return Ok(StopOutcome::IdentityMismatch);
    }

    if !request_termination(pid)? {
        let _ = pidfile.remove();
        return Ok(StopOutcome::Stopped);
    }
    if wait_for_exit(pid, GRACE_PERIOD).await? {
        let _ = pidfile.remove();
        return Ok(StopOutcome::Stopped);
    }

    if !verify_process(pid, ident)? {
        return Ok(StopOutcome::IdentityMismatch);
    }
    if !force_termination(pid)? {
        let _ = pidfile.remove();
        return Ok(StopOutcome::ForceKilled);
    }
    if wait_for_exit(pid, GRACE_PERIOD).await? {
        let _ = pidfile.remove();
        return Ok(StopOutcome::ForceKilled);
    }

    Ok(StopOutcome::Failed(format!(
        "process {pid} did not exit after termination"
    )))
}

/// List PIDs of live processes whose command line contains
/// `ident.command_pattern`, from a full process-table scan.
///
/// A command line like `... mercury-cortex daemon stop` is a stop
/// invocation, not the service itself. Excluding it prevents wrapper
/// shells (e.g. `sh -c "mercury-cortex daemon stop"`) from being
/// terminated when their argv contains the service's command pattern.
fn find_matching_pids(ident: &ServiceIdentity<'_>) -> Result<Vec<u32>, Error> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        false,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::OnlyIfNotSet)
            .with_exe(UpdateKind::OnlyIfNotSet),
    );

    let stop_suffix = format!("{} stop", ident.command_pattern);

    let mut pids = Vec::new();
    for (pid, process) in system.processes() {
        let command = command_line(process);
        if command.contains(ident.command_pattern) && !command.contains(&stop_suffix) {
            pids.push(pid.as_u32());
        }
    }
    Ok(pids)
}
