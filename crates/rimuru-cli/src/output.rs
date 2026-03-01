use comfy_table::{Cell, CellAlignment, Color, ContentArrangement, Table};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

fn new_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(headers.iter().map(|h| Cell::new(*h).fg(Color::Cyan)).collect::<Vec<_>>());
    table
}

fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(|v| v.as_str()).unwrap_or("-")
}

fn u64_field(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn f64_field(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn bool_field(v: &Value, key: &str) -> bool {
    v.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn yes_no(val: bool) -> &'static str {
    if val { "yes" } else { "no" }
}

fn status_cell(status: &str, colors: &[(&str, Color)]) -> Cell {
    for (s, c) in colors {
        if *s == status {
            return Cell::new(status).fg(*c);
        }
    }
    Cell::new(status)
}

pub fn print_value<T: Serialize>(data: &T, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(data).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"));
            println!("{json}");
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml_to_string(data);
            println!("{yaml}");
        }
        OutputFormat::Table => {
            let json = serde_json::to_value(data).unwrap_or(Value::Null);
            println!("{}", value_to_kv_table(&json));
        }
    }
}

fn serde_yaml_to_string<T: Serialize + ?Sized>(data: &T) -> String {
    let val = serde_json::to_value(data).unwrap_or(Value::Null);
    yaml_from_value(&val, 0)
}

fn yaml_from_value(val: &Value, indent: usize) -> String {
    let pad = " ".repeat(indent);
    match val {
        Value::Null => format!("{pad}null"),
        Value::Bool(b) => format!("{pad}{b}"),
        Value::Number(n) => format!("{pad}{n}"),
        Value::String(s) => format!("{pad}{s}"),
        Value::Array(arr) => {
            if arr.is_empty() {
                return format!("{pad}[]");
            }
            arr.iter()
                .map(|v| {
                    let inner = yaml_from_value(v, 0);
                    format!("{pad}- {inner}")
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        Value::Object(map) => {
            if map.is_empty() {
                return format!("{pad}{{}}");
            }
            map.iter()
                .map(|(k, v)| match v {
                    Value::Object(_) | Value::Array(_) => {
                        let inner = yaml_from_value(v, indent + 2);
                        format!("{pad}{k}:\n{inner}")
                    }
                    _ => {
                        let inner = yaml_from_value(v, 0);
                        format!("{pad}{k}: {inner}")
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

fn value_to_kv_table(val: &Value) -> String {
    match val {
        Value::Object(map) => {
            let mut table = new_table(&["Key", "Value"]);
            for (k, v) in map {
                table.add_row(vec![Cell::new(k).fg(Color::Green), Cell::new(flat_value(v))]);
            }
            table.to_string()
        }
        _ => format!("{val}"),
    }
}

fn flat_value(val: &Value) -> String {
    match val {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(flat_value).collect();
            items.join(", ")
        }
        Value::Object(_) => serde_json::to_string(val).unwrap_or_default(),
    }
}

pub fn format_agents_list(agents: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(agents).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(agents),
        OutputFormat::Table => {
            let mut table = new_table(&["ID", "Name", "Type", "Status", "Version", "Sessions", "Est. Cost"]);
            let agent_status_colors = [
                ("connected", Color::Green), ("active", Color::Green),
                ("idle", Color::Yellow), ("disconnected", Color::DarkGrey),
                ("error", Color::Red),
            ];
            for agent in agents {
                let id = str_field(agent, "id");
                table.add_row(vec![
                    Cell::new(&id[..8.min(id.len())]),
                    Cell::new(str_field(agent, "name")),
                    Cell::new(str_field(agent, "agent_type")),
                    status_cell(str_field(agent, "status"), &agent_status_colors),
                    Cell::new(str_field(agent, "version")),
                    Cell::new(u64_field(agent, "session_count")).set_alignment(CellAlignment::Right),
                    Cell::new(format!("${:.4}", f64_field(agent, "total_cost"))),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_sessions_list(sessions: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(sessions).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(sessions),
        OutputFormat::Table => {
            let mut table = new_table(&["ID", "Agent", "Status", "Model", "Messages", "Tokens", "Cost", "Started"]);
            let session_status_colors = [
                ("active", Color::Green), ("completed", Color::Blue),
                ("abandoned", Color::Yellow), ("error", Color::Red),
            ];
            for session in sessions {
                let id = str_field(session, "id");
                let started = str_field(session, "started_at");
                let started_short = if started.len() > 16 { &started[..16] } else { started };
                table.add_row(vec![
                    Cell::new(&id[..8.min(id.len())]),
                    Cell::new(str_field(session, "agent_type")),
                    status_cell(str_field(session, "status"), &session_status_colors),
                    Cell::new(str_field(session, "model")),
                    Cell::new(u64_field(session, "messages")).set_alignment(CellAlignment::Right),
                    Cell::new(format_tokens(u64_field(session, "total_tokens"))),
                    Cell::new(format!("${:.4}", f64_field(session, "total_cost"))),
                    Cell::new(started_short),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_costs_summary(costs: &Value, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(costs).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(costs),
        OutputFormat::Table => {
            let mut out = String::new();

            let mut summary = new_table(&["Metric", "Value"]);
            summary.add_row(vec![Cell::new("Est. Cost (API rates)"), Cell::new(format!("${:.2}", f64_field(costs, "total_cost")))]);
            summary.add_row(vec![Cell::new("Input Tokens"), Cell::new(format_tokens(u64_field(costs, "total_input_tokens")))]);
            summary.add_row(vec![Cell::new("Output Tokens"), Cell::new(format_tokens(u64_field(costs, "total_output_tokens")))]);
            summary.add_row(vec![Cell::new("Sessions"), Cell::new(u64_field(costs, "total_records"))]);
            out.push_str(&summary.to_string());

            if let Some(by_agent) = costs.get("by_agent").and_then(|v| v.as_array()) {
                if !by_agent.is_empty() {
                    out.push_str("\n\nBy Agent:\n");
                    let mut agent_table = new_table(&["Agent", "Type", "Cost", "Records"]);
                    for a in by_agent {
                        agent_table.add_row(vec![
                            Cell::new(str_field(a, "agent_name")),
                            Cell::new(str_field(a, "agent_type")),
                            Cell::new(format!("${:.4}", f64_field(a, "total_cost"))),
                            Cell::new(u64_field(a, "record_count")).set_alignment(CellAlignment::Right),
                        ]);
                    }
                    out.push_str(&agent_table.to_string());
                }
            }

            if let Some(by_model) = costs.get("by_model").and_then(|v| v.as_array()) {
                if !by_model.is_empty() {
                    out.push_str("\n\nBy Model:\n");
                    let mut model_table = new_table(&["Model", "Provider", "Cost", "Tokens"]);
                    for m in by_model {
                        model_table.add_row(vec![
                            Cell::new(str_field(m, "model")),
                            Cell::new(str_field(m, "provider")),
                            Cell::new(format!("${:.4}", f64_field(m, "total_cost"))),
                            Cell::new(format_tokens(u64_field(m, "total_tokens"))),
                        ]);
                    }
                    out.push_str(&model_table.to_string());
                }
            }

            out
        }
    }
}

pub fn format_daily_costs(days: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(days).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(days),
        OutputFormat::Table => {
            let mut table = new_table(&["Date", "Cost", "Input Tokens", "Output Tokens", "Records"]);
            for day in days {
                table.add_row(vec![
                    Cell::new(str_field(day, "date")),
                    Cell::new(format!("${:.4}", f64_field(day, "total_cost"))),
                    Cell::new(format_tokens(u64_field(day, "total_input_tokens"))),
                    Cell::new(format_tokens(u64_field(day, "total_output_tokens"))),
                    Cell::new(u64_field(day, "record_count")).set_alignment(CellAlignment::Right),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_models_list(models: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(models).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(models),
        OutputFormat::Table => {
            let mut table = new_table(&["ID", "Name", "Provider", "Input $/1M", "Output $/1M", "Context", "Vision", "Tools"]);
            for model in models {
                table.add_row(vec![
                    Cell::new(str_field(model, "id")),
                    Cell::new(str_field(model, "name")),
                    Cell::new(str_field(model, "provider")),
                    Cell::new(format!("${:.2}", f64_field(model, "input_price_per_million"))),
                    Cell::new(format!("${:.2}", f64_field(model, "output_price_per_million"))),
                    Cell::new(format_tokens(u64_field(model, "context_window"))),
                    Cell::new(yes_no(bool_field(model, "supports_vision"))),
                    Cell::new(yes_no(bool_field(model, "supports_tools"))),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_metrics(metrics: &Value, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(metrics).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(metrics),
        OutputFormat::Table => {
            let mut table = new_table(&["Metric", "Value"]);

            let fields = [
                ("CPU Usage", "cpu_usage_percent", "%"),
                ("Memory Used", "memory_used_mb", " MB"),
                ("Memory Total", "memory_total_mb", " MB"),
                ("Active Agents", "active_agents", ""),
                ("Active Sessions", "active_sessions", ""),
                ("Cost Today", "total_cost_today", ""),
                ("Req/min", "requests_per_minute", ""),
                ("Avg Response", "avg_response_time_ms", " ms"),
                ("Error Rate", "error_rate", "%"),
                ("Uptime", "uptime_secs", " s"),
            ];

            for (label, key, suffix) in &fields {
                let val = if *key == "total_cost_today" {
                    let n = metrics.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    format!("${n:.4}")
                } else if let Some(n) = metrics.get(key).and_then(|v| v.as_f64()) {
                    format!("{n:.2}{suffix}")
                } else if let Some(n) = metrics.get(key).and_then(|v| v.as_u64()) {
                    format!("{n}{suffix}")
                } else {
                    "-".into()
                };
                table.add_row(vec![Cell::new(label), Cell::new(val)]);
            }

            if let Some(ts) = metrics.get("timestamp").and_then(|v| v.as_str()) {
                table.add_row(vec![Cell::new("Timestamp"), Cell::new(ts)]);
            }

            table.to_string()
        }
    }
}

pub fn format_metrics_history(entries: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(entries).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(entries),
        OutputFormat::Table => {
            let mut table = new_table(&["Timestamp", "CPU %", "Mem MB", "Agents", "Sessions", "Cost", "Err %"]);
            for entry in entries {
                let ts = str_field(entry, "timestamp");
                let ts_short = if ts.len() > 19 { &ts[..19] } else { ts };
                table.add_row(vec![
                    Cell::new(ts_short),
                    Cell::new(format!("{:.1}", f64_field(entry, "cpu_usage_percent"))),
                    Cell::new(format!("{:.0}", f64_field(entry, "memory_used_mb"))),
                    Cell::new(u64_field(entry, "active_agents")).set_alignment(CellAlignment::Right),
                    Cell::new(u64_field(entry, "active_sessions")).set_alignment(CellAlignment::Right),
                    Cell::new(format!("${:.4}", f64_field(entry, "total_cost_today"))),
                    Cell::new(format!("{:.2}", f64_field(entry, "error_rate"))),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_plugins_list(plugins: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(plugins).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(plugins),
        OutputFormat::Table => {
            let mut table = new_table(&["ID", "Name", "Version", "Language", "Enabled", "Functions"]);
            for plugin in plugins {
                let fns = plugin.get("functions").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                table.add_row(vec![
                    Cell::new(str_field(plugin, "id")),
                    Cell::new(str_field(plugin, "name")),
                    Cell::new(str_field(plugin, "version")),
                    Cell::new(str_field(plugin, "language")),
                    Cell::new(yes_no(bool_field(plugin, "enabled"))),
                    Cell::new(fns).set_alignment(CellAlignment::Right),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_hooks_list(hooks: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(hooks).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(hooks),
        OutputFormat::Table => {
            let mut table = new_table(&["ID", "Event Type", "Function", "Priority", "Enabled"]);
            for hook in hooks {
                let priority = hook.get("priority").and_then(|v| v.as_i64()).unwrap_or(0);
                table.add_row(vec![
                    Cell::new(str_field(hook, "id")),
                    Cell::new(str_field(hook, "event_type")),
                    Cell::new(str_field(hook, "function_id")),
                    Cell::new(priority).set_alignment(CellAlignment::Right),
                    Cell::new(yes_no(bool_field(hook, "enabled"))),
                ]);
            }
            table.to_string()
        }
    }
}

pub fn format_health(health: &Value, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(health).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(health),
        OutputFormat::Table => {
            let mut table = new_table(&["Component", "Status"]);
            let health_colors = [
                ("healthy", Color::Green), ("ok", Color::Green),
                ("degraded", Color::Yellow),
            ];

            table.add_row(vec![Cell::new("Overall"), status_cell(str_field(health, "status"), &health_colors)]);

            if let Some(components) = health.get("components").and_then(|v| v.as_object()) {
                for (name, info) in components {
                    table.add_row(vec![Cell::new(name), status_cell(str_field(info, "status"), &health_colors)]);
                }
            }

            if let Some(uptime) = health.get("uptime_secs").and_then(|v| v.as_u64()) {
                table.add_row(vec![Cell::new("Uptime"), Cell::new(format_uptime(uptime))]);
            }

            if let Some(version) = health.get("version").and_then(|v| v.as_str()) {
                table.add_row(vec![Cell::new("Version"), Cell::new(version)]);
            }

            table.to_string()
        }
    }
}

pub fn format_config(config: &Value, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(config).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(config),
        OutputFormat::Table => {
            let mut table = new_table(&["Key", "Value"]);
            if let Some(obj) = config.as_object() {
                for (k, v) in obj {
                    table.add_row(vec![Cell::new(k).fg(Color::Green), Cell::new(flat_value(v))]);
                }
            } else {
                table.add_row(vec![Cell::new("value"), Cell::new(flat_value(config))]);
            }
            table.to_string()
        }
    }
}

pub fn format_detected_agents(agents: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(agents).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(agents),
        OutputFormat::Table => {
            let mut table = new_table(&["Agent", "Type", "Installed", "Registered"]);
            for agent in agents {
                let name = agent.get("display_name").or_else(|| agent.get("name"))
                    .and_then(|v| v.as_str()).unwrap_or("-");
                let installed = bool_field(agent, "installed");
                let registered = bool_field(agent, "registered");
                let bool_cell = |val: bool| if val { Cell::new("yes").fg(Color::Green) } else { Cell::new("no").fg(Color::DarkGrey) };
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new(str_field(agent, "agent_type")),
                    bool_cell(installed),
                    bool_cell(registered),
                ]);
            }
            table.to_string()
        }
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

pub fn format_mcp_list(servers: &[Value], format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(servers).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml_to_string(servers),
        OutputFormat::Table => {
            let mut table = new_table(&["Name", "Command", "Args", "Source", "Enabled"]);
            for server in servers {
                let args = server.get("args")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();
                table.add_row(vec![
                    Cell::new(str_field(server, "name")),
                    Cell::new(str_field(server, "command")),
                    Cell::new(if args.is_empty() { "-".to_string() } else { args }),
                    Cell::new(str_field(server, "source")),
                    Cell::new(yes_no(bool_field(server, "enabled"))),
                ]);
            }
            table.to_string()
        }
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m {seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
