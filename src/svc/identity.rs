//! Process identity: how to recognize a Mercury Cortex service process.

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::Error;

/// A service's identity: how to recognize its processes in the process table.
///
/// Generic and data-driven; a service is *described*, never coded for, so
/// `mcp`, `daemon`, `api`, … all reuse this unchanged.
#[derive(Debug, Clone, Copy)]
pub struct ServiceIdentity<'a> {
    /// Short service name used for the PID file, e.g. `"mcp"`.
    pub name: &'a str,
    /// Substring that the process command line must contain, e.g.
    /// `"mercury-cortex mcp serve"`.
    pub command_pattern: &'a str,
}

/// The command line of `process` as a single joined string: the executable
/// path plus its arguments, each separated by a space.
///
/// This mirrors `ps -p <pid> -o command=` so the matcher has one string to
/// search, but works on Windows too (where `ps` does not exist). Non-UTF-8
/// argument bytes are replaced by the empty string; the executable path is
/// included because on macOS `cmd()` omits argv[0].
pub(crate) fn command_line(process: &Process) -> String {
    let mut parts = Vec::new();
    if let Some(exe) = process.exe() {
        parts.push(exe.as_os_str().to_owned());
    }
    parts.extend(process.cmd().iter().cloned());
    parts
        .into_iter()
        .filter_map(|s| s.into_string().ok())
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns `Ok(true)` when `pid` is a live process whose command line
/// contains `ident.command_pattern`. A missing process is `Ok(false)`, not
/// an error, so stale-PID handling stays uniform.
pub fn verify_process(pid: u32, ident: &ServiceIdentity<'_>) -> Result<bool, Error> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        false,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::OnlyIfNotSet)
            .with_exe(UpdateKind::OnlyIfNotSet),
    );

    match system.process(Pid::from_u32(pid)) {
        Some(process) => Ok(command_line(process).contains(ident.command_pattern)),
        None => Ok(false),
    }
}
