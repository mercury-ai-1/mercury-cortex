use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Local stream abstraction so the test connects over whichever transport the
/// daemon uses (Unix socket on Unix, TCP loopback on Windows).
trait TestStream: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> TestStream for T {}
type BoxedStream = Box<dyn TestStream>;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Ensure a spawned daemon is always killed and reaped, even on panic.
struct ChildGuard(Option<std::process::Child>);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestIpcRequest {
    version: u32,
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestIpcSuccess {
    version: u32,
    id: String,
    result: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestIpcFailure {
    version: u32,
    id: String,
    error: TestIpcError,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestIpcError {
    code: String,
    message: String,
}

/// Connect to the daemon's platform IPC endpoint, retrying until the daemon
/// is ready. The endpoint string is either a socket path (Unix) or a
/// `host:port` address (Windows), resolved via [`mercury_cortex::ipc::endpoint_for`].
async fn connect_until_ready(endpoint: &str) -> BoxedStream {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        if let Ok(stream) = connect(endpoint).await {
            return stream;
        }
        assert!(
            Instant::now() < deadline,
            "daemon IPC endpoint did not become reachable: {endpoint}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(unix)]
async fn connect(endpoint: &str) -> anyhow::Result<BoxedStream> {
    use tokio::net::UnixStream;
    Ok(Box::new(UnixStream::connect(endpoint).await?))
}

#[cfg(windows)]
async fn connect(endpoint: &str) -> anyhow::Result<BoxedStream> {
    use tokio::net::TcpStream;
    let addr: std::net::SocketAddr = endpoint.parse()?;
    Ok(Box::new(TcpStream::connect(addr).await?))
}

/// Spawn the real daemon binary against a hermetic home and wait until its
/// IPC endpoint accepts connections. Returns the child and the endpoint
/// string the daemon listens on.
async fn spawn_daemon(home: &std::path::Path) -> (std::process::Child, String) {
    let bin = env!("CARGO_BIN_EXE_mercury-cortex");
    let mut cmd = Command::new(bin);
    cmd.args(["daemon"])
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    cmd.env("USERPROFILE", home);
    let child = cmd.spawn().expect("spawn mercury-cortex daemon");

    let socket = home.join(".mercury/cortex/runtime.sock");
    let endpoint = mercury_cortex::ipc::endpoint_for(&socket);
    let _stream = connect_until_ready(&endpoint).await;
    (child, endpoint)
}

#[tokio::test]
async fn server_rejects_unsupported_protocol_version() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();
    unsafe { std::env::set_var("HOME", home) };
    #[cfg(windows)]
    unsafe {
        std::env::set_var("USERPROFILE", home)
    };

    let (child, endpoint) = spawn_daemon(home).await;
    let _child = ChildGuard(Some(child));

    let mut stream = connect_until_ready(&endpoint).await;
    let req = TestIpcRequest {
        version: 999,
        id: "v-test".into(),
        method: "runtime/ping".into(),
        params: serde_json::Value::Null,
    };
    let mut line = serde_json::to_string(&req)?;
    line.push('\n');
    stream.write_all(line.as_bytes()).await?;
    stream.flush().await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let parsed: TestIpcFailure = serde_json::from_slice(&buf[..n])?;

    assert_eq!(parsed.error.code, "INVALID_VERSION");
    assert!(parsed.error.message.contains("999"));
    Ok(())
}

#[tokio::test]
async fn server_accepts_current_protocol_version() -> anyhow::Result<()> {
    let home = tempfile::TempDir::new()?.path().to_path_buf();
    unsafe { std::env::set_var("HOME", &home) };
    #[cfg(windows)]
    unsafe {
        std::env::set_var("USERPROFILE", &home)
    };

    let (child, endpoint) = spawn_daemon(&home).await;
    let _child = ChildGuard(Some(child));

    let mut stream = connect_until_ready(&endpoint).await;
    let req = TestIpcRequest {
        version: 1,
        id: "v-ok".into(),
        method: "runtime/ping".into(),
        params: serde_json::Value::Null,
    };
    let mut line = serde_json::to_string(&req)?;
    line.push('\n');
    stream.write_all(line.as_bytes()).await?;
    stream.flush().await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let parsed: TestIpcSuccess = serde_json::from_slice(&buf[..n])?;

    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.id, "v-ok");
    Ok(())
}
