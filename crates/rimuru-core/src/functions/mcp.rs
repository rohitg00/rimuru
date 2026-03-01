use iii_sdk::III;
use serde_json::{json, Value};

use crate::state::StateKV;

pub fn register(iii: &III, _kv: &StateKV) {
    register_list(iii);
}

fn register_list(iii: &III) {
    iii.register_function("rimuru.mcp.list", move |_input: Value| {
        async move {
            let servers = crate::discovery::discover_mcp_servers().await;
            Ok(json!({
                "servers": servers,
                "total": servers.len()
            }))
        }
    });
}
