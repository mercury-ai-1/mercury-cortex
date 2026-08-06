//! Platform-abstracted IPC transport.
//!
//! On Unix, IPC uses a Unix domain socket at `RuntimeConfig::socket_path`.
//! Windows has no Unix domain sockets, so the daemon binds a TCP listener on
//! the loopback interface instead. The port is derived deterministically from
//! the socket path, so the daemon and every client resolve the same address
//! without any sidecar file or coordination.
//!
//! Everything below the request/response framing in this crate talks to
//! [`IpcStream`], never to `UnixStream`/`TcpStream` directly.

use std::io;
use std::path::Path;

use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(windows)]
use std::net::SocketAddr;
#[cfg(windows)]
use tokio::net::TcpListener;
#[cfg(windows)]
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio::net::UnixStream;

/// A socket capable of bidirectional IPC I/O, abstracted over the platform
/// socket type so the rest of the crate never names `UnixStream`/`TcpStream`.
pub(crate) trait IpcSocket: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> IpcSocket for T {}

/// A boxed bidirectional byte stream used for IPC.
pub(crate) type IpcStream = Box<dyn IpcSocket>;

/// Where the IPC server listens.
#[derive(Debug, Clone)]
pub(crate) enum Endpoint {
    /// A Unix domain socket at the given path.
    #[cfg(unix)]
    Unix(std::path::PathBuf),
    /// A TCP loopback address (Windows).
    #[cfg(windows)]
    Tcp(SocketAddr),
}

impl Endpoint {
    /// Resolve the IPC endpoint for the configured socket path.
    pub(crate) fn from_socket_path(path: &Path) -> Self {
        #[cfg(unix)]
        {
            Endpoint::Unix(path.to_path_buf())
        }
        #[cfg(windows)]
        {
            Endpoint::Tcp(tcp_addr(path))
        }
    }

    /// Fast probe for whether a daemon may be listening here.
    ///
    /// On Unix this is the socket file existing. Windows has no socket file,
    /// so the probe is always affirmative and the caller relies on a real
    /// connect (which fails fast with `ECONNREFUSED` on loopback) instead.
    pub(crate) fn probe(&self) -> bool {
        match self {
            #[cfg(unix)]
            Endpoint::Unix(path) => path.exists(),
            #[cfg(windows)]
            Endpoint::Tcp(_) => true,
        }
    }

    /// Human-readable endpoint for logs and status.
    pub(crate) fn display(&self) -> String {
        match self {
            #[cfg(unix)]
            Endpoint::Unix(path) => path.display().to_string(),
            #[cfg(windows)]
            Endpoint::Tcp(addr) => addr.to_string(),
        }
    }
}

/// A listening IPC socket, abstracted over the platform socket type.
pub(crate) enum IpcListener {
    #[cfg(unix)]
    Unix(UnixListener),
    #[cfg(windows)]
    Tcp(TcpListener),
}

/// Bind a listener on the endpoint.
///
/// On Unix any stale socket left by a previous unclean shutdown is removed
/// first so the bind cannot collide with a dead socket file.
pub(crate) async fn bind(endpoint: &Endpoint) -> io::Result<IpcListener> {
    match endpoint {
        #[cfg(unix)]
        Endpoint::Unix(path) => {
            let _ = tokio::fs::remove_file(path).await;
            UnixListener::bind(path).map(IpcListener::Unix)
        }
        #[cfg(windows)]
        Endpoint::Tcp(addr) => TcpListener::bind(addr).await.map(IpcListener::Tcp),
    }
}

impl IpcListener {
    /// Accept the next connection, boxed so callers stay platform-agnostic.
    pub(crate) async fn accept(&self) -> io::Result<IpcStream> {
        let (stream, _addr) = match self {
            #[cfg(unix)]
            IpcListener::Unix(l) => l.accept().await?,
            #[cfg(windows)]
            IpcListener::Tcp(l) => l.accept().await?,
        };
        Ok(Box::new(stream))
    }
}

/// Connect to the endpoint, returning a boxed stream.
pub(crate) async fn connect(endpoint: &Endpoint) -> io::Result<IpcStream> {
    let stream = match endpoint {
        #[cfg(unix)]
        Endpoint::Unix(path) => UnixStream::connect(path).await?,
        #[cfg(windows)]
        Endpoint::Tcp(addr) => TcpStream::connect(addr).await?,
    };
    Ok(Box::new(stream))
}

/// Derive a stable loopback port from the socket path (Windows only).
///
/// FNV-1a over the socket path, mapped into the unprivileged port range. Both
/// the daemon and every client hash the same `socket_path`, so they agree on
/// the address without writing anything to disk.
#[cfg(windows)]
fn tcp_addr(path: &Path) -> SocketAddr {
    let lossy = path.to_string_lossy();
    let bytes = lossy.as_bytes();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let port = 1024 + (hash % (u16::MAX as u64 - 1023)) as u16;
    SocketAddr::from(([127, 0, 0, 1], port))
}
