use std::collections::HashMap;

use crate::state::AppState;
use rimuru_core::{AgentRepository, CostRepository, Repository};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CostSummaryResponse {
    pub total_cost: f64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub session_count: i64,
    pub period: String,
}

#[derive(Debug, Serialize)]
pub struct CostBreakdownItem {
    pub name: String,
    pub cost: f64,
    pub tokens: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct CostBreakdownResponse {
    pub by_agent: Vec<CostBreakdownItem>,
    pub by_model: Vec<CostBreakdownItem>,
}

#[derive(Debug, Serialize)]
pub struct CostHistoryPoint {
    pub date: String,
    pub cost: f64,
    pub tokens: i64,
}

#[derive(Debug, Deserialize)]
pub struct TimeRangeRequest {
    pub range: String,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[tauri::command]
pub async fn get_cost_summary(
    state: State<'_, AppState>,
    time_range: Option<TimeRangeRequest>,
) -> Result<CostSummaryResponse, String> {
    let repo = CostRepository::new(state.db.pool().clone());
    let records = repo.get_all().await.map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let range = time_range
        .as_ref()
        .map(|t| t.range.as_str())
        .unwrap_or("today");

    let filtered: Vec<_> = records
        .into_iter()
        .filter(|r| {
            let age = now.signed_duration_since(r.recorded_at);
            match range {
                "today" => age.num_hours() < 24,
                "week" => age.num_days() < 7,
                "month" => age.num_days() < 30,
                "year" => age.num_days() < 365,
                _ => true,
            }
        })
        .collect();

    let total_cost: f64 = filtered.iter().map(|r| r.cost_usd).sum();
    let input_tokens: i64 = filtered.iter().map(|r| r.input_tokens).sum();
    let output_tokens: i64 = filtered.iter().map(|r| r.output_tokens).sum();
    let total_tokens = input_tokens + output_tokens;

    Ok(CostSummaryResponse {
        total_cost,
        total_tokens,
        input_tokens,
        output_tokens,
        session_count: filtered.len() as i64,
        period: range.to_string(),
    })
}

#[tauri::command]
pub async fn get_cost_breakdown(
    state: State<'_, AppState>,
    time_range: Option<TimeRangeRequest>,
) -> Result<CostBreakdownResponse, String> {
    let repo = CostRepository::new(state.db.pool().clone());
    let records = repo.get_all().await.map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let range = time_range
        .as_ref()
        .map(|t| t.range.as_str())
        .unwrap_or("month");

    let filtered: Vec<_> = records
        .into_iter()
        .filter(|r| {
            let age = now.signed_duration_since(r.recorded_at);
            match range {
                "today" => age.num_hours() < 24,
                "week" => age.num_days() < 7,
                "month" => age.num_days() < 30,
                "year" => age.num_days() < 365,
                _ => true,
            }
        })
        .collect();

    let total_cost: f64 = filtered.iter().map(|r| r.cost_usd).sum();

    let mut by_model: HashMap<String, (f64, i64)> = HashMap::new();
    for record in &filtered {
        let entry = by_model
            .entry(record.model_name.clone())
            .or_insert((0.0, 0));
        entry.0 += record.cost_usd;
        entry.1 += record.input_tokens + record.output_tokens;
    }

    let percentage = |cost: f64| -> f64 {
        if total_cost > 0.0 {
            (cost / total_cost) * 100.0
        } else {
            0.0
        }
    };

    let by_model_items: Vec<CostBreakdownItem> = by_model
        .into_iter()
        .map(|(name, (cost, tokens))| CostBreakdownItem {
            percentage: percentage(cost),
            name,
            cost,
            tokens,
        })
        .collect();

    let agent_repo = AgentRepository::new(state.db.pool().clone());
    let mut by_agent_map: HashMap<uuid::Uuid, (f64, i64)> = HashMap::new();
    for record in &filtered {
        let entry = by_agent_map.entry(record.agent_id).or_insert((0.0, 0));
        entry.0 += record.cost_usd;
        entry.1 += record.input_tokens + record.output_tokens;
    }

    let mut by_agent_items: Vec<CostBreakdownItem> = Vec::new();
    for (agent_id, (cost, tokens)) in by_agent_map {
        let name = agent_repo
            .get_by_id(agent_id)
            .await
            .ok()
            .flatten()
            .map(|a| a.name)
            .unwrap_or_else(|| agent_id.to_string());
        by_agent_items.push(CostBreakdownItem {
            percentage: percentage(cost),
            name,
            cost,
            tokens,
        });
    }

    Ok(CostBreakdownResponse {
        by_agent: by_agent_items,
        by_model: by_model_items,
    })
}

#[tauri::command]
pub async fn get_cost_history(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<CostHistoryPoint>, String> {
    let repo = CostRepository::new(state.db.pool().clone());
    let records = repo.get_all().await.map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let num_days = days.unwrap_or(30);

    let filtered: Vec<_> = records
        .into_iter()
        .filter(|r| {
            let age = now.signed_duration_since(r.recorded_at);
            age.num_days() < num_days
        })
        .collect();

    let mut by_date: HashMap<String, (f64, i64)> = HashMap::new();
    for record in filtered {
        let date = record.recorded_at.format("%Y-%m-%d").to_string();
        let entry = by_date.entry(date).or_insert((0.0, 0));
        entry.0 += record.cost_usd;
        entry.1 += record.input_tokens + record.output_tokens;
    }

    let mut history: Vec<CostHistoryPoint> = by_date
        .into_iter()
        .map(|(date, (cost, tokens))| CostHistoryPoint { date, cost, tokens })
        .collect();

    history.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(history)
}
