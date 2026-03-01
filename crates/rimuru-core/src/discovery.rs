use serde_json::{json, Value};

pub async fn discover_plugins() -> Vec<Value> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    let installed_path = home.join(".claude/plugins/installed_plugins.json");
    let settings_path = home.join(".claude/settings.json");

    let settings: Value = read_json_file(&settings_path).await;
    let enabled_plugins = settings.get("enabledPlugins").cloned().unwrap_or(json!({}));
    let installed: Value = read_json_file(&installed_path).await;

    let mut result = Vec::new();

    let plugins = match installed.get("plugins").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return result,
    };

    for (key, entries) in plugins {
        let parts: Vec<&str> = key.split('@').collect();
        let plugin_name = parts.first().copied().unwrap_or("");
        let marketplace = parts.get(1).copied().unwrap_or("");

        let entry = match entries.as_array().and_then(|a| a.first()) {
            Some(e) => e,
            None => continue,
        };

        let install_path = entry
            .get("installPath")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        let version = entry
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");

        let plugin_json_path = format!("{}/.claude-plugin/plugin.json", install_path);
        let (name, description) = match tokio::fs::read_to_string(&plugin_json_path).await {
            Ok(pj_content) => {
                let pj: Value = serde_json::from_str(&pj_content).unwrap_or(json!({}));
                (
                    pj.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(plugin_name)
                        .to_string(),
                    pj.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                )
            }
            Err(_) => (plugin_name.to_string(), String::new()),
        };

        let enabled = enabled_plugins
            .get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let hooks_path = format!("{}/hooks/hooks.json", install_path);
        let hook_events: Vec<String> = match tokio::fs::read_to_string(&hooks_path).await {
            Ok(h_content) => {
                let h: Value = serde_json::from_str(&h_content).unwrap_or(json!({}));
                h.get("hooks")
                    .and_then(|hooks| hooks.as_object())
                    .map(|hooks_map| hooks_map.keys().cloned().collect())
                    .unwrap_or_default()
            }
            Err(_) => vec![],
        };

        result.push(json!({
            "id": key,
            "name": name,
            "version": version,
            "description": description,
            "author": marketplace,
            "enabled": enabled,
            "installed": true,
            "hooks": hook_events,
            "config": {},
            "language": "TypeScript",
            "binary_path": install_path,
            "functions": hook_events,
            "marketplace": marketplace
        }));
    }

    result
}

pub async fn discover_hooks() -> Vec<Value> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    let installed_path = home.join(".claude/plugins/installed_plugins.json");
    let installed: Value = read_json_file(&installed_path).await;

    let mut hooks = Vec::new();

    let plugins = match installed.get("plugins").and_then(|p| p.as_object()) {
        Some(p) => p,
        None => return hooks,
    };

    for (key, entries) in plugins {
        let entry = match entries.as_array().and_then(|a| a.first()) {
            Some(e) => e,
            None => continue,
        };

        let install_path = entry
            .get("installPath")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        let hooks_path = format!("{}/hooks/hooks.json", install_path);

        let h_content = match tokio::fs::read_to_string(&hooks_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let h: Value = serde_json::from_str(&h_content).unwrap_or(json!({}));
        let hooks_map = match h.get("hooks").and_then(|hm| hm.as_object()) {
            Some(m) => m,
            None => continue,
        };

        for (event, matchers) in hooks_map {
            let matcher_list = match matchers.as_array() {
                Some(l) => l,
                None => continue,
            };

            for (idx, matcher) in matcher_list.iter().enumerate() {
                let description = matcher
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Hook");

                let script = matcher
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .and_then(|h| h.first())
                    .and_then(|h| h.get("command"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");

                let matcher_str = matcher
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .unwrap_or("*");

                hooks.push(json!({
                    "id": format!("{}-{}-{}", key, event, idx),
                    "name": description,
                    "event": event,
                    "event_type": event,
                    "function_id": script,
                    "plugin_id": key,
                    "enabled": true,
                    "script": script,
                    "matcher": matcher_str,
                    "timeout_ms": 30000,
                    "priority": 0,
                    "last_run": null,
                    "last_status": null,
                    "run_count": 0,
                    "error_count": 0
                }));
            }
        }
    }

    hooks
}

pub async fn discover_mcp_servers() -> Vec<Value> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };

    let mut servers = Vec::new();

    let claude_code_settings = home.join(".claude/settings.json");
    if let Ok(c) = tokio::fs::read_to_string(&claude_code_settings).await {
        if let Ok(v) = serde_json::from_str::<Value>(&c) {
            extract_mcp_servers(&v, "Claude Code", &mut servers);
        }
    }

    let claude_desktop = home.join("Library/Application Support/Claude/claude_desktop_config.json");
    if let Ok(c) = tokio::fs::read_to_string(&claude_desktop).await {
        if let Ok(v) = serde_json::from_str::<Value>(&c) {
            extract_mcp_servers(&v, "Claude Desktop", &mut servers);
        }
    }

    servers
}

fn extract_mcp_servers(config: &Value, source: &str, servers: &mut Vec<Value>) {
    let mcp_servers = match config.get("mcpServers").and_then(|m| m.as_object()) {
        Some(s) => s,
        None => return,
    };

    for (name, cfg) in mcp_servers {
        let command = cfg.get("command").and_then(|c| c.as_str()).unwrap_or("");
        let args = cfg
            .get("args")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let env = cfg.get("env").cloned().unwrap_or(json!({}));

        servers.push(json!({
            "id": format!("{}:{}", source, name),
            "name": name,
            "command": command,
            "args": args,
            "env": mask_env_values(&env),
            "enabled": true,
            "source": source
        }));
    }
}

pub fn mask_env_values(env: &Value) -> Value {
    match env.as_object() {
        Some(map) => {
            let masked: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| {
                    let val = match v.as_str() {
                        Some(s) if s.len() > 8 => {
                            Value::String(format!("{}...{}", &s[..4], &s[s.len() - 4..]))
                        }
                        _ => v.clone(),
                    };
                    (k.clone(), val)
                })
                .collect();
            Value::Object(masked)
        }
        None => json!({}),
    }
}

async fn read_json_file(path: &std::path::Path) -> Value {
    match tokio::fs::read_to_string(path).await {
        Ok(c) => serde_json::from_str(&c).unwrap_or(json!({})),
        Err(_) => json!({}),
    }
}
