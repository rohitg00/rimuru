//! End-to-end integration test for the MCP proxy against a real
//! process that speaks Content-Length framed JSON-RPC over stdio.
//!
//! The mock server lives at `examples/mock_mcp_server.rs`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

use rimuru_core::mcp::proxy::McpProxy;
use rimuru_core::mcp::types::ProxyServerConfig;

static BUILD_MOCK: Once = Once::new();

fn mock_server_path() -> PathBuf {
    BUILD_MOCK.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .args([
                "build",
                "--quiet",
                "--example",
                "mock_mcp_server",
                "-p",
                "rimuru-core",
            ])
            .status()
            .expect("cargo build --example mock_mcp_server");
        assert!(status.success(), "mock_mcp_server build failed");
    });

    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // walk up to workspace root
    p.pop(); // crates/rimuru-core -> crates
    p.pop(); // crates -> workspace root
    p.push("target");
    p.push("debug");
    p.push("examples");
    let name = if cfg!(windows) {
        "mock_mcp_server.exe"
    } else {
        "mock_mcp_server"
    };
    p.push(name);
    p
}

fn config(name: &str, tools: usize, progressive: bool) -> ProxyServerConfig {
    let mut env = std::collections::HashMap::new();
    env.insert("MOCK_MCP_TOOLS".to_string(), tools.to_string());
    ProxyServerConfig {
        name: name.to_string(),
        command: mock_server_path().to_string_lossy().to_string(),
        args: vec![],
        env,
        progressive_disclosure: progressive,
        tool_threshold: 5,
    }
}

#[tokio::test]
async fn proxy_connects_and_lists_tools() {
    let proxy = McpProxy::new();
    let cfg = config("mock1", 3, true);
    let result = proxy.connect_server(&cfg).await.expect("connect");
    assert_eq!(result.server_name, "mock1");
    assert_eq!(result.tool_count, 3);

    let tools = proxy.list_tools(Some("mock1"), true, 10).await;
    assert_eq!(tools.len(), 3);
}

#[tokio::test]
async fn proxy_progressive_disclosure_hides_schemas() {
    let proxy = McpProxy::new();
    let cfg = config("mock2", 20, true);
    proxy.connect_server(&cfg).await.expect("connect");

    // 20 tools > threshold=5 → progressive should strip schemas
    let tools = proxy.list_tools(Some("mock2"), true, 5).await;
    assert_eq!(tools.len(), 20);
    assert!(
        tools.iter().all(|t| t.input_schema.is_none()),
        "expected schemas hidden"
    );

    // Explicit non-progressive → schemas present
    let tools_full = proxy.list_tools(Some("mock2"), false, 5).await;
    assert!(tools_full.iter().all(|t| t.input_schema.is_some()));
}

#[tokio::test]
async fn proxy_search_is_case_insensitive() {
    let proxy = McpProxy::new();
    let cfg = config("mock3", 5, true);
    proxy.connect_server(&cfg).await.expect("connect");

    let results = proxy.search_tools("TOOL_1", 10).await;
    assert!(!results.is_empty(), "expected matches for TOOL_1");
}
