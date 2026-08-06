//! Service process management for long-lived Mercury Cortex processes.
//!
//! Generic and service-agnostic: any service (`mcp`, `daemon`, `api`, …) is
//! described by a [`ServiceIdentity`](identity::ServiceIdentity) (a name and a
//! command-line pattern) plus a PID file under the shared data directory. The
//! module never references any concrete service.

use std::path::PathBuf;

mod identity;
mod pidfile;
mod signal;
mod stop;

pub use identity::{ServiceIdentity, verify_process};
pub use pidfile::{PidFile, PidFileGuard};
#[cfg(unix)]
pub use signal::send_signal;
pub use signal::{is_alive, wait_for_exit};
pub use stop::{StopOutcome, stop};

/// Errors produced by the `svc` module.
#[derive(Debug)]
pub enum Error {
    /// Underlying OS/filesystem error.
    Io(std::io::Error),
    /// A PID value that could not be parsed or is out of range.
    InvalidPid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::InvalidPid(s) => write!(f, "invalid pid: {s}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::InvalidPid(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// The resolved path to a PID file for a service, for error messages.
pub fn pidfile_path(data_dir: &std::path::Path, service_name: &str) -> PathBuf {
    data_dir.join(format!("{service_name}.pid"))
}
