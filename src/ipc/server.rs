use std::sync::Arc;

use tokio::net::UnixListener;

use mercury_cortex_core::runtime::RuntimeContext;

use super::protocol::{
    CODE_INVALID_VERSION, IpcFailure, IpcRequest, IpcSuccess, PROTOCOL_VERSION, validate_version,
};
use super::router;

/// Start the IPC server on the configured Unix socket.
///
/// Returns a `JoinHandle` that lives for the duration of the runtime. The
/// runtime stores this handle so the server is cancelled on drop.
pub(crate) async fn start(
    ctx: Arc<RuntimeContext>,
) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    let socket_path = &ctx.config.socket_path;

    // Clean up any stale socket from a previous unclean shutdown.
    let _ = tokio::fs::remove_file(socket_path).await;

    let listener = UnixListener::bind(socket_path)?;

    tracing::info!(path = %socket_path.display(), "IPC server listening");

    let handle = tokio::spawn(async move {
        loop {
            let (stream, _addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!(error = %e, "IPC accept error");
                    continue;
                }
            };

            let ctx = ctx.clone();
            tokio::spawn(async move {
                handle_connection(stream, &ctx).await;
            });
        }
    });

    Ok(handle)
}

/// Timeout for reading a request on the IPC server.
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

async fn handle_connection(stream: tokio::net::UnixStream, ctx: &RuntimeContext) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::time::timeout;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    match timeout(READ_TIMEOUT, reader.read_line(&mut line)).await {
        Ok(Ok(0) | Err(_)) | Err(_) => return, // EOF, error, or timeout
        Ok(Ok(_)) => {}
    }

    let request: IpcRequest = match serde_json::from_str(&line) {
        Ok(req) => req,
        Err(e) => {
            let failure = IpcFailure::new("?", "INVALID_PARAMS", e.to_string());
            let mut response = serde_json::to_string(&failure).unwrap_or_default();
            response.push('\n');
            let _ = reader.get_mut().write_all(response.as_bytes()).await;
            return;
        }
    };

    if !validate_version(request.version) {
        let failure = IpcFailure::with_recovery(
            &request.id,
            CODE_INVALID_VERSION,
            format!("unsupported protocol version {}", request.version),
            "restart the CLI to match the daemon version",
        );
        let mut response = serde_json::to_string(&failure).unwrap_or_default();
        response.push('\n');
        let _ = reader.get_mut().write_all(response.as_bytes()).await;
        return;
    }

    let response = match router::dispatch(ctx, &request.method, request.params).await {
        Ok(result) => {
            let success = IpcSuccess {
                version: PROTOCOL_VERSION,
                id: request.id.clone(),
                result,
            };
            serde_json::to_string(&success).unwrap_or_default()
        }
        Err(err) => {
            let failure = IpcFailure::from((&request.id as &str, err));
            serde_json::to_string(&failure).unwrap_or_default()
        }
    };

    let _ = reader.get_mut().write_all(response.as_bytes()).await;
}
