use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::models::{
    Agent, AgentCostSummary, AgentType, CostRecord, CostSummary, DailyCostSummary, ModelCostSummary,
};
use crate::state::StateKV;

use super::sysutil::{kv_err, require_str};

struct AgentAccum {
    agent_type: AgentType,
    cost: f64,
    input_tokens: u64,
    output_tokens: u64,
    count: u64,
}

struct ModelAccum {
    cost: f64,
    tokens: u64,
    count: u64,
}

struct DailyAccum {
    cost: f64,
    input_tokens: u64,
    output_tokens: u64,
    count: u64,
}

struct ModelBreakdownAccum {
    cost: f64,
    count: u64,
}

pub fn register(iii: &III, kv: &StateKV) {
    register_record(iii, kv);
    register_summary(iii, kv);
    register_daily(iii, kv);
    register_by_agent(iii, kv);
    register_daily_rollup(iii, kv);
}

fn register_record(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.record".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let agent_id_str = require_str(&input, "agent_id")?;

                let agent_id = Uuid::parse_str(&agent_id_str)
                    .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid agent_id: {}", e)))?;

                let agent_type: AgentType =
                    serde_json::from_value(input.get("agent_type").cloned().ok_or_else(|| {
                        iii_sdk::IIIError::Handler("agent_type is required".into())
                    })?)
                    .map_err(|e| {
                        iii_sdk::IIIError::Handler(format!("invalid agent_type: {}", e))
                    })?;

                let model = input
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let provider = input
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let input_tokens = input
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let output_tokens = input
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let input_cost = input
                    .get("input_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let output_cost = input
                    .get("output_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let mut record = CostRecord::new(
                    agent_id,
                    agent_type,
                    model,
                    provider,
                    input_tokens,
                    output_tokens,
                    input_cost,
                    output_cost,
                );

                if let Some(session_id) = input.get("session_id").and_then(|v| v.as_str()) {
                    record.session_id = Uuid::parse_str(session_id).ok();
                }

                if let Some(cache_read) = input.get("cache_read_tokens").and_then(|v| v.as_u64()) {
                    record.cache_read_tokens = cache_read;
                }

                if let Some(cache_write) = input.get("cache_write_tokens").and_then(|v| v.as_u64())
                {
                    record.cache_write_tokens = cache_write;
                }

                let record_id = record.id.to_string();
                kv.set("cost_records", &record_id, &record)
                    .await
                    .map_err(kv_err)?;

                let today = Utc::now().format("%Y-%m-%d").to_string();
                kv.increment(
                    "cost_daily",
                    &today,
                    "total_cost_cents",
                    (record.total_cost * 100.0) as i64,
                )
                .await
                .map_err(kv_err)?;
                kv.increment("cost_daily", &today, "record_count", 1)
                    .await
                    .map_err(kv_err)?;

                let agent_cost_key = format!("{}::{}", agent_id, today);
                kv.increment(
                    "cost_agent",
                    &agent_cost_key,
                    "total_cost_cents",
                    (record.total_cost * 100.0) as i64,
                )
                .await
                .map_err(kv_err)?;
                kv.increment(
                    "cost_agent",
                    &agent_cost_key,
                    "input_tokens",
                    input_tokens as i64,
                )
                .await
                .map_err(kv_err)?;
                kv.increment(
                    "cost_agent",
                    &agent_cost_key,
                    "output_tokens",
                    output_tokens as i64,
                )
                .await
                .map_err(kv_err)?;

                Ok(json!({
                    "record": record,
                    "recorded": true
                }))
            }
        },
    );
}

fn register_summary(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.summary".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let since = input
                    .get("since")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                let until = input
                    .get("until")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                let filtered: Vec<&CostRecord> = records
                    .iter()
                    .filter(|r| since.is_none_or(|s| r.recorded_at >= s))
                    .filter(|r| until.is_none_or(|u| r.recorded_at <= u))
                    .collect();

                let total_cost: f64 = filtered.iter().map(|r| r.total_cost).sum();
                let total_input: u64 = filtered.iter().map(|r| r.input_tokens).sum();
                let total_output: u64 = filtered.iter().map(|r| r.output_tokens).sum();

                let mut agent_map: HashMap<Uuid, AgentAccum> = HashMap::new();
                for r in &filtered {
                    let entry = agent_map.entry(r.agent_id).or_insert(AgentAccum {
                        agent_type: r.agent_type,
                        cost: 0.0,
                        input_tokens: 0,
                        output_tokens: 0,
                        count: 0,
                    });
                    entry.cost += r.total_cost;
                    entry.input_tokens += r.input_tokens;
                    entry.output_tokens += r.output_tokens;
                    entry.count += 1;
                }

                let agents: Vec<Agent> = kv.list("agents").await.map_err(kv_err)?;

                let by_agent: Vec<AgentCostSummary> = agent_map
                    .iter()
                    .map(|(id, accum)| {
                        let name = agents
                            .iter()
                            .find(|a| a.id == *id)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| id.to_string());

                        AgentCostSummary {
                            agent_id: *id,
                            agent_type: accum.agent_type,
                            agent_name: name,
                            total_cost: accum.cost,
                            total_input_tokens: accum.input_tokens,
                            total_output_tokens: accum.output_tokens,
                            record_count: accum.count,
                        }
                    })
                    .collect();

                let mut model_map: HashMap<(String, String), ModelAccum> = HashMap::new();
                for r in &filtered {
                    let entry = model_map
                        .entry((r.model.clone(), r.provider.clone()))
                        .or_insert(ModelAccum {
                            cost: 0.0,
                            tokens: 0,
                            count: 0,
                        });
                    entry.cost += r.total_cost;
                    entry.tokens += r.input_tokens + r.output_tokens;
                    entry.count += 1;
                }

                let by_model: Vec<ModelCostSummary> = model_map
                    .iter()
                    .map(|((model, provider), accum)| ModelCostSummary {
                        model: model.clone(),
                        provider: provider.clone(),
                        total_cost: accum.cost,
                        total_tokens: accum.tokens,
                        record_count: accum.count,
                    })
                    .collect();

                let summary = CostSummary {
                    total_cost,
                    total_input_tokens: total_input,
                    total_output_tokens: total_output,
                    total_records: filtered.len() as u64,
                    by_agent,
                    by_model,
                    period_start: since,
                    period_end: until,
                };

                Ok(json!({"summary": summary}))
            }
        },
    );
}

fn register_daily(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.daily".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let days = input.get("days").and_then(|v| v.as_u64()).unwrap_or(30);

                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let cutoff = Utc::now() - chrono::Duration::days(days as i64);

                let filtered: Vec<&CostRecord> =
                    records.iter().filter(|r| r.recorded_at >= cutoff).collect();

                let mut daily_map: HashMap<NaiveDate, DailyAccum> = HashMap::new();

                for r in &filtered {
                    let date = r.recorded_at.date_naive();
                    let entry = daily_map.entry(date).or_insert(DailyAccum {
                        cost: 0.0,
                        input_tokens: 0,
                        output_tokens: 0,
                        count: 0,
                    });
                    entry.cost += r.total_cost;
                    entry.input_tokens += r.input_tokens;
                    entry.output_tokens += r.output_tokens;
                    entry.count += 1;
                }

                let mut daily: Vec<Value> = daily_map
                    .iter()
                    .map(|(date, accum)| {
                        json!({
                            "date": date.to_string(),
                            "total_cost": accum.cost,
                            "input_tokens": accum.input_tokens,
                            "output_tokens": accum.output_tokens,
                            "record_count": accum.count
                        })
                    })
                    .collect();

                daily.sort_by(|a, b| {
                    let da = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    let db = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
                    da.cmp(db)
                });

                let total_cost: f64 = filtered.iter().map(|r| r.total_cost).sum();

                Ok(json!({
                    "daily": daily,
                    "total_cost": total_cost,
                    "days": days,
                    "total_days_with_usage": daily.len()
                }))
            }
        },
    );
}

fn register_by_agent(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.by_agent".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let agent_id_str = require_str(&input, "agent_id")?;

                let agent_id = Uuid::parse_str(&agent_id_str)
                    .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid agent_id: {}", e)))?;

                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let days = input.get("days").and_then(|v| v.as_u64()).unwrap_or(30);

                let cutoff = Utc::now() - chrono::Duration::days(days as i64);

                let filtered: Vec<&CostRecord> = records
                    .iter()
                    .filter(|r| r.agent_id == agent_id)
                    .filter(|r| r.recorded_at >= cutoff)
                    .collect();

                let total_cost: f64 = filtered.iter().map(|r| r.total_cost).sum();
                let total_input: u64 = filtered.iter().map(|r| r.input_tokens).sum();
                let total_output: u64 = filtered.iter().map(|r| r.output_tokens).sum();

                let mut model_breakdown: HashMap<String, ModelBreakdownAccum> = HashMap::new();
                for r in &filtered {
                    let entry =
                        model_breakdown
                            .entry(r.model.clone())
                            .or_insert(ModelBreakdownAccum {
                                cost: 0.0,
                                count: 0,
                            });
                    entry.cost += r.total_cost;
                    entry.count += 1;
                }

                let models: Vec<Value> = model_breakdown
                    .iter()
                    .map(|(model, accum)| {
                        json!({
                            "model": model,
                            "total_cost": accum.cost,
                            "record_count": accum.count
                        })
                    })
                    .collect();

                Ok(json!({
                    "agent_id": agent_id,
                    "total_cost": total_cost,
                    "total_input_tokens": total_input,
                    "total_output_tokens": total_output,
                    "total_records": filtered.len(),
                    "by_model": models,
                    "days": days
                }))
            }
        },
    );
}

fn register_daily_rollup(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.daily_rollup".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let date_str = input.get("date").and_then(|v| v.as_str()).unwrap_or("");

                let target_date = if date_str.is_empty() {
                    Utc::now().date_naive()
                } else {
                    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .map_err(|e| iii_sdk::IIIError::Handler(format!("invalid date: {}", e)))?
                };

                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let day_records: Vec<&CostRecord> = records
                    .iter()
                    .filter(|r| r.recorded_at.date_naive() == target_date)
                    .collect();

                let total_cost: f64 = day_records.iter().map(|r| r.total_cost).sum();
                let total_input: u64 = day_records.iter().map(|r| r.input_tokens).sum();
                let total_output: u64 = day_records.iter().map(|r| r.output_tokens).sum();

                let mut agent_map: HashMap<Uuid, AgentAccum> = HashMap::new();
                for r in &day_records {
                    let entry = agent_map.entry(r.agent_id).or_insert(AgentAccum {
                        agent_type: r.agent_type,
                        cost: 0.0,
                        input_tokens: 0,
                        output_tokens: 0,
                        count: 0,
                    });
                    entry.cost += r.total_cost;
                    entry.input_tokens += r.input_tokens;
                    entry.output_tokens += r.output_tokens;
                    entry.count += 1;
                }

                let agents: Vec<Agent> = kv.list("agents").await.map_err(kv_err)?;

                let by_agent: Vec<AgentCostSummary> = agent_map
                    .iter()
                    .map(|(id, accum)| {
                        let name = agents
                            .iter()
                            .find(|a| a.id == *id)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| id.to_string());

                        AgentCostSummary {
                            agent_id: *id,
                            agent_type: accum.agent_type,
                            agent_name: name,
                            total_cost: accum.cost,
                            total_input_tokens: accum.input_tokens,
                            total_output_tokens: accum.output_tokens,
                            record_count: accum.count,
                        }
                    })
                    .collect();

                let rollup = DailyCostSummary {
                    date: target_date,
                    total_cost,
                    total_input_tokens: total_input,
                    total_output_tokens: total_output,
                    record_count: day_records.len() as u64,
                    by_agent,
                };

                kv.set("cost_daily", &target_date.to_string(), &rollup)
                    .await
                    .map_err(kv_err)?;

                Ok(json!({
                    "rollup": rollup,
                    "persisted": true
                }))
            }
        },
    );
}
