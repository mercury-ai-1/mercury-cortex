use std::sync::Arc;
use std::time::Duration;

use rmcp::service::{RoleServer, RxJsonRpcMessage, TxJsonRpcMessage};
use rmcp::transport::Transport;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

/// Maximum time to wait for a single read operation on stdin.
/// Prevents the process from hanging indefinitely if the client
/// disconnects without closing the pipe.
const READ_TIMEOUT: Duration = Duration::from_mins(5);

#[derive(Clone)]
enum Framing {
    ContentLength,
    Ndjson,
}

#[must_use]
pub fn mcp_stdio() -> McpStdioTransport {
    McpStdioTransport::new()
}

pub struct McpStdioTransport {
    reader: Arc<Mutex<BufReader<tokio::io::Stdin>>>,
    writer: Arc<Mutex<tokio::io::Stdout>>,
    framing: Arc<Mutex<Option<Framing>>>,
}

impl McpStdioTransport {
    #[must_use]
    pub fn new() -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(tokio::io::stdin()))),
            writer: Arc::new(Mutex::new(tokio::io::stdout())),
            framing: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for McpStdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport<RoleServer> for McpStdioTransport {
    type Error = std::io::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send + 'static {
        let writer = self.writer.clone();
        let framing = self.framing.clone();
        async move {
            let json = serde_json::to_string(&item)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let bytes = json.as_bytes();
            let detected = framing.lock().await.clone();
            let mut writer = writer.lock().await;
            if let Some(Framing::ContentLength) = detected {
                let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
                writer.write_all(header.as_bytes()).await?;
                writer.write_all(bytes).await?;
            } else {
                writer.write_all(bytes).await?;
                writer.write_all(b"\n").await?;
            }
            writer.flush().await
        }
    }

    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleServer>> {
        let mut reader = self.reader.lock().await;
        let framing = self.framing.clone();
        loop {
            let mut line = String::new();
            let n = tokio::time::timeout(READ_TIMEOUT, reader.read_line(&mut line))
                .await
                .ok()?
                .ok()?;
            if n == 0 {
                return None;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(len_str) = trimmed
                .to_lowercase()
                .strip_prefix("content-length:")
                .map(str::trim)
            {
                {
                    let mut f = framing.lock().await;
                    if f.is_none() {
                        *f = Some(Framing::ContentLength);
                    }
                }
                let len: usize = len_str.parse().ok()?;
                let mut sep = String::new();
                tokio::time::timeout(READ_TIMEOUT, reader.read_line(&mut sep))
                    .await
                    .ok()?
                    .ok()?;
                let mut body = vec![0u8; len];
                tokio::time::timeout(READ_TIMEOUT, reader.read_exact(&mut body))
                    .await
                    .ok()?
                    .ok()?;
                return serde_json::from_slice(&body).ok();
            }

            {
                let mut f = framing.lock().await;
                if f.is_none() {
                    *f = Some(Framing::Ndjson);
                }
            }
            return serde_json::from_str(trimmed).ok();
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
