use iii_sdk::{III, RegisterFunctionMessage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::models::{ContextBreakdown, Session, SessionStatus, TurnRecord};
use crate::state::StateKV;

#[derive(Debug, Serialize, Deserialize)]
struct RunawayPattern {
    pattern_type: String,
    description: String,
    severity: f64,
    metadata: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RunawayAnalysis {
    session_id: Uuid,
    is_runaway: bool,
    severity: f64,
    patterns: Vec<RunawayPattern>,
    tokens_burned: u64,
    estimated_cost_wasted: f64,
    recommendation: String,
}

pub fn register(iii: &III, kv: &StateKV) {
    register_analyze(iii, kv);
    register_scan(iii, kv);
    register_configure(iii, kv);
}

struct RunawayConfig {
    repeat_threshold: u32,
    token_explosion_ratio: f64,
}

impl Default for RunawayConfig {
    fn default() -> Self {
        Self {
            repeat_threshold: 3,
            token_explosion_ratio: 2.0,
        }
    }
}

async fn load_runaway_config(kv: &StateKV) -> RunawayConfig {
    let mut cfg = RunawayConfig::default();

    if let Ok(Some(v)) = kv.get::<Value>("config", "runaway_repeat_threshold").await
        && let Some(n) = v.as_u64()
    {
        cfg.repeat_threshold = n as u32;
    }

    if let Ok(Some(v)) = kv
        .get::<Value>("config", "runaway_token_explosion_ratio")
        .await
        && let Some(n) = v.as_f64()
    {
        cfg.token_explosion_ratio = n;
    }

    cfg
}

async fn load_runaway_window(kv: &StateKV) -> usize {
    if let Ok(Some(v)) = kv.get::<Value>("config", "runaway_window").await
        && let Some(n) = v.as_u64()
    {
        return n as usize;
    }
    10
}

fn analyze_turns(session_id: Uuid, turns: &[TurnRecord], cfg: &RunawayConfig) -> RunawayAnalysis {
    let mut patterns: Vec<RunawayPattern> = Vec::new();

    detect_repeated_calls(turns, cfg.repeat_threshold, &mut patterns);
    detect_repeated_errors(turns, cfg.repeat_threshold, &mut patterns);
    detect_token_explosion(turns, cfg.token_explosion_ratio, &mut patterns);
    detect_oscillation(turns, &mut patterns);

    let severity = patterns.iter().map(|p| p.severity).fold(0.0_f64, f64::max);
    let is_runaway = severity >= 0.5;

    let tokens_burned: u64 = if is_runaway {
        turns.iter().map(|t| t.input_tokens + t.output_tokens).sum()
    } else {
        0
    };

    let estimated_cost_wasted = tokens_burned as f64 / 1_000_000.0 * 9.0;

    let recommendation = if is_runaway {
        if severity >= 0.8 {
            "Session is severely stuck. Stop immediately and re-prompt with a different approach."
                .to_string()
        } else {
            "Session appears stuck. Consider stopping and re-prompting.".to_string()
        }
    } else {
        "No runaway patterns detected.".to_string()
    };

    RunawayAnalysis {
        session_id,
        is_runaway,
        severity,
        patterns,
        tokens_burned,
        estimated_cost_wasted,
        recommendation,
    }
}

fn detect_repeated_calls(turns: &[TurnRecord], threshold: u32, patterns: &mut Vec<RunawayPattern>) {
    if turns.len() < 3 {
        return;
    }

    let mut max_streak = 1u32;
    let mut streak = 1u32;
    let mut streak_tool = String::new();

    for window in turns.windows(2) {
        let prev_tools: Vec<&str> = window[0]
            .tool_calls
            .iter()
            .map(|t| t.tool_name.as_str())
            .collect();
        let curr_tools: Vec<&str> = window[1]
            .tool_calls
            .iter()
            .map(|t| t.tool_name.as_str())
            .collect();

        if !prev_tools.is_empty() && prev_tools == curr_tools {
            streak += 1;
            if streak > max_streak {
                max_streak = streak;
                streak_tool = prev_tools.join(", ");
            }
        } else {
            streak = 1;
        }
    }

    if max_streak > threshold {
        let severity = (max_streak as f64 / 10.0).min(1.0);
        patterns.push(RunawayPattern {
            pattern_type: "repeated_calls".to_string(),
            description: format!(
                "Tool '{}' called {} times consecutively",
                streak_tool, max_streak
            ),
            severity,
            metadata: json!({"tool": streak_tool, "count": max_streak}),
        });
    }
}

fn detect_repeated_errors(
    turns: &[TurnRecord],
    threshold: u32,
    patterns: &mut Vec<RunawayPattern>,
) {
    let mut max_streak = 0u32;
    let mut streak = 0u32;
    let mut last_signature: Option<String> = None;
    let mut max_signature: Option<String> = None;

    for turn in turns {
        if turn.content_type.contains("error") {
            let signature = format!("{}::{}", turn.content_type, turn.role);
            if last_signature.as_ref() == Some(&signature) {
                streak += 1;
            } else {
                streak = 1;
                last_signature = Some(signature.clone());
            }
            if streak > max_streak {
                max_streak = streak;
                max_signature = Some(signature);
            }
        } else {
            streak = 0;
            last_signature = None;
        }
    }

    if max_streak > threshold {
        let severity = (max_streak as f64 / 8.0).min(1.0);
        patterns.push(RunawayPattern {
            pattern_type: "repeated_errors".to_string(),
            description: format!("{} consecutive identical error turns detected", max_streak),
            severity,
            metadata: json!({
                "count": max_streak,
                "signature": max_signature
            }),
        });
    }
}

fn detect_token_explosion(
    turns: &[TurnRecord],
    explosion_ratio: f64,
    patterns: &mut Vec<RunawayPattern>,
) {
    if turns.len() < 4 {
        return;
    }

    let baseline_end = turns.len() - 3;
    let baseline = &turns[..baseline_end];
    if baseline.is_empty() {
        return;
    }

    let baseline_total: u64 = baseline.iter().map(|t| t.input_tokens).sum();
    let baseline_avg = baseline_total as f64 / baseline.len() as f64;

    if baseline_avg == 0.0 {
        return;
    }

    let last_3 = &turns[baseline_end..];
    let all_exploded = last_3
        .iter()
        .all(|t| t.input_tokens as f64 > baseline_avg * explosion_ratio);

    if all_exploded {
        let last_3_avg: f64 = last_3.iter().map(|t| t.input_tokens as f64).sum::<f64>() / 3.0;
        let ratio = last_3_avg / baseline_avg;
        let severity = ((ratio - 1.0) / 4.0).min(1.0);

        patterns.push(RunawayPattern {
            pattern_type: "token_explosion".to_string(),
            description: format!(
                "Last 3 turns use {:.1}x baseline input tokens ({:.0} vs {:.0} baseline avg)",
                ratio, last_3_avg, baseline_avg
            ),
            severity,
            metadata: json!({
                "ratio": ratio,
                "last_3_avg": last_3_avg,
                "baseline_avg": baseline_avg
            }),
        });
    }
}

fn detect_oscillation(turns: &[TurnRecord], patterns: &mut Vec<RunawayPattern>) {
    if turns.len() < 4 {
        return;
    }

    let tool_sequence: Vec<String> = turns
        .iter()
        .filter_map(|t| t.tool_calls.first().map(|tc| tc.tool_name.clone()))
        .collect();

    if tool_sequence.len() < 4 {
        return;
    }

    let mut max_oscillation = 0u32;

    for i in 0..tool_sequence.len().saturating_sub(3) {
        let a = &tool_sequence[i];
        let b = &tool_sequence[i + 1];
        if a == b {
            continue;
        }

        let mut count = 2u32;
        for (offset, tool) in tool_sequence.iter().enumerate().skip(i + 2) {
            let expected = if (offset - i) % 2 == 0 { a } else { b };
            if tool == expected {
                count += 1;
            } else {
                break;
            }
        }

        if count > max_oscillation {
            max_oscillation = count;
        }
    }

    if max_oscillation > 4 {
        let severity = (max_oscillation as f64 / 8.0).min(1.0);
        patterns.push(RunawayPattern {
            pattern_type: "oscillation".to_string(),
            description: format!("Tools oscillating back and forth {} times", max_oscillation),
            severity,
            metadata: json!({"count": max_oscillation}),
        });
    }
}

fn register_analyze(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.runaway.analyze".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let session_id_str = require_str(&input, "session_id")?;

                let configured_window = load_runaway_window(&kv).await;
                let window = input
                    .get("window")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
                    .unwrap_or(configured_window);

                let cfg = load_runaway_config(&kv).await;

                let session_id = session_id_str.parse::<Uuid>().map_err(|e| {
                    iii_sdk::IIIError::Handler(format!("invalid session_id: {}", e))
                })?;

                let breakdown: Option<ContextBreakdown> = kv
                    .get("context_breakdowns", &session_id_str)
                    .await
                    .map_err(kv_err)?;

                let turns = match breakdown {
                    Some(b) => b.turns,
                    None => {
                        return Ok(api_response(json!({
                            "session_id": session_id,
                            "is_runaway": false,
                            "severity": 0.0,
                            "patterns": [],
                            "tokens_burned": 0,
                            "estimated_cost_wasted": 0.0,
                            "recommendation": "No context breakdown available for analysis."
                        })));
                    }
                };

                let windowed = if turns.len() > window {
                    &turns[turns.len() - window..]
                } else {
                    &turns
                };

                let analysis = analyze_turns(session_id, windowed, &cfg);
                Ok(api_response(
                    serde_json::to_value(analysis).unwrap_or_default(),
                ))
            }
        },
    );
}

fn register_scan(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.runaway.scan".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);

                let configured_window = load_runaway_window(&kv).await;
                let window = input
                    .get("window")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as usize)
                    .unwrap_or(configured_window);

                let cfg = load_runaway_config(&kv).await;

                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;
                let active: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| matches!(s.status, SessionStatus::Active))
                    .collect();

                let mut flagged: Vec<RunawayAnalysis> = Vec::new();

                for session in &active {
                    let sid = session.id.to_string();
                    let breakdown: Option<ContextBreakdown> =
                        kv.get("context_breakdowns", &sid).await.map_err(kv_err)?;

                    if let Some(b) = breakdown {
                        let turns = if b.turns.len() > window {
                            &b.turns[b.turns.len() - window..]
                        } else {
                            &b.turns
                        };

                        let analysis = analyze_turns(session.id, turns, &cfg);
                        if analysis.is_runaway {
                            flagged.push(analysis);
                        }
                    }
                }

                flagged.sort_by(|a, b| {
                    b.severity
                        .partial_cmp(&a.severity)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                Ok(api_response(json!({
                    "flagged": flagged,
                    "total_active_sessions": active.len(),
                    "total_flagged": flagged.len()
                })))
            }
        },
    );
}

fn register_configure(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.runaway.configure".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);

                let config_keys = [
                    ("window", "runaway_window"),
                    ("repeat_threshold", "runaway_repeat_threshold"),
                    ("token_explosion_ratio", "runaway_token_explosion_ratio"),
                    ("auto_scan_enabled", "runaway_auto_scan_enabled"),
                ];

                let has_updates = config_keys
                    .iter()
                    .any(|(input_key, _)| input.get(*input_key).is_some());

                if has_updates {
                    let mut updated: Vec<String> = Vec::new();
                    for (input_key, config_key) in &config_keys {
                        let Some(val) = input.get(*input_key) else {
                            continue;
                        };
                        match *input_key {
                            "window" | "repeat_threshold" => {
                                let n = val.as_u64().ok_or_else(|| {
                                    iii_sdk::IIIError::Handler(format!(
                                        "{} must be a positive integer",
                                        input_key
                                    ))
                                })?;
                                if n == 0 {
                                    return Err(iii_sdk::IIIError::Handler(format!(
                                        "{} must be > 0",
                                        input_key
                                    )));
                                }
                            }
                            "token_explosion_ratio" => {
                                let n = val.as_f64().ok_or_else(|| {
                                    iii_sdk::IIIError::Handler(
                                        "token_explosion_ratio must be a number".into(),
                                    )
                                })?;
                                if !n.is_finite() || n <= 1.0 {
                                    return Err(iii_sdk::IIIError::Handler(
                                        "token_explosion_ratio must be > 1.0".into(),
                                    ));
                                }
                            }
                            "auto_scan_enabled" if !val.is_boolean() => {
                                return Err(iii_sdk::IIIError::Handler(
                                    "auto_scan_enabled must be a boolean".into(),
                                ));
                            }
                            _ => {}
                        }
                        kv.set("config", config_key, val).await.map_err(kv_err)?;
                        updated.push(config_key.to_string());
                    }
                    Ok(api_response(json!({
                        "updated": updated,
                        "count": updated.len()
                    })))
                } else {
                    let defaults = json!({
                        "runaway_window": 10,
                        "runaway_repeat_threshold": 3,
                        "runaway_token_explosion_ratio": 2.0,
                        "runaway_auto_scan_enabled": false
                    });

                    let mut config = serde_json::Map::new();
                    for (key, default_val) in defaults.as_object().unwrap() {
                        let stored: Option<Value> = kv.get("config", key).await.map_err(kv_err)?;
                        config.insert(key.clone(), stored.unwrap_or_else(|| default_val.clone()));
                    }

                    Ok(api_response(json!({"config": config})))
                }
            }
        },
    );
}
