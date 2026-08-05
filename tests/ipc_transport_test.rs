use std::path::PathBuf;
use std::time::Duration;

use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::net::UnixStream;
use tokio::time::timeout;

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

fn test_socket_path(tmp: &TempDir) -> PathBuf {
    tmp.path().join("transport-test.sock")
}

/// Start a simple echo server that accepts one connection, reads one request,
/// and responds with a null result.
async fn start_echo_server(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
    let socket = socket_path.to_path_buf();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();

        let resp = TestIpcSuccess {
            version: 1,
            id: req.id,
            result: serde_json::Value::Null,
        };
        let mut resp_json = serde_json::to_string(&resp).unwrap();
        resp_json.push('\n');
        reader
            .get_mut()
            .write_all(resp_json.as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
    })
}

async fn send_request(stream: &mut UnixStream, method: &str) -> Result<String, String> {
    let req = TestIpcRequest {
        version: 1,
        id: uuid::Uuid::new_v4().to_string(),
        method: method.to_string(),
        params: serde_json::Value::Null,
    };
    let req_json = serde_json::to_string(&req).map_err(|e| e.to_string())?;
    stream
        .write_all(req_json.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stream.write_all(b"\n").await.map_err(|e| e.to_string())?;
    stream.flush().await.map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; 4096];
    let n = timeout(Duration::from_secs(5), stream.read(&mut buf))
        .await
        .map_err(|_| "read timed out".to_string())?
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

#[tokio::test]
async fn test_request_response_through_socket() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);
    let _server = start_echo_server(&socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("should connect");
    let response = send_request(&mut stream, "runtime/ping")
        .await
        .expect("request should succeed");
    let parsed: TestIpcSuccess = serde_json::from_str(response.trim()).expect("valid response");
    assert_eq!(parsed.result, serde_json::Value::Null);
}

#[tokio::test]
async fn test_multiple_requests_on_same_connection() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let _server = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        for _ in 0..2 {
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();
            let resp = TestIpcSuccess {
                version: 1,
                id: req.id,
                result: serde_json::Value::Null,
            };
            let mut resp_json = serde_json::to_string(&resp).unwrap();
            resp_json.push('\n');
            reader
                .get_mut()
                .write_all(resp_json.as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();
        }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let r1 = send_request(&mut stream, "runtime/ping")
        .await
        .expect("first call");
    let p1: TestIpcSuccess = serde_json::from_str(r1.trim()).unwrap();
    assert_eq!(p1.result, serde_json::Value::Null);

    let r2 = send_request(&mut stream, "runtime/ping")
        .await
        .expect("second call");
    let p2: TestIpcSuccess = serde_json::from_str(r2.trim()).unwrap();
    assert_eq!(p2.result, serde_json::Value::Null);
}

#[tokio::test]
async fn test_multiple_concurrent_connections() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let _server = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        for _ in 0..12 {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();
                let resp = TestIpcSuccess {
                    version: 1,
                    id: req.id,
                    result: serde_json::Value::Null,
                };
                let mut resp_json = serde_json::to_string(&resp).unwrap();
                resp_json.push('\n');
                reader
                    .get_mut()
                    .write_all(resp_json.as_bytes())
                    .await
                    .unwrap();
                reader.get_mut().flush().await.unwrap();
            });
        }
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut streams = Vec::new();
    for _ in 0..10 {
        let stream = UnixStream::connect(&socket_path)
            .await
            .expect("should connect");
        streams.push(stream);
    }
    assert_eq!(streams.len(), 10);
}

#[tokio::test]
async fn test_connection_fails_on_bad_path() {
    let result = timeout(
        Duration::from_secs(3),
        UnixStream::connect("/nonexistent/socket.sock"),
    )
    .await;
    match result {
        Ok(Err(_)) => {}
        Ok(Ok(_)) => panic!("unexpectedly connected to non-existent socket"),
        Err(_) => panic!("timeout: connection should have failed fast"),
    }
}
