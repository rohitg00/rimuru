pub mod agents;
pub mod config;
pub mod costs;
pub mod health;
pub mod hooks;
pub mod mcp;
pub mod metrics;
pub mod models;
pub mod plugins;
pub mod sessions;
pub mod skillkit;

use iii_sdk::III;

use crate::state::StateKV;

pub fn register_all(iii: &III, kv: &StateKV) {
    agents::register(iii, kv);
    sessions::register(iii, kv);
    costs::register(iii, kv);
    models::register(iii, kv);
    metrics::register(iii, kv);
    hooks::register(iii, kv);
    plugins::register(iii, kv);
    mcp::register(iii, kv);
    skillkit::register(iii, kv);
    health::register(iii, kv);
    config::register(iii, kv);
}
