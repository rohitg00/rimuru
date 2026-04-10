use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot};
use tracing::debug;

use super::types::*;
use crate::error::RimuruError;

type Result<T> = std::result::Result<T, RimuruError>;

pub struct McpClient {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
    next_id: AtomicU64,
    server_info: Option<McpInitializeResult>,
    _child: Child,
}

impl McpClient {
    pub async fn connect(config: &ProxyServerConfig) -> Result<Self> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| RimuruError::Bridge(format!("Failed to spawn MCP server: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| RimuruError::Bridge("No stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RimuruError::Bridge("No stdout".to_string()))?;

        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                if line.starts_with("Content-Length:") || line.starts_with("content-length:") {
                    continue;
                }

                match serde_json::from_str::<JsonRpcResponse>(&line) {
                    Ok(resp) => {
                        if let Some(id) = resp.id {
                            let mut map = pending_clone.lock().await;
                            if let Some(tx) = map.remove(&id) {
                                let _ = tx.send(resp);
                            }
                        }
                    }
                    Err(e) => {
                        debug!(
                            "Non-JSON line from MCP server: {} ({})",
                            &line[..line.len().min(100)],
                            e
                        );
                    }
                }
            }
        });

        let mut client = Self {
            stdin: Arc::new(Mutex::new(stdin)),
            pending,
            next_id: AtomicU64::new(1),
            server_info: None,
            _child: child,
        };

        client.initialize().await?;
        Ok(client)
    }

    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let msg = serde_json::to_string(&request)
            .map_err(|e| RimuruError::Bridge(format!("Serialize error: {}", e)))?;

        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(format!("{}\n", msg).as_bytes())
            .await
            .map_err(|e| RimuruError::Bridge(format!("Write error: {}", e)))?;
        stdin
            .flush()
            .await
            .map_err(|e| RimuruError::Bridge(format!("Flush error: {}", e)))?;
        drop(stdin);

        let resp = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| {
                RimuruError::Bridge(format!("Timeout waiting for response to {}", method))
            })?
            .map_err(|_| RimuruError::Bridge("Channel closed".to_string()))?;

        if let Some(err) = resp.error {
            return Err(RimuruError::Bridge(format!(
                "MCP error {}: {}",
                err.code, err.message
            )));
        }

        Ok(resp.result.unwrap_or(json!(null)))
    }

    async fn initialize(&mut self) -> Result<()> {
        let result = self
            .send_request(
                "initialize",
                Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "rimuru-mcp-proxy",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })),
            )
            .await?;

        let init_result: McpInitializeResult = serde_json::from_value(result)
            .map_err(|e| RimuruError::Bridge(format!("Invalid initialize response: {}", e)))?;

        self.server_info = Some(init_result);

        self.send_request("notifications/initialized", Some(json!({})))
            .await
            .ok();

        Ok(())
    }

    pub async fn tools_list(&self) -> Result<Vec<McpTool>> {
        let result = self.send_request("tools/list", Some(json!({}))).await?;

        let list_result: McpToolsListResult = serde_json::from_value(result)
            .map_err(|e| RimuruError::Bridge(format!("Invalid tools/list response: {}", e)))?;

        Ok(list_result.tools)
    }

    pub async fn tools_call(&self, tool_name: &str, arguments: Value) -> Result<McpToolCallResult> {
        let result = self
            .send_request(
                "tools/call",
                Some(json!({
                    "name": tool_name,
                    "arguments": arguments
                })),
            )
            .await?;

        let call_result: McpToolCallResult = serde_json::from_value(result)
            .map_err(|e| RimuruError::Bridge(format!("Invalid tools/call response: {}", e)))?;

        Ok(call_result)
    }

    pub fn server_info(&self) -> Option<&McpInitializeResult> {
        self.server_info.as_ref()
    }

    pub fn estimate_tokens(value: &Value) -> u64 {
        let s = value.to_string();
        (s.len() as u64) / 4
    }
}
