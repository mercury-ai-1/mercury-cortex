use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

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

/// Spawn the real daemon binary against a hermetic `$HOME` and wait for its
/// runtime socket to appear. Returns the child and the socket path.
async fn spawn_daemon(home: &Path) -> (std::process::Child, std::path::PathBuf) {
    let bin = env!("CARGO_BIN_EXE_mercury-cortex");
    let child = Command::new(bin)
        .args(["daemon"])
        .env("HOME", home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mercury-cortex daemon");

    let socket = home.join(".mercury/cortex/runtime.sock");
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    while !socket.exists() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(socket.exists(), "daemon socket did not appear");
    (child, socket)
}

#[tokio::test]
async fn server_rejects_unsupported_protocol_version() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();
    unsafe { std::env::set_var("HOME", home) };

    let (child, socket) = spawn_daemon(home).await;
    let _child = ChildGuard(Some(child));

    let result = async {
        let mut stream = UnixStream::connect(&socket).await?;
        let req = TestIpcRequest {
            version: 999,
            id: "v-test".into(),
            method: "runtime/ping".into(),
            params: serde_json::Value::Null,
        };
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');
        stream.write_all(line.as_bytes()).await?;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        let parsed: TestIpcFailure =
            serde_json::from_slice(&buf[..n]).map_err(anyhow::Error::from)?;
        Ok::<_, anyhow::Error>(parsed)
    }
    .await;

    let parsed = result?;
    assert_eq!(parsed.error.code, "INVALID_VERSION");
    assert!(parsed.error.message.contains("999"));
    Ok(())
}

#[tokio::test]
async fn server_accepts_current_protocol_version() -> anyhow::Result<()> {
    let tmp = tempfile::TempDir::new()?;
    let home = tmp.path();
    unsafe { std::env::set_var("HOME", home) };

    let (child, socket) = spawn_daemon(home).await;
    let _child = ChildGuard(Some(child));

    let result = async {
        let mut stream = UnixStream::connect(&socket).await?;
        let req = TestIpcRequest {
            version: 1,
            id: "v-ok".into(),
            method: "runtime/ping".into(),
            params: serde_json::Value::Null,
        };
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');
        stream.write_all(line.as_bytes()).await?;
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        let parsed: TestIpcSuccess =
            serde_json::from_slice(&buf[..n]).map_err(anyhow::Error::from)?;
        Ok::<_, anyhow::Error>(parsed)
    }
    .await;

    let parsed = result?;
    assert_eq!(parsed.version, 1);
    assert_eq!(parsed.id, "v-ok");
    Ok(())
}
