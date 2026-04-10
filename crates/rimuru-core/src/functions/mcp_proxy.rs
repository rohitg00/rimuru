use std::sync::Arc;

use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};
use tokio::sync::RwLock;

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::mcp::proxy::McpProxy;
use crate::mcp::types::ProxyServerConfig;
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    register_connect(iii, kv, proxy.clone());
    register_tools_list(iii, kv, proxy.clone());
    register_tools_call(iii, kv, proxy.clone());
    register_search_tools(iii, kv, proxy.clone());
    register_stats(iii, kv, proxy.clone());
    register_disconnect(iii, kv, proxy);
}

fn register_connect(iii: &III, kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.connect".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            let proxy = proxy.clone();
            async move {
                let input = extract_input(input);
                let name = require_str(&input, "name")?;
                let command = require_str(&input, "command")?;

                let args: Vec<String> = input
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                let progressive = input
                    .get("progressive_disclosure")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let config = ProxyServerConfig {
                    name: name.clone(),
                    command,
                    args,
                    env: Default::default(),
                    progressive_disclosure: progressive,
                    tool_threshold: 10,
                };

                let proxy = proxy.read().await;
                let result = proxy.connect_server(&config).await.map_err(kv_err)?;

                if let Err(e) = kv.set("mcp_servers", &name, &config).await {
                    tracing::warn!("Failed to persist server config: {}", e);
                }

                Ok(api_response(
                    serde_json::to_value(result).unwrap_or_default(),
                ))
            }
        },
    );
}

fn register_tools_list(iii: &III, _kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.tools".to_string()),
        move |input: Value| {
            let proxy = proxy.clone();
            async move {
                let input = extract_input(input);
                let server = input.get("server").and_then(|v| v.as_str());

                let progressive = input
                    .get("progressive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let threshold = input
                    .get("threshold")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                let proxy = proxy.read().await;
                let tools = proxy.list_tools(server, progressive, threshold).await;

                let total_schema_tokens: u64 = tools.iter().map(|t| t.schema_tokens).sum();
                let schemas_included = tools.iter().any(|t| t.input_schema.is_some());

                Ok(api_response(json!({
                    "tools": tools,
                    "total": tools.len(),
                    "total_schema_tokens": total_schema_tokens,
                    "progressive_disclosure": progressive && !schemas_included,
                })))
            }
        },
    );
}

fn register_tools_call(iii: &III, kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.call".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            let proxy = proxy.clone();
            async move {
                let input = extract_input(input);
                let tool_name = require_str(&input, "tool")?;
                let arguments = input.get("arguments").cloned().unwrap_or(json!({}));

                let proxy = proxy.read().await;
                let result = proxy
                    .call_tool(&tool_name, arguments, &kv)
                    .await
                    .map_err(kv_err)?;

                Ok(api_response(json!({
                    "result": result.result,
                    "server": result.server,
                    "input_tokens": result.input_tokens,
                    "output_tokens": result.output_tokens,
                    "cache_hit": result.cache_hit,
                    "latency_ms": result.latency_ms,
                })))
            }
        },
    );
}

fn register_search_tools(iii: &III, _kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.search".to_string()),
        move |input: Value| {
            let proxy = proxy.clone();
            async move {
                let input = extract_input(input);
                let query = require_str(&input, "query")?;

                let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                let proxy = proxy.read().await;
                let results = proxy.search_tools(&query, limit).await;

                Ok(api_response(json!({
                    "tools": results,
                    "total": results.len(),
                    "query": query,
                })))
            }
        },
    );
}

fn register_stats(iii: &III, kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.stats".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            let proxy = proxy.clone();
            async move {
                let proxy = proxy.read().await;
                let stats = proxy.get_stats(&kv).await;

                let total_calls: u64 = stats.iter().map(|(_, m)| m.call_count).sum();
                let total_input: u64 = stats.iter().map(|(_, m)| m.total_input_tokens).sum();
                let total_output: u64 = stats.iter().map(|(_, m)| m.total_output_tokens).sum();
                let total_cache_hits: u64 = stats.iter().map(|(_, m)| m.cache_hits).sum();

                let tools: Vec<Value> = stats
                    .iter()
                    .map(|(name, m)| {
                        json!({
                            "tool": name,
                            "calls": m.call_count,
                            "input_tokens": m.total_input_tokens,
                            "output_tokens": m.total_output_tokens,
                            "cache_hits": m.cache_hits,
                            "cache_misses": m.cache_misses,
                            "avg_latency_ms": m.avg_latency_ms,
                            "last_called": m.last_called,
                        })
                    })
                    .collect();

                Ok(api_response(json!({
                    "tools": tools,
                    "total_calls": total_calls,
                    "total_input_tokens": total_input,
                    "total_output_tokens": total_output,
                    "total_cache_hits": total_cache_hits,
                    "cache_hit_rate": if total_calls > 0 { total_cache_hits as f64 / total_calls as f64 * 100.0 } else { 0.0 },
                })))
            }
        },
    );
}

fn register_disconnect(iii: &III, kv: &StateKV, proxy: Arc<RwLock<McpProxy>>) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.proxy.disconnect".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            let _proxy = proxy.clone();
            async move {
                let input = extract_input(input);
                let name = require_str(&input, "name")?;

                if let Err(e) = kv.delete("mcp_servers", &name).await {
                    tracing::warn!("Failed to remove server config: {}", e);
                }

                Ok(api_response(json!({
                    "disconnected": name,
                })))
            }
        },
    );
}
