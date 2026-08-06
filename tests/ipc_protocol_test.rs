use serde_json::json;

#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::time::Duration;
#[cfg(unix)]
use tempfile::TempDir;

#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(unix)]
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
    #[serde(default)]
    recovery: Option<String>,
}

#[cfg(unix)]
fn test_socket_path(tmp: &TempDir) -> PathBuf {
    tmp.path().join("test.sock")
}

#[tokio::test]
async fn test_protocol_request_round_trip() {
    let req = TestIpcRequest {
        version: 1,
        id: "req-1".into(),
        method: "runtime/ping".into(),
        params: json!({"key": "value"}),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: TestIpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.id, "req-1");
    assert_eq!(deserialized.method, "runtime/ping");
    assert_eq!(deserialized.params["key"], "value");
}

#[tokio::test]
async fn test_protocol_request_empty_params() {
    let req = TestIpcRequest {
        version: 1,
        id: "req-2".into(),
        method: "runtime/status".into(),
        params: serde_json::Value::Null,
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: TestIpcRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.params, serde_json::Value::Null);
}

#[tokio::test]
async fn test_protocol_success_round_trip() {
    let resp = TestIpcSuccess {
        version: 1,
        id: "resp-1".into(),
        result: json!({"status": "running", "uptime_ms": 1234}),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: TestIpcSuccess = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.id, "resp-1");
    assert_eq!(deserialized.result["status"], "running");
    assert_eq!(deserialized.result["uptime_ms"], 1234);
}

#[tokio::test]
async fn test_protocol_failure_round_trip() {
    let resp = TestIpcFailure {
        version: 1,
        id: "fail-1".into(),
        error: TestIpcError {
            code: "NOT_FOUND".into(),
            message: "method not found: unknown/endpoint".into(),
            recovery: None,
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: TestIpcFailure = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.version, 1);
    assert_eq!(deserialized.id, "fail-1");
    assert_eq!(deserialized.error.code, "NOT_FOUND");
    assert!(deserialized.error.message.contains("unknown/endpoint"));
}

#[tokio::test]
async fn test_protocol_failure_variant_codes() {
    for code in &[
        "NOT_FOUND",
        "DATABASE_ERROR",
        "VALIDATION_ERROR",
        "INTERNAL_ERROR",
        "RUNTIME_NOT_READY",
    ] {
        let resp = TestIpcFailure {
            version: 1,
            id: "fail".into(),
            error: TestIpcError {
                code: code.to_string(),
                message: format!("{code}: something went wrong"),
                recovery: None,
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: TestIpcFailure = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.error.code, *code);
    }
}

#[tokio::test]
async fn test_protocol_failure_with_recovery_round_trip() {
    let resp = TestIpcFailure {
        version: 1,
        id: "fail-rec".into(),
        error: TestIpcError {
            code: "INVALID_VERSION".into(),
            message: "unsupported protocol version 999".into(),
            recovery: Some("restart the CLI to match the daemon version".into()),
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: TestIpcFailure = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.error.code, "INVALID_VERSION");
    assert_eq!(
        deserialized.error.recovery.as_deref(),
        Some("restart the CLI to match the daemon version")
    );
}

#[tokio::test]
async fn test_protocol_failure_recovery_defaults_to_none() {
    let resp = TestIpcFailure {
        version: 1,
        id: "fail-none".into(),
        error: TestIpcError {
            code: "NOT_FOUND".into(),
            message: "unknown".into(),
            recovery: None,
        },
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: TestIpcFailure = serde_json::from_str(&json).unwrap();
    assert!(deserialized.error.recovery.is_none());
    // Wire shape stays additive: absent recovery must parse too.
    let without_field =
        r#"{"version":1,"id":"x","error":{"code":"NOT_FOUND","message":"unknown"}}"#;
    let parsed: TestIpcFailure = serde_json::from_str(without_field).unwrap();
    assert!(parsed.error.recovery.is_none());
}

#[cfg(unix)]
async fn run_echo_server(socket_path: &std::path::Path) -> tokio::task::JoinHandle<()> {
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
            result: json!({"pong": true}),
        };
        let resp_json = serde_json::to_string(&resp).unwrap();
        reader
            .get_mut()
            .write_all(resp_json.as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
    })
}

#[cfg(unix)]
#[tokio::test]
async fn test_request_response_over_unix_socket() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let server_handle = run_echo_server(&socket_path).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let req = TestIpcRequest {
        version: 1,
        id: "client-1".into(),
        method: "runtime/ping".into(),
        params: serde_json::Value::Null,
    };
    let req_json = serde_json::to_string(&req).unwrap();
    stream.write_all(req_json.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    let response: TestIpcSuccess = serde_json::from_str(response_str.trim()).unwrap();
    assert_eq!(response.result["pong"], true);
    assert_eq!(response.id, "client-1");

    server_handle.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_request_with_params_response() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let server_handle = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(req.method, "project/register");

        let result = json!({"project_id": "projects:abc", "status": "registered"});
        let resp = TestIpcSuccess {
            version: 1,
            id: req.id,
            result,
        };
        let resp_json = serde_json::to_string(&resp).unwrap();
        reader
            .get_mut()
            .write_all(resp_json.as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let req = TestIpcRequest {
        version: 1,
        id: "client-2".into(),
        method: "project/register".into(),
        params: json!({"root": "/tmp/test-project"}),
    };
    let req_json = serde_json::to_string(&req).unwrap();
    stream.write_all(req_json.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    let response: TestIpcSuccess = serde_json::from_str(response_str.trim()).unwrap();
    assert_eq!(response.result["project_id"], "projects:abc");

    server_handle.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_server_returns_failure() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let server_handle = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();

        let resp = TestIpcFailure {
            version: 1,
            id: req.id,
            error: TestIpcError {
                code: "NOT_FOUND".into(),
                message: format!("Unknown method: {}", req.method),
                recovery: None,
            },
        };
        let resp_json = serde_json::to_string(&resp).unwrap();
        reader
            .get_mut()
            .write_all(resp_json.as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let req = TestIpcRequest {
        version: 1,
        id: "fail-client".into(),
        method: "unknown/method".into(),
        params: serde_json::Value::Null,
    };
    let req_json = serde_json::to_string(&req).unwrap();
    stream.write_all(req_json.as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    let response: TestIpcFailure = serde_json::from_str(response_str.trim()).unwrap();
    assert_eq!(response.error.code, "NOT_FOUND");
    assert!(response.error.message.contains("unknown/method"));
    assert_eq!(response.id, "fail-client");

    server_handle.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_connection_refused() {
    let result = timeout(
        Duration::from_secs(3),
        UnixStream::connect("/tmp/mercury-nonexistent-test-socket.sock"),
    )
    .await;
    match result {
        Ok(Err(_)) => {}
        Ok(Ok(_)) => panic!("unexpectedly connected to non-existent socket"),
        Err(_) => panic!("timeout: connection should have failed fast"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn test_connection_to_closed_socket() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let listener = UnixListener::bind(&socket_path).unwrap();
    drop(listener);

    let result = UnixStream::connect(&socket_path).await;
    assert!(result.is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn test_multiple_requests_on_same_connection() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let server_handle = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);

        for i in 0..3 {
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let req: TestIpcRequest = serde_json::from_str(line.trim()).unwrap();
            assert_eq!(req.method, "runtime/ping");

            let resp = TestIpcSuccess {
                version: 1,
                id: req.id,
                result: json!({"seq": i}),
            };
            let resp_json = serde_json::to_string(&resp).unwrap();
            reader
                .get_mut()
                .write_all(resp_json.as_bytes())
                .await
                .unwrap();
            reader.get_mut().write_all(b"\n").await.unwrap();
            reader.get_mut().flush().await.unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    for i in 0..3 {
        let req = TestIpcRequest {
            version: 1,
            id: format!("multi-{i}"),
            method: "runtime/ping".into(),
            params: serde_json::Value::Null,
        };
        let req_json = serde_json::to_string(&req).unwrap();
        stream.write_all(req_json.as_bytes()).await.unwrap();
        stream.write_all(b"\n").await.unwrap();
        stream.flush().await.unwrap();

        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let response: TestIpcSuccess = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(response.result["seq"], i);
        assert_eq!(response.id, format!("multi-{i}"));
    }

    server_handle.await.unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn test_invalid_json_returns_error() {
    let tmp = TempDir::new().unwrap();
    let socket_path = test_socket_path(&tmp);

    let socket = socket_path.clone();
    let server_handle = tokio::spawn(async move {
        let listener = UnixListener::bind(&socket).unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();

        let resp = TestIpcFailure {
            version: 1,
            id: "?".into(),
            error: TestIpcError {
                code: "INVALID_PARAMS".into(),
                message: format!("failed to parse: {line}"),
                recovery: None,
            },
        };
        let resp_json = serde_json::to_string(&resp).unwrap();
        reader
            .get_mut()
            .write_all(resp_json.as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    stream.write_all(b"not valid json\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await.unwrap();
    let response_str = String::from_utf8_lossy(&buf[..n]);
    let response: TestIpcFailure = serde_json::from_str(response_str.trim()).unwrap();
    assert_eq!(response.error.code, "INVALID_PARAMS");

    server_handle.await.unwrap();
}
