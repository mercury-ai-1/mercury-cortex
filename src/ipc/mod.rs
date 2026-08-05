//! IPC (Inter-Process Communication) client/server over Unix domain sockets.
//!
//! Used as an optimization so that CLI commands (`profile`, `project`,
//! `clear-file-data`) can reach the daemon's runtime without opening a
//! direct database connection.  The IPC server is started alongside every
//! `Runtime::new()` invocation.

pub(crate) mod client;
pub(crate) mod engine;
pub(crate) mod files;
pub(crate) mod graph;
pub(crate) mod profile;
pub(crate) mod project;
pub(crate) mod protocol;
pub(crate) mod router;
pub(crate) mod server;
pub(crate) mod transport;
