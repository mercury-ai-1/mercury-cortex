//! PID file read/write and an RAII guard for the running process.

use std::path::{Path, PathBuf};

use super::{Error, pidfile_path};

/// A PID file on disk at `<data_dir>/<service_name>.pid`.
#[derive(Debug, Clone)]
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    /// Create a handle for `<data_dir>/<service_name>.pid` (does not touch disk).
    pub fn new(data_dir: &Path, service_name: &str) -> Self {
        Self {
            path: pidfile_path(data_dir, service_name),
        }
    }

    /// Write `pid` to the file, creating the parent directory if needed.
    pub fn write(&self, pid: u32) -> Result<(), Error> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, pid.to_string())?;
        Ok(())
    }

    /// Read the PID back. `Ok(None)` when the file is missing or unparseable.
    pub fn read(&self) -> Result<Option<u32>, Error> {
        match std::fs::read_to_string(&self.path) {
            Ok(contents) => Ok(contents.trim().parse::<u32>().ok()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Remove the file. Removing a missing file is not an error.
    pub fn remove(&self) -> Result<(), Error> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// The on-disk path of this PID file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// RAII guard: writes the current process PID on construction, removes the
/// file on `Drop`. `mcp serve` holds one for its whole lifetime so the PID
/// file is cleaned up on every exit path (normal stdio close, signal, panic).
#[must_use]
pub struct PidFileGuard {
    file: PidFile,
}

impl PidFileGuard {
    /// Write the current process's PID file and return the guard.
    pub fn acquire(data_dir: &Path, service_name: &str) -> Result<Self, Error> {
        let file = PidFile::new(data_dir, service_name);
        file.write(std::process::id())?;
        Ok(Self { file })
    }
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        let _ = self.file.remove();
    }
}
