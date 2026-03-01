use iii_sdk::{Bridge, Streams, UpdateOp, WorkerMetadata};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};

const ENGINE_URL: &str = "ws://127.0.0.1:49134";
const PLUGIN_NAME: &str = "example";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
struct TransformInput {
    text: String,
    #[serde(default)]
    operation: TransformOp,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TransformOp {
    #[default]
    Uppercase,
    Lowercase,
    Reverse,
    WordCount,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    info!("Starting rimuru example plugin v{}", PLUGIN_VERSION);

    let bridge = Bridge::with_metadata(
        ENGINE_URL,
        WorkerMetadata {
            runtime: "rust".to_string(),
            version: PLUGIN_VERSION.to_string(),
            name: format!("plugin.{}", PLUGIN_NAME),
            os: std::env::consts::OS.to_string(),
        },
    );

    bridge.connect().await?;
    info!("Connected to iii engine at {}", ENGINE_URL);

    let streams = Streams::new(bridge.clone());

    register_hello(&bridge);
    register_transform(&bridge);
    register_cost_hook(&bridge, &streams);
    register_manifest(&bridge);

    info!("Plugin '{}' is ready — press Ctrl+C to stop", PLUGIN_NAME);

    tokio::signal::ctrl_c().await?;
    info!("Shutting down plugin '{}'", PLUGIN_NAME);
    bridge.disconnect();

    Ok(())
}

fn register_hello(bridge: &Bridge) {
    bridge.register_function("plugin.example.hello", |input: Value| async move {
        let name = input
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("world");

        info!("plugin.example.hello called with name={}", name);

        Ok(json!({
            "message": format!("Hello, {}! Greetings from the Rust example plugin.", name),
            "plugin": PLUGIN_NAME,
            "version": PLUGIN_VERSION,
            "language": "rust"
        }))
    });

    info!("Registered function: plugin.example.hello");
}

fn register_transform(bridge: &Bridge) {
    bridge.register_function("plugin.example.transform", |input: Value| async move {
        let parsed: TransformInput = serde_json::from_value(input).map_err(|e| {
            iii_sdk::IIIError::Handler(format!(
                "invalid input — expected {{\"text\": \"...\", \"operation\": \"uppercase|lowercase|reverse|word_count\"}}: {}",
                e
            ))
        })?;

        info!(
            "plugin.example.transform called — op={:?}, text_len={}",
            parsed.operation,
            parsed.text.len()
        );

        let result = match parsed.operation {
            TransformOp::Uppercase => parsed.text.to_uppercase(),
            TransformOp::Lowercase => parsed.text.to_lowercase(),
            TransformOp::Reverse => parsed.text.chars().rev().collect(),
            TransformOp::WordCount => {
                let count = parsed.text.split_whitespace().count();
                format!("{}", count)
            }
        };

        Ok(json!({
            "original": parsed.text,
            "result": result,
            "operation": format!("{:?}", parsed.operation).to_lowercase()
        }))
    });

    info!("Registered function: plugin.example.transform");
}

fn register_cost_hook(bridge: &Bridge, streams: &Streams) {
    let streams = streams.clone();

    bridge.register_function("plugin.example.on_cost_recorded", move |input: Value| {
        let streams = streams.clone();
        async move {
            let amount = input
                .get("amount")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let agent_id = input
                .get("agent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let record_id = input
                .get("record_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            info!(
                "Hook fired: cost_recorded — record={}, agent={}, amount=${:.6}",
                record_id, agent_id, amount
            );

            if amount > 1.0 {
                warn!(
                    "High cost alert: ${:.4} from agent {}",
                    amount, agent_id
                );
            }

            let counter_key = format!("plugin_state::{}::cost_hook_count", PLUGIN_NAME);
            match streams
                .update(
                    &counter_key,
                    vec![UpdateOp::merge(json!({
                        "invocations": 1,
                        "last_amount": amount,
                        "last_agent": agent_id
                    }))],
                )
                .await
            {
                Ok(_) => info!("Updated plugin cost hook state"),
                Err(e) => error!("Failed to update plugin state: {}", e),
            }

            Ok(json!({
                "handled": true,
                "plugin": PLUGIN_NAME,
                "alert": amount > 1.0
            }))
        }
    });

    info!("Registered hook handler: plugin.example.on_cost_recorded (event: cost_recorded)");
}

fn register_manifest(bridge: &Bridge) {
    bridge.register_function("plugin.example.manifest", |_input: Value| async move {
        Ok(json!({
            "id": format!("plugin.{}", PLUGIN_NAME),
            "name": "Example Rust Plugin",
            "version": PLUGIN_VERSION,
            "language": "rust",
            "description": "A reference Rust plugin for rimuru that demonstrates function registration, input parsing, state access, and hook handling.",
            "functions": [
                {
                    "id": "plugin.example.hello",
                    "description": "Returns a greeting message",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string", "description": "Name to greet" }
                        }
                    }
                },
                {
                    "id": "plugin.example.transform",
                    "description": "Transforms text using the specified operation",
                    "input_schema": {
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": { "type": "string", "description": "Text to transform" },
                            "operation": {
                                "type": "string",
                                "enum": ["uppercase", "lowercase", "reverse", "word_count"],
                                "default": "uppercase"
                            }
                        }
                    }
                }
            ],
            "hooks": [
                {
                    "event_type": "cost_recorded",
                    "function_id": "plugin.example.on_cost_recorded",
                    "priority": 10
                }
            ]
        }))
    });

    info!("Registered function: plugin.example.manifest");
}
