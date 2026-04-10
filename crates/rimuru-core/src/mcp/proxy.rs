use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::client::McpClient;
use super::types::*;
use crate::error::RimuruError;
use crate::state::StateKV;

type Result<T> = std::result::Result<T, RimuruError>;

pub const KV_SCOPE_METRICS: &str = "mcp_metrics";
pub const KV_SCOPE_SERVERS: &str = "mcp_servers";

pub struct McpProxy {
    clients: Arc<RwLock<HashMap<String, McpClient>>>,
    tool_index: Arc<RwLock<HashMap<String, (String, McpTool)>>>,
    cache: Arc<RwLock<HashMap<String, (Value, std::time::Instant)>>>,
    cache_ttl: std::time::Duration,
    cache_max: usize,
}

impl Default for McpProxy {
    fn default() -> Self {
        Self::new()
    }
}

impl McpProxy {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: std::time::Duration::from_secs(300),
            cache_max: 256,
        }
    }

    pub async fn connect_server(&self, config: &ProxyServerConfig) -> Result<ConnectResult> {
        let client = McpClient::connect(config).await?;

        let server_name = config.name.clone();
        let server_info = client.server_info().cloned();

        let tools = client.tools_list().await?;
        let tool_count = tools.len();

        let mut index = self.tool_index.write().await;
        index.retain(|_, (srv, _)| srv != &server_name);
        for tool in &tools {
            let key = format!("{}::{}", server_name, tool.name);
            index.insert(key, (server_name.clone(), tool.clone()));
        }
        drop(index);

        self.cache
            .write()
            .await
            .retain(|k, _| !k.starts_with(&format!("{}::", server_name)));

        let schema_tokens: u64 = tools
            .iter()
            .map(|t| McpClient::estimate_tokens(&t.input_schema.clone().unwrap_or(json!({}))))
            .sum();

        self.clients
            .write()
            .await
            .insert(server_name.clone(), client);

        info!(
            "Connected to MCP server '{}': {} tools, ~{} schema tokens",
            server_name, tool_count, schema_tokens
        );

        Ok(ConnectResult {
            server_name,
            server_info,
            tool_count,
            schema_tokens,
        })
    }

    pub async fn list_tools(
        &self,
        server: Option<&str>,
        progressive: bool,
        threshold: usize,
    ) -> Vec<ToolListEntry> {
        let index = self.tool_index.read().await;

        let tools: Vec<_> = index
            .iter()
            .filter(|(_, (srv, _))| server.is_none() || server == Some(srv.as_str()))
            .collect();

        let use_progressive = progressive && tools.len() > threshold;

        tools
            .iter()
            .map(|(name, (srv, tool))| ToolListEntry::from_index(name, srv, tool, !use_progressive))
            .collect()
    }

    pub async fn search_tools(&self, query: &str, limit: usize) -> Vec<ToolListEntry> {
        let query_lower = query.to_lowercase();
        let index = self.tool_index.read().await;

        let mut scored: Vec<(u32, ToolListEntry)> = index
            .iter()
            .filter_map(|(name, (srv, tool))| {
                let name_lower = name.to_lowercase();
                let desc_lower = tool.description.as_deref().unwrap_or("").to_lowercase();

                let mut score: u32 = 0;
                if name_lower == query_lower {
                    score += 100;
                } else if name_lower.contains(&query_lower) {
                    score += 50;
                }
                if desc_lower.contains(&query_lower) {
                    score += 25;
                }

                for word in query_lower.split_whitespace() {
                    if name_lower.contains(word) {
                        score += 10;
                    }
                    if desc_lower.contains(word) {
                        score += 5;
                    }
                }

                if score > 0 {
                    Some((score, ToolListEntry::from_index(name, srv, tool, true)))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().take(limit).map(|(_, t)| t).collect()
    }

    pub async fn disconnect_server(&mut self, name: &str) {
        self.clients.write().await.remove(name);
        self.tool_index
            .write()
            .await
            .retain(|_, (srv, _)| srv != name);
        self.cache
            .write()
            .await
            .retain(|k, _| !k.starts_with(&format!("{}::", name)));
        info!("Disconnected MCP server '{}'", name);
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        kv: &StateKV,
    ) -> Result<ToolCallResult> {
        let start = std::time::Instant::now();

        let (server_name, tool) = {
            let index = self.tool_index.read().await;
            if let Some(entry) = index.get(tool_name) {
                entry.clone()
            } else {
                let matches: Vec<_> = index
                    .iter()
                    .filter(|(_, (_, t))| t.name == tool_name)
                    .collect();

                match matches.len() {
                    0 => {
                        return Err(RimuruError::Bridge(format!(
                            "Tool not found: {}",
                            tool_name
                        )));
                    }
                    1 => matches[0].1.clone(),
                    n => {
                        let servers: Vec<_> =
                            matches.iter().map(|(_, (srv, _))| srv.as_str()).collect();
                        return Err(RimuruError::Bridge(format!(
                            "Ambiguous tool '{}' found in {} servers: {}. Use 'server::tool' format.",
                            tool_name,
                            n,
                            servers.join(", ")
                        )));
                    }
                }
            }
        };

        let resolved_name = &tool.name;
        let cache_key = format!(
            "{}::{}::{}",
            server_name,
            resolved_name,
            sha256_short(&serde_json::to_string(&arguments).unwrap_or_default())
        );
        {
            let cache = self.cache.read().await;
            if let Some((cached_result, cached_at)) = cache.get(&cache_key)
                && cached_at.elapsed() < self.cache_ttl
            {
                let output_tokens = McpClient::estimate_tokens(cached_result);
                self.record_metrics(
                    kv,
                    resolved_name,
                    &server_name,
                    0,
                    output_tokens,
                    true,
                    start.elapsed().as_millis() as f64,
                )
                .await;
                return Ok(ToolCallResult {
                    result: cached_result.clone(),
                    server: server_name,
                    input_tokens: 0,
                    output_tokens,
                    cache_hit: true,
                    latency_ms: start.elapsed().as_millis() as f64,
                });
            }
        }

        let input_tokens = McpClient::estimate_tokens(&arguments);

        let clients = self.clients.read().await;
        let client = clients
            .get(&server_name)
            .ok_or_else(|| RimuruError::Bridge(format!("Server not connected: {}", server_name)))?;

        let mcp_result = client.tools_call(resolved_name, arguments).await?;
        let result_value = serde_json::to_value(&mcp_result).unwrap_or(json!(null));
        let output_tokens = McpClient::estimate_tokens(&result_value);
        let latency_ms = start.elapsed().as_millis() as f64;

        {
            let mut cache = self.cache.write().await;
            if cache.len() >= self.cache_max {
                let oldest = cache
                    .iter()
                    .min_by_key(|(_, (_, t))| *t)
                    .map(|(k, _)| k.clone());
                if let Some(k) = oldest {
                    cache.remove(&k);
                }
            }
            cache.insert(cache_key, (result_value.clone(), std::time::Instant::now()));
        }

        self.record_metrics(
            kv,
            resolved_name,
            &server_name,
            input_tokens,
            output_tokens,
            false,
            latency_ms,
        )
        .await;

        Ok(ToolCallResult {
            result: result_value,
            server: server_name,
            input_tokens,
            output_tokens,
            cache_hit: false,
            latency_ms,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn record_metrics(
        &self,
        kv: &StateKV,
        tool_name: &str,
        server_name: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_hit: bool,
        latency_ms: f64,
    ) {
        let key = format!("{}::{}", server_name, tool_name);
        let mut metrics: ToolMetrics = kv
            .get(KV_SCOPE_METRICS, &key)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        metrics.call_count += 1;
        metrics.total_input_tokens += input_tokens;
        metrics.total_output_tokens += output_tokens;
        if cache_hit {
            metrics.cache_hits += 1;
        } else {
            metrics.cache_misses += 1;
        }
        let n = metrics.call_count as f64;
        metrics.avg_latency_ms = ((metrics.avg_latency_ms * (n - 1.0)) + latency_ms) / n;
        metrics.last_called = Some(Utc::now().to_rfc3339());

        if let Err(e) = kv.set(KV_SCOPE_METRICS, &key, &metrics).await {
            warn!("Failed to record MCP metrics for {}: {}", key, e);
        }
    }

    pub async fn get_stats(&self, kv: &StateKV) -> Vec<(String, ToolMetrics)> {
        let keys = match kv.list_keys(KV_SCOPE_METRICS).await {
            Ok(k) => k,
            Err(e) => {
                warn!("Failed to list MCP metric keys: {}", e);
                return Vec::new();
            }
        };

        let mut stats = Vec::with_capacity(keys.len());
        for key in keys {
            if let Ok(Some(metrics)) = kv.get::<ToolMetrics>(KV_SCOPE_METRICS, &key).await {
                stats.push((key, metrics));
            }
        }

        stats.sort_by(|a, b| b.1.call_count.cmp(&a.1.call_count));
        stats
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectResult {
    pub server_name: String,
    pub server_info: Option<McpInitializeResult>,
    pub tool_count: usize,
    pub schema_tokens: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolListEntry {
    pub name: String,
    pub server: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    pub schema_tokens: u64,
}

impl ToolListEntry {
    fn from_index(name: &str, server: &str, tool: &McpTool, include_schema: bool) -> Self {
        Self {
            name: name.to_string(),
            server: server.to_string(),
            description: tool.description.clone(),
            input_schema: if include_schema {
                tool.input_schema.clone()
            } else {
                None
            },
            schema_tokens: tool
                .input_schema
                .as_ref()
                .map(McpClient::estimate_tokens)
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolCallResult {
    pub result: Value,
    pub server: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_hit: bool,
    pub latency_ms: f64,
}

fn sha256_short(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
