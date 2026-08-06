//! IPC (Inter-Process Communication) client/server.
//!
//! The transport is platform-abstracted: a Unix domain socket on Unix and a
//! TCP loopback socket on Windows. Used as an optimization so that CLI
//! commands (`profile`, `project`, `clear-file-data`) can reach the daemon's
//! runtime without opening a direct database connection. The IPC server is
//! started alongside every `Runtime::new()` invocation.

pub(crate) mod client;
pub(crate) mod engine;
pub(crate) mod files;
pub(crate) mod graph;
pub(crate) mod net;
pub(crate) mod profile;
pub(crate) mod project;
pub(crate) mod protocol;
pub(crate) mod router;
pub(crate) mod server;
pub(crate) mod transport;

/// Resolve the platform IPC endpoint for a configured socket path, as a
/// display string.
///
/// On Unix this is the socket path itself; on Windows it is the derived
/// TCP loopback address (`127.0.0.1:<port>`) that the daemon binds. Exposed
/// so tools and tests can reach the daemon without duplicating the
/// platform-specific derivation.
pub fn endpoint_for(socket_path: &std::path::Path) -> String {
    net::Endpoint::from_socket_path(socket_path).display()
}
