use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_search(iii, kv);
    register_install(iii, kv);
    register_translate(iii, kv);
    register_recommend(iii, kv);
}

async fn run_skillkit_command(args: &[&str]) -> Result<Value, iii_sdk::IIIError> {
    let output = tokio::process::Command::new("npx")
        .arg("skillkit")
        .args(args)
        .arg("--json")
        .output()
        .await
        .map_err(|e| iii_sdk::IIIError::Handler(format!("failed to run skillkit: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let json_output = serde_json::from_str::<Value>(&stdout).ok();
        if let Some(parsed) = json_output {
            return Ok(json!({
                "success": false,
                "output": parsed,
                "exit_code": output.status.code()
            }));
        }

        return Err(iii_sdk::IIIError::Handler(format!(
            "skillkit {} failed (exit {}): {}",
            args.first().unwrap_or(&""),
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            }
        )));
    }

    let parsed: Value = serde_json::from_str(&stdout).unwrap_or_else(|_| {
        json!({
            "raw_output": stdout.trim(),
            "success": true
        })
    });

    Ok(json!({
        "success": true,
        "output": parsed
    }))
}

fn register_search(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.skillkit.search".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let query = require_str(&input, "query")?;

                let limit = input
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(20)
                    .to_string();

                let cache_key = format!("search::{}", query);
                let cached: Option<Value> = kv.get("config", &cache_key).await.map_err(kv_err)?;

                if let Some(cached_result) = cached {
                    return Ok(api_response(json!({
                        "results": cached_result,
                        "cached": true,
                        "query": query
                    })));
                }

                let result = run_skillkit_command(&["search", &query, "--limit", &limit]).await?;

                if let Err(e) = kv.set("config", &cache_key, &result).await {
                    tracing::warn!("Failed to cache skillkit result for {}: {}", cache_key, e);
                }

                Ok(api_response(json!({
                    "results": result,
                    "cached": false,
                    "query": query
                })))
            }
        },
    );
}

fn register_install(iii: &III, _kv: &StateKV) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.skillkit.install".to_string()),
        move |input: Value| async move {
            let input = extract_input(input);
            let skill_name = require_str(&input, "skill")?;

            let agent = input
                .get("agent")
                .and_then(|v| v.as_str())
                .unwrap_or("claude-code");

            let mut args = vec!["install", &skill_name];

            if agent != "claude-code" {
                args.push("--agent");
                args.push(agent);
            }

            let result = run_skillkit_command(&args).await?;

            Ok(api_response(json!({
                "skill": skill_name,
                "agent": agent,
                "result": result
            })))
        },
    );
}

fn register_translate(iii: &III, _kv: &StateKV) {
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.skillkit.translate".to_string()),
        move |input: Value| async move {
            let input = extract_input(input);
            let skill_name = require_str(&input, "skill")?;

            let target_agent = require_str(&input, "target_agent")?;

            let result =
                run_skillkit_command(&["translate", &skill_name, "--agent", &target_agent]).await?;

            Ok(api_response(json!({
                "skill": skill_name,
                "target_agent": target_agent,
                "result": result
            })))
        },
    );
}

fn register_recommend(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.skillkit.recommend".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let context = input
                    .get("context")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let agent = input
                    .get("agent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("claude-code")
                    .to_string();

                let limit = input
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(5)
                    .to_string();

                let mut args = vec!["recommend"];

                let has_context = !context.is_empty();
                if has_context {
                    args.push("--context");
                    args.push(&context);
                }

                args.push("--agent");
                args.push(&agent);
                args.push("--limit");
                args.push(&limit);

                let result = run_skillkit_command(&args).await?;

                let agents: Vec<crate::models::Agent> = kv.list("agents").await.map_err(kv_err)?;

                let active_agent_types: Vec<String> = agents
                    .iter()
                    .map(|a| format!("{:?}", a.agent_type).to_lowercase())
                    .collect();

                Ok(api_response(json!({
                    "recommendations": result,
                    "agent": agent,
                    "context": context,
                    "active_agents": active_agent_types
                })))
            }
        },
    );
}
