use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post, put};
use axum::Router;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};

const UI_HTML: &str = include_str!("../../../ui/dist/index.html");

use crate::discovery;
use crate::state::StateKV;

struct MetricSnapshot {
    timestamp: String,
    cpu: f64,
    memory: f64,
    requests: f64,
    connections: f64,
}

type AppState = Arc<AppStateInner>;

struct AppStateInner {
    kv: StateKV,
    metrics_buffer: Mutex<VecDeque<MetricSnapshot>>,
}

async fn call_function(kv: &StateKV, function_id: &str, input: Value) -> Result<Value, StatusCode> {
    kv.iii()
        .trigger(function_id, input)
        .await
        .map_err(|e| {
            tracing::error!("Function call failed: {} - {}", function_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

fn unwrap_field(v: Value, field: &str) -> Value {
    match v {
        Value::Object(ref map) => map.get(field).cloned().unwrap_or(v),
        _ => v,
    }
}

async fn api_agents_list(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.list", json!({})).await {
        Ok(v) => Json(unwrap_field(v, "agents")).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_agents_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.get", json!({"agent_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_agents_register(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.create", body).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_agents_detect(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.detect", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_agents_connect(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.connect", body).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_agents_disconnect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.agents.disconnect", json!({"agent_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_sessions_list(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.sessions.list", json!({})).await {
        Ok(v) => Json(unwrap_field(v, "sessions")).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_sessions_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.sessions.get", json!({"session_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_sessions_active(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.sessions.active", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_sessions_history(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.sessions.history", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_costs_summary(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.costs.summary", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_costs_daily(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.costs.daily", json!({})).await {
        Ok(v) => {
            let daily = unwrap_field(v, "daily");
            let fixed = match daily {
                Value::Array(items) => {
                    let mapped: Vec<Value> = items
                        .into_iter()
                        .map(|mut item| {
                            if let Value::Object(ref mut map) = item {
                                if let Some(tc) = map.remove("total_cost") {
                                    map.insert("cost".to_string(), tc);
                                }
                            }
                            item
                        })
                        .collect();
                    Value::Array(mapped)
                }
                other => other,
            };
            Json(fixed).into_response()
        }
        Err(s) => s.into_response(),
    }
}

async fn api_costs_by_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.costs.by_agent", json!({"agent_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_costs_record(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.costs.record", body).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_costs_list(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.costs.summary", json!({})).await {
        Ok(v) => {
            let summary = unwrap_field(v, "summary");
            if let Some(by_agent) = summary.get("by_agent").and_then(|a| a.as_array()) {
                let records: Vec<Value> = by_agent
                    .iter()
                    .map(|a| {
                        let agent_name = a.get("agent_name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        json!({
                            "agent_name": agent_name,
                            "model": agent_name,
                            "cost": a.get("total_cost").and_then(|c| c.as_f64()).unwrap_or(0.0),
                            "total_cost": a.get("total_cost").and_then(|c| c.as_f64()).unwrap_or(0.0),
                            "input_tokens": a.get("total_input_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
                            "output_tokens": a.get("total_output_tokens").and_then(|t| t.as_u64()).unwrap_or(0),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "recorded_at": chrono::Utc::now().to_rfc3339(),
                        })
                    })
                    .collect();
                Json(Value::Array(records)).into_response()
            } else {
                Json(json!([])).into_response()
            }
        }
        Err(s) => s.into_response(),
    }
}

async fn api_models_list(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.models.list", json!({})).await {
        Ok(v) => Json(unwrap_field(v, "models")).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_models_sync(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.models.sync", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_models_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.models.get", json!({"model_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_metrics_current(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.metrics.current", json!({})).await {
        Ok(v) => Json(unwrap_field(v, "metrics")).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_metrics_history(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.metrics.history", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_metrics_timeline(State(state): State<AppState>) -> impl IntoResponse {
    let buf = state.metrics_buffer.lock().await;
    let timestamps: Vec<&str> = buf.iter().map(|s| s.timestamp.as_str()).collect();
    let cpu: Vec<f64> = buf.iter().map(|s| s.cpu).collect();
    let memory: Vec<f64> = buf.iter().map(|s| s.memory).collect();
    let requests: Vec<f64> = buf.iter().map(|s| s.requests).collect();
    let connections: Vec<f64> = buf.iter().map(|s| s.connections).collect();

    Json(json!({
        "timestamps": timestamps,
        "cpu": cpu,
        "memory": memory,
        "requests": requests,
        "connections": connections
    }))
}

async fn api_health(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.health.check", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_hooks_list() -> impl IntoResponse {
    Json(Value::Array(discovery::discover_hooks().await))
}

async fn api_hooks_register(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.hooks.register", body).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_hooks_dispatch(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.hooks.dispatch", body).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_hooks_update(Path(_id): Path<String>) -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

async fn api_hooks_executions() -> impl IntoResponse {
    Json(json!([]))
}

async fn api_plugins_list() -> impl IntoResponse {
    Json(Value::Array(discovery::discover_plugins().await))
}

async fn api_plugins_install(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.plugins.install", body).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_plugins_uninstall(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.plugins.uninstall", json!({"plugin_id": id})).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_plugins_toggle(
    State(state): State<AppState>,
    Path((id, action)): Path<(String, String)>,
) -> impl IntoResponse {
    let enabled = action == "enable";
    match call_function(&state.kv, "rimuru.plugins.toggle", json!({"plugin_id": id, "enabled": enabled})).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => Json(json!({"status": "ok", "enabled": enabled})).into_response(),
    }
}

async fn api_mcp_list() -> impl IntoResponse {
    Json(Value::Array(discovery::discover_mcp_servers().await))
}

async fn api_config_get(State(state): State<AppState>) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.config.get", json!({})).await {
        Ok(v) => Json(unwrap_field(v, "config")).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_config_set(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    match call_function(&state.kv, "rimuru.config.set", body).await {
        Ok(v) => Json(v).into_response(),
        Err(s) => s.into_response(),
    }
}

async fn api_stats(State(state): State<AppState>) -> impl IntoResponse {
    let agents_result = call_function(&state.kv, "rimuru.agents.list", json!({})).await;
    let sessions_result = call_function(&state.kv, "rimuru.sessions.list", json!({})).await;
    let costs_result = call_function(&state.kv, "rimuru.costs.summary", json!({})).await;

    let (total_agents, active_agents) = match agents_result {
        Ok(ref v) => {
            let agents = v.get("agents").and_then(|a| a.as_array());
            let total = agents.map(|a| a.len()).unwrap_or(0);
            let active = agents
                .map(|a| {
                    a.iter()
                        .filter(|agent| {
                            agent
                                .get("status")
                                .and_then(|s| s.as_str())
                                .map(|s| s == "active" || s == "connected")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);
            (total, active)
        }
        Err(_) => (0, 0),
    };

    let (total_sessions, active_sessions) = match sessions_result {
        Ok(ref v) => {
            let total = v.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as usize;
            let sessions = v.get("sessions").and_then(|s| s.as_array());
            let active = sessions
                .map(|s| {
                    s.iter()
                        .filter(|sess| {
                            sess.get("status")
                                .and_then(|st| st.as_str())
                                .map(|st| st == "active")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);
            (total, active)
        }
        Err(_) => (0, 0),
    };

    let (total_cost, total_cost_today, total_tokens, models_used) = match costs_result {
        Ok(ref v) => {
            let summary = v.get("summary").unwrap_or(v);
            let tc = summary
                .get("total_cost")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0);
            let tct = summary
                .get("total_cost_today")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0);
            let tt = summary
                .get("total_input_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0)
                + summary
                    .get("total_output_tokens")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0);
            let mu = summary
                .get("by_model")
                .and_then(|m| m.as_array())
                .map(|m| m.len())
                .unwrap_or(0);
            (tc, tct, tt, mu)
        }
        Err(_) => (0.0, 0.0, 0, 0),
    };

    let plugins = discovery::discover_plugins().await;
    let hooks = discovery::discover_hooks().await;
    let plugins_installed = plugins.len();
    let hooks_active = hooks.iter().filter(|h| {
        h.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false)
    }).count();

    Json(json!({
        "total_cost": total_cost,
        "total_cost_today": total_cost_today,
        "active_agents": active_agents,
        "total_agents": total_agents,
        "active_sessions": active_sessions,
        "total_sessions": total_sessions,
        "total_tokens": total_tokens,
        "models_used": models_used,
        "plugins_installed": plugins_installed,
        "hooks_active": hooks_active
    }))
}

async fn api_activity(State(state): State<AppState>) -> impl IntoResponse {
    let mut events: Vec<Value> = Vec::new();

    if let Ok(v) = call_function(&state.kv, "rimuru.agents.list", json!({})).await {
        if let Some(agents) = v.get("agents").and_then(|a| a.as_array()) {
            for agent in agents {
                let name = agent.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                let status = agent.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                let ts = agent.get("last_seen")
                    .or_else(|| agent.get("connected_at"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                if !ts.is_empty() {
                    let event_type = if status == "connected" { "agent_connected" } else { "agent_disconnected" };
                    events.push(json!({
                        "id": format!("agent-{}", agent.get("id").and_then(|i| i.as_str()).unwrap_or("0")),
                        "type": event_type,
                        "message": format!("{} {}", name, if status == "connected" { "connected" } else { "disconnected" }),
                        "agent_id": agent.get("id"),
                        "timestamp": ts,
                        "metadata": {}
                    }));
                }
            }
        }
    }

    if let Ok(v) = call_function(&state.kv, "rimuru.sessions.list", json!({})).await {
        if let Some(sessions) = v.get("sessions").and_then(|s| s.as_array()) {
            let recent: Vec<&Value> = sessions.iter().take(20).collect();
            for sess in recent {
                let model = sess.get("model").and_then(|m| m.as_str()).unwrap_or("unknown");
                let status = sess.get("status").and_then(|s| s.as_str()).unwrap_or("completed");
                let agent_type = sess.get("agent_type").and_then(|t| t.as_str()).unwrap_or("agent");
                let cost = sess.get("total_cost").and_then(|c| c.as_f64()).unwrap_or(0.0);
                let ts = sess.get("started_at").and_then(|t| t.as_str()).unwrap_or("");
                let ended = sess.get("ended_at").and_then(|t| t.as_str()).unwrap_or("");
                let sid = sess.get("id").and_then(|i| i.as_str()).unwrap_or("0");

                if !ts.is_empty() {
                    events.push(json!({
                        "id": format!("sess-start-{}", sid),
                        "type": "session_started",
                        "message": format!("Session started ({} on {})", agent_type.replace('_', " "), model),
                        "agent_id": sess.get("agent_id"),
                        "timestamp": ts,
                        "metadata": {"model": model, "cost": cost}
                    }));
                }

                if !ended.is_empty() && status != "active" {
                    events.push(json!({
                        "id": format!("sess-end-{}", sid),
                        "type": "session_ended",
                        "message": format!("Session {} (${:.2} spent)", status, cost),
                        "agent_id": sess.get("agent_id"),
                        "timestamp": ended,
                        "metadata": {"model": model, "cost": cost}
                    }));
                }
            }
        }
    }

    events.sort_by(|a, b| {
        let ta = a.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
        let tb = b.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
        tb.cmp(ta)
    });
    events.truncate(20);

    Json(Value::Array(events))
}

async fn serve_ui() -> impl IntoResponse {
    Html(UI_HTML)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_ui))
        .route("/api/stats", get(api_stats))
        .route("/api/activity", get(api_activity))
        .route("/api/activity/recent", get(api_activity))
        .route("/api/agents", get(api_agents_list).post(api_agents_register))
        .route("/api/agents/detect", get(api_agents_detect))
        .route("/api/agents/connect", post(api_agents_connect))
        .route("/api/agents/{id}", get(api_agents_get))
        .route("/api/agents/{id}/disconnect", post(api_agents_disconnect))
        .route("/api/sessions", get(api_sessions_list))
        .route("/api/sessions/active", get(api_sessions_active))
        .route("/api/sessions/history", get(api_sessions_history))
        .route("/api/sessions/{id}", get(api_sessions_get))
        .route("/api/costs/summary", get(api_costs_summary))
        .route("/api/costs/daily", get(api_costs_daily))
        .route("/api/costs/agent/{id}", get(api_costs_by_agent))
        .route("/api/costs", get(api_costs_list).post(api_costs_record))
        .route("/api/models", get(api_models_list))
        .route("/api/models/sync", post(api_models_sync))
        .route("/api/models/{id}", get(api_models_get))
        .route("/api/metrics", get(api_metrics_current))
        .route("/api/metrics/history", get(api_metrics_history))
        .route("/api/metrics/timeline", get(api_metrics_timeline))
        .route("/api/health", get(api_health))
        .route("/api/hooks", get(api_hooks_list))
        .route("/api/hooks/register", post(api_hooks_register))
        .route("/api/hooks/dispatch", post(api_hooks_dispatch))
        .route("/api/hooks/executions", get(api_hooks_executions))
        .route("/api/hooks/{id}", put(api_hooks_update))
        .route("/api/plugins", get(api_plugins_list))
        .route("/api/plugins/install", post(api_plugins_install))
        .route("/api/plugins/{id}", axum::routing::delete(api_plugins_uninstall))
        .route("/api/plugins/{id}/{action}", post(api_plugins_toggle))
        .route("/api/mcp", get(api_mcp_list))
        .route("/api/config", get(api_config_get).post(api_config_set).put(api_config_set))
        .fallback(get(serve_ui))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub async fn serve(kv: StateKV, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state: AppState = Arc::new(AppStateInner {
        kv,
        metrics_buffer: Mutex::new(VecDeque::with_capacity(120)),
    });

    let bg_state = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            if let Ok(v) = call_function(&bg_state.kv, "rimuru.metrics.current", json!({})).await {
                let m = v.get("metrics").unwrap_or(&v);
                let snapshot = MetricSnapshot {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    cpu: m.get("cpu_usage_percent").and_then(|c| c.as_f64()).unwrap_or(0.0),
                    memory: m.get("memory_used_mb").and_then(|c| c.as_f64()).unwrap_or(0.0),
                    requests: m.get("requests_per_minute").and_then(|c| c.as_f64()).unwrap_or(0.0) / 60.0,
                    connections: m.get("active_agents").and_then(|c| c.as_f64()).unwrap_or(0.0),
                };
                let mut buf = bg_state.metrics_buffer.lock().await;
                if buf.len() >= 120 {
                    buf.pop_front();
                }
                buf.push_back(snapshot);
            }
        }
    });

    let app = router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
