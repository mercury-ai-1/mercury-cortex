//! Process identity: how to recognize a Mercury Cortex service process.

use super::Error;

/// A service's identity: how to recognize its processes in the process table.
///
/// Generic and data-driven — a service is *described*, never coded for, so
/// `mcp`, `daemon`, `api`, … all reuse this unchanged.
#[derive(Debug, Clone, Copy)]
pub struct ServiceIdentity<'a> {
    /// Short service name used for the PID file, e.g. `"mcp"`.
    pub name: &'a str,
    /// Substring that the process command line must contain, e.g.
    /// `"mercury-cortex mcp serve"`.
    pub command_pattern: &'a str,
}

/// Returns `Ok(true)` when `pid` is a live process whose command line
/// contains `ident.command_pattern`. A missing process is `Ok(false)`, not
/// an error, so stale-PID handling stays uniform.
pub fn verify_process(pid: u32, ident: &ServiceIdentity<'_>) -> Result<bool, Error> {
    let output = std::process::Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("command=")
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    let command = String::from_utf8_lossy(&output.stdout);
    Ok(command.contains(ident.command_pattern))
}
