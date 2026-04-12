use chrono::{Datelike, Utc};
use iii_sdk::{III, IIIError, RegisterFunctionMessage, TriggerRequest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::sysutil::{api_response, extract_input, kv_err};
use crate::models::CostRecord;
use crate::state::StateKV;

const VALID_ACTIONS: &[&str] = &["alert", "block", "warn"];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BudgetAlert {
    timestamp: String,
    alert_type: String,
    message: String,
    monthly_spent: f64,
    daily_spent: f64,
    limit_hit: String,
}

pub fn register(iii: &III, kv: &StateKV) {
    register_check(iii, kv);
    register_status(iii, kv);
    register_set(iii, kv);
    register_alerts(iii, kv);
}

async fn get_config_f64(kv: &StateKV, key: &str, default: f64) -> Result<f64, IIIError> {
    let value: Option<Value> = kv.get("config", key).await.map_err(kv_err)?;
    Ok(value.and_then(|v| v.as_f64()).unwrap_or(default))
}

async fn get_config_str(kv: &StateKV, key: &str, default: &str) -> Result<String, IIIError> {
    let value: Option<Value> = kv.get("config", key).await.map_err(kv_err)?;
    Ok(value
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| default.to_string()))
}

async fn read_daily_cents(kv: &StateKV, date_key: &str) -> Result<i64, IIIError> {
    let value: Option<Value> = kv.get("cost_daily", date_key).await.map_err(kv_err)?;
    Ok(value
        .and_then(|v| v.get("total_cost_cents").and_then(|c| c.as_i64()))
        .unwrap_or(0))
}

async fn read_agent_daily_cents(
    kv: &StateKV,
    agent_id: Uuid,
    date_key: &str,
) -> Result<i64, IIIError> {
    let key = format!("{}::{}", agent_id, date_key);
    let value: Option<Value> = kv.get("cost_agent", &key).await.map_err(kv_err)?;
    Ok(value
        .and_then(|v| v.get("total_cost_cents").and_then(|c| c.as_i64()))
        .unwrap_or(0))
}

async fn compute_monthly_spent(kv: &StateKV) -> Result<f64, IIIError> {
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let last_day = days_in_month(year, month);

    let mut total_cents: i64 = 0;
    for day in 1..=last_day {
        let date_key = format!("{:04}-{:02}-{:02}", year, month, day);
        total_cents += read_daily_cents(kv, &date_key).await?;
    }
    Ok(total_cents as f64 / 100.0)
}

async fn compute_daily_spent(kv: &StateKV) -> Result<f64, IIIError> {
    let date_key = Utc::now().format("%Y-%m-%d").to_string();
    let cents = read_daily_cents(kv, &date_key).await?;
    Ok(cents as f64 / 100.0)
}

async fn compute_session_spent(kv: &StateKV, session_id: Uuid) -> Result<f64, IIIError> {
    let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;
    Ok(records
        .iter()
        .filter(|r| r.session_id == Some(session_id))
        .map(|r| r.total_cost)
        .sum())
}

async fn compute_agent_daily_spent(kv: &StateKV, agent_id: Uuid) -> Result<f64, IIIError> {
    let date_key = Utc::now().format("%Y-%m-%d").to_string();
    let cents = read_agent_daily_cents(kv, agent_id, &date_key).await?;
    Ok(cents as f64 / 100.0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(31)
}

fn register_check(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.budget.check".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);

                let monthly_limit = get_config_f64(&kv, "budget_monthly", 0.0).await?;
                let daily_limit = get_config_f64(&kv, "budget_daily", 0.0).await?;
                let session_limit = get_config_f64(&kv, "budget_session", 0.0).await?;
                let agent_daily_limit = get_config_f64(&kv, "budget_daily_agent", 0.0).await?;
                let alert_threshold = get_config_f64(&kv, "budget_alert_threshold", 0.8).await?;
                let action = get_config_str(&kv, "budget_action", "alert").await?;

                let pending_cost = input
                    .get("pending_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let monthly_spent = compute_monthly_spent(&kv).await? + pending_cost;
                let daily_spent = compute_daily_spent(&kv).await? + pending_cost;

                let session_spent: Option<f64> =
                    if let Some(sid) = input.get("session_id").and_then(|v| v.as_str()) {
                        let parsed = sid
                            .parse::<Uuid>()
                            .map_err(|e| IIIError::Handler(format!("invalid session_id: {}", e)))?;
                        Some(compute_session_spent(&kv, parsed).await? + pending_cost)
                    } else {
                        None
                    };

                let agent_daily_spent: Option<f64> =
                    if let Some(aid) = input.get("agent_id").and_then(|v| v.as_str()) {
                        let parsed = aid
                            .parse::<Uuid>()
                            .map_err(|e| IIIError::Handler(format!("invalid agent_id: {}", e)))?;
                        Some(compute_agent_daily_spent(&kv, parsed).await? + pending_cost)
                    } else {
                        None
                    };

                let mut status = "ok".to_string();
                let mut warnings: Vec<String> = Vec::new();
                let mut exceeded = false;
                let mut warning = false;

                let checks: [(&str, f64, Option<f64>); 4] = [
                    ("Monthly", monthly_limit, Some(monthly_spent)),
                    ("Daily", daily_limit, Some(daily_spent)),
                    ("Session", session_limit, session_spent),
                    ("Agent daily", agent_daily_limit, agent_daily_spent),
                ];

                for (label, limit, spent_opt) in checks {
                    if limit <= 0.0 {
                        continue;
                    }
                    let Some(spent) = spent_opt else {
                        continue;
                    };
                    if spent >= limit {
                        status = "exceeded".to_string();
                        exceeded = true;
                        warnings.push(format!(
                            "{} budget exceeded: ${:.2} / ${:.2}",
                            label, spent, limit
                        ));
                    } else if spent >= limit * alert_threshold {
                        if status != "exceeded" {
                            status = "warning".to_string();
                        }
                        warning = true;
                        warnings.push(format!(
                            "{} budget warning: ${:.2} / ${:.2} ({:.0}%)",
                            label,
                            spent,
                            limit,
                            spent / limit * 100.0
                        ));
                    }
                }

                if warning || exceeded {
                    let alert = BudgetAlert {
                        timestamp: Utc::now().to_rfc3339(),
                        alert_type: status.clone(),
                        message: warnings.join("; "),
                        monthly_spent,
                        daily_spent,
                        limit_hit: if exceeded {
                            "exceeded".to_string()
                        } else {
                            "threshold".to_string()
                        },
                    };

                    let alert_key = format!(
                        "alert_{}_{}",
                        Utc::now().timestamp_millis(),
                        Uuid::new_v4().simple()
                    );
                    if let Err(e) = kv.set("budget_alerts", &alert_key, &alert).await {
                        tracing::warn!("failed to persist budget alert: {}", e);
                    }

                    let event_type = if exceeded {
                        "budget.exceeded"
                    } else {
                        "budget.warning"
                    };
                    let _ = kv
                        .iii()
                        .trigger(TriggerRequest {
                            function_id: "rimuru.hooks.dispatch".to_string(),
                            payload: json!({
                                "event_type": event_type,
                                "payload": {
                                    "status": status,
                                    "monthly_spent": monthly_spent,
                                    "daily_spent": daily_spent,
                                    "session_spent": session_spent,
                                    "warnings": warnings
                                }
                            }),
                            action: None,
                            timeout_ms: Some(5000),
                        })
                        .await;
                }

                Ok(api_response(json!({
                    "status": status,
                    "exceeded": exceeded,
                    "warning": warning,
                    "action": action,
                    "monthly_spent": monthly_spent,
                    "daily_spent": daily_spent,
                    "session_spent": session_spent,
                    "agent_daily_spent": agent_daily_spent,
                    "warnings": warnings
                })))
            }
        },
    );
}

fn register_status(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.budget.status".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let monthly_limit = get_config_f64(&kv, "budget_monthly", 0.0).await?;
                let daily_limit = get_config_f64(&kv, "budget_daily", 0.0).await?;
                let session_limit = get_config_f64(&kv, "budget_session", 0.0).await?;
                let agent_daily_limit = get_config_f64(&kv, "budget_daily_agent", 0.0).await?;
                let alert_threshold = get_config_f64(&kv, "budget_alert_threshold", 0.8).await?;
                let action = get_config_str(&kv, "budget_action", "alert").await?;

                let monthly_spent = compute_monthly_spent(&kv).await?;
                let daily_spent = compute_daily_spent(&kv).await?;

                let now = Utc::now();
                let day_of_month = now.day() as f64;
                let days_this_month = days_in_month(now.year(), now.month()) as f64;
                let burn_rate_daily = if day_of_month > 0.0 {
                    monthly_spent / day_of_month
                } else {
                    0.0
                };
                let projected_monthly = burn_rate_daily * days_this_month;

                let monthly_remaining = if monthly_limit > 0.0 {
                    (monthly_limit - monthly_spent).max(0.0)
                } else {
                    -1.0
                };

                let daily_remaining = if daily_limit > 0.0 {
                    (daily_limit - daily_spent).max(0.0)
                } else {
                    -1.0
                };

                let status = if (monthly_limit > 0.0 && monthly_spent >= monthly_limit)
                    || (daily_limit > 0.0 && daily_spent >= daily_limit)
                {
                    "exceeded"
                } else if (monthly_limit > 0.0 && monthly_spent >= monthly_limit * alert_threshold)
                    || (daily_limit > 0.0 && daily_spent >= daily_limit * alert_threshold)
                {
                    "warning"
                } else {
                    "ok"
                };

                Ok(api_response(json!({
                    "monthly_limit": monthly_limit,
                    "monthly_spent": monthly_spent,
                    "monthly_remaining": monthly_remaining,
                    "daily_limit": daily_limit,
                    "daily_spent": daily_spent,
                    "daily_remaining": daily_remaining,
                    "session_limit": session_limit,
                    "agent_daily_limit": agent_daily_limit,
                    "alert_threshold": alert_threshold,
                    "action_on_exceed": action,
                    "status": status,
                    "status_scope": "global",
                    "burn_rate_daily": burn_rate_daily,
                    "projected_monthly": projected_monthly,
                    "days_in_month": days_this_month
                })))
            }
        },
    );
}

fn validate_limit(value: &Value, key: &str) -> Result<f64, IIIError> {
    let n = value
        .as_f64()
        .ok_or_else(|| IIIError::Handler(format!("{} must be a number", key)))?;
    if !n.is_finite() {
        return Err(IIIError::Handler(format!("{} must be finite", key)));
    }
    if n < 0.0 {
        return Err(IIIError::Handler(format!("{} must be >= 0", key)));
    }
    Ok(n)
}

fn validate_threshold(value: &Value) -> Result<f64, IIIError> {
    let n = value
        .as_f64()
        .ok_or_else(|| IIIError::Handler("alert_threshold must be a number".into()))?;
    if !(0.0..=1.0).contains(&n) {
        return Err(IIIError::Handler(
            "alert_threshold must be between 0.0 and 1.0".into(),
        ));
    }
    Ok(n)
}

fn validate_action(value: &Value) -> Result<String, IIIError> {
    let s = value
        .as_str()
        .ok_or_else(|| IIIError::Handler("action must be a string".into()))?;
    if !VALID_ACTIONS.contains(&s) {
        return Err(IIIError::Handler(format!(
            "action must be one of: {}",
            VALID_ACTIONS.join(", ")
        )));
    }
    Ok(s.to_string())
}

fn register_set(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.budget.set".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let mut updated: Vec<String> = Vec::new();

                let limit_keys = [
                    ("monthly_limit", "budget_monthly"),
                    ("daily_limit", "budget_daily"),
                    ("session_limit", "budget_session"),
                    ("daily_agent_limit", "budget_daily_agent"),
                ];

                for (input_key, config_key) in &limit_keys {
                    if let Some(val) = input.get(*input_key) {
                        let n = validate_limit(val, input_key)?;
                        kv.set("config", config_key, &json!(n))
                            .await
                            .map_err(kv_err)?;
                        updated.push(config_key.to_string());
                    }
                }

                if let Some(val) = input.get("alert_threshold") {
                    let n = validate_threshold(val)?;
                    kv.set("config", "budget_alert_threshold", &json!(n))
                        .await
                        .map_err(kv_err)?;
                    updated.push("budget_alert_threshold".to_string());
                }

                if let Some(val) = input.get("action") {
                    let s = validate_action(val)?;
                    kv.set("config", "budget_action", &json!(s))
                        .await
                        .map_err(kv_err)?;
                    updated.push("budget_action".to_string());
                }

                Ok(api_response(json!({
                    "updated": updated,
                    "count": updated.len()
                })))
            }
        },
    );
}

fn register_alerts(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.budget.alerts".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

                let mut alerts: Vec<BudgetAlert> =
                    kv.list("budget_alerts").await.map_err(kv_err)?;
                alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                let total = alerts.len();
                alerts.truncate(limit);

                Ok(api_response(json!({
                    "alerts": alerts,
                    "count": alerts.len(),
                    "total": total
                })))
            }
        },
    );
}
