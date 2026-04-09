use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};

use crate::state::StateKV;

pub fn register(iii: &III, _kv: &StateKV) {
    register_list(iii);
}

fn register_list(iii: &III) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.mcp.list".to_string()),
        move |_input: Value| async move {
            let servers = crate::discovery::discover_mcp_servers().await;
            Ok(json!({
                "servers": servers,
                "total": servers.len()
            }))
        },
    );
}
