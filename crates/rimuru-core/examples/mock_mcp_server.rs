//! Mock MCP server for integration tests.
//!
//! Speaks Content-Length framed JSON-RPC over stdio, responding to:
//!   - `initialize`
//!   - `tools/list`
//!   - `tools/call` (echoes `arguments` back as text content)
//!
//! The number of tools is controlled by the env var `MOCK_MCP_TOOLS`
//! (default `3`). Each tool is named `tool_{i}` and has a trivial schema.

use std::env;
use std::io::{self, BufRead, Write};

use serde_json::{Value, json};

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    let mut header_line = String::new();

    loop {
        header_line.clear();
        let n = reader.read_line(&mut header_line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = header_line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed
            .strip_prefix("Content-Length:")
            .or_else(|| trimmed.strip_prefix("content-length:"))
        {
            content_length = rest.trim().parse().ok();
        }
    }

    let len = match content_length {
        Some(n) => n,
        None => return Ok(None),
    };

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
}

fn write_message<W: Write>(writer: &mut W, body: &str) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()
}

fn tool_count() -> usize {
    env::var("MOCK_MCP_TOOLS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3)
}

fn handle(req: Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {"name": "mock-mcp-server", "version": "0.1.0"},
            "capabilities": {"tools": {}},
        }),
        "tools/list" => {
            let tools: Vec<_> = (0..tool_count())
                .map(|i| {
                    json!({
                        "name": format!("tool_{i}"),
                        "description": format!("mock tool {i}"),
                        "inputSchema": {
                            "type": "object",
                            "properties": {"echo": {"type": "string"}},
                        }
                    })
                })
                .collect();
            json!({"tools": tools})
        }
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let args = params.get("arguments").cloned().unwrap_or(Value::Null);
            json!({
                "content": [
                    {"type": "text", "text": format!("called {name} with {args}")}
                ],
                "isError": false,
            })
        }
        _ => {
            id.as_ref()?;
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("method not found: {method}")},
            }));
        }
    };

    id.as_ref()?;

    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    }))
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin = stdin.lock();
    let mut stdout = stdout.lock();

    while let Some(body) = read_message(&mut stdin)? {
        if body.is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(resp) = handle(req) {
            let s = resp.to_string();
            write_message(&mut stdout, &s)?;
        }
    }

    Ok(())
}
