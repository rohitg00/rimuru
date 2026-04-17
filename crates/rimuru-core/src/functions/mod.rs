pub mod agents;
pub mod budget;
pub mod config;
pub mod context;
pub mod costs;
pub mod guard;
pub mod hardware;
pub mod health;
pub mod hooks;
pub mod indexer;
pub mod jwt;
pub mod mcp;
pub mod mcp_proxy;
pub mod metrics;
pub mod models;
pub mod optimize;
pub mod plugins;
pub mod runaway;
pub mod sessions;
pub mod skillkit;
pub mod sync;
pub mod sysutil;
pub mod team;

use std::sync::Arc;

use iii_sdk::III;
use tokio::sync::RwLock;

use crate::mcp::proxy::McpProxy;
use crate::state::StateKV;

pub fn register_all(iii: &III, kv: &StateKV) {
    let proxy = Arc::new(RwLock::new(McpProxy::new()));

    agents::register(iii, kv);
    budget::register(iii, kv);
    context::register(iii, kv);
    sessions::register(iii, kv);
    costs::register(iii, kv);
    guard::register(iii, kv);
    models::register(iii, kv);
    metrics::register(iii, kv);
    hooks::register(iii, kv);
    plugins::register(iii, kv);
    mcp::register(iii, kv);
    mcp_proxy::register(iii, kv, proxy);
    indexer::register(iii, kv);
    runaway::register(iii, kv);
    optimize::register(iii, kv);
    skillkit::register(iii, kv);
    sync::register(iii, kv);
    team::register(iii, kv);
    health::register(iii, kv);
    config::register(iii, kv);
    hardware::register(iii, kv);
}
