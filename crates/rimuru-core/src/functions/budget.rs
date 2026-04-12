use chrono::{Datelike, Utc};
use iii_sdk::{III, RegisterFunctionMessage, TriggerRequest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::sysutil::{api_response, extract_input, kv_err};
use crate::models::CostRecord;
use crate::state::StateKV;

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

async fn get_config_f64(kv: &StateKV, key: &str, default: f64) -> f64 {
    kv.get::<Value>("config", key)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_f64())
        .unwrap_or(default)
}

async fn get_config_str(kv: &StateKV, key: &str, default: &str) -> String {
    kv.get::<Value>("config", key)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| default.to_string())
}

async fn compute_monthly_spent(kv: &StateKV) -> f64 {
    let now = Utc::now();
    let records: Vec<CostRecord> = kv.list("cost_records").await.unwrap_or_default();
    records
        .iter()
        .filter(|r| r.recorded_at.year() == now.year() && r.recorded_at.month() == now.month())
        .map(|r| r.total_cost)
        .sum()
}

async fn compute_daily_spent(kv: &StateKV) -> f64 {
    let today = Utc::now().date_naive();
    let records: Vec<CostRecord> = kv.list("cost_records").await.unwrap_or_default();
    records
        .iter()
        .filter(|r| r.recorded_at.date_naive() == today)
        .map(|r| r.total_cost)
        .sum()
}

fn register_check(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.budget.check".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);

                let monthly_limit = get_config_f64(&kv, "budget_monthly", 0.0).await;
                let daily_limit = get_config_f64(&kv, "budget_daily", 0.0).await;
                let session_limit = get_config_f64(&kv, "budget_session", 0.0).await;
                let alert_threshold = get_config_f64(&kv, "budget_alert_threshold", 0.8).await;
                let action = get_config_str(&kv, "budget_action", "alert").await;

                let monthly_spent = compute_monthly_spent(&kv).await;
                let daily_spent = compute_daily_spent(&kv).await;
                let session_cost = input
                    .get("session_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let mut status = "ok".to_string();
                let mut warnings: Vec<String> = Vec::new();
                let mut exceeded = false;
                let mut warning = false;

                if monthly_limit > 0.0 {
                    if monthly_spent >= monthly_limit {
                        status = "exceeded".to_string();
                        exceeded = true;
                        warnings.push(format!(
                            "Monthly budget exceeded: ${:.2} / ${:.2}",
                            monthly_spent, monthly_limit
                        ));
                    } else if monthly_spent >= monthly_limit * alert_threshold {
                        if status != "exceeded" {
                            status = "warning".to_string();
                        }
                        warning = true;
                        warnings.push(format!(
                            "Monthly budget warning: ${:.2} / ${:.2} ({:.0}%)",
                            monthly_spent,
                            monthly_limit,
                            monthly_spent / monthly_limit * 100.0
                        ));
                    }
                }

                if daily_limit > 0.0 {
                    if daily_spent >= daily_limit {
                        status = "exceeded".to_string();
                        exceeded = true;
                        warnings.push(format!(
                            "Daily budget exceeded: ${:.2} / ${:.2}",
                            daily_spent, daily_limit
                        ));
                    } else if daily_spent >= daily_limit * alert_threshold {
                        if status != "exceeded" {
                            status = "warning".to_string();
                        }
                        warning = true;
                        warnings.push(format!(
                            "Daily budget warning: ${:.2} / ${:.2} ({:.0}%)",
                            daily_spent,
                            daily_limit,
                            daily_spent / daily_limit * 100.0
                        ));
                    }
                }

                if session_limit > 0.0 && session_cost >= session_limit {
                    status = "exceeded".to_string();
                    exceeded = true;
                    warnings.push(format!(
                        "Session budget exceeded: ${:.2} / ${:.2}",
                        session_cost, session_limit
                    ));
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

                    let alert_key = format!("alert_{}", Utc::now().timestamp_millis());
                    let _ = kv.set("budget_alerts", &alert_key, &alert).await;

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
                let monthly_limit = get_config_f64(&kv, "budget_monthly", 0.0).await;
                let daily_limit = get_config_f64(&kv, "budget_daily", 0.0).await;
                let session_limit = get_config_f64(&kv, "budget_session", 0.0).await;
                let alert_threshold = get_config_f64(&kv, "budget_alert_threshold", 0.8).await;
                let action = get_config_str(&kv, "budget_action", "alert").await;

                let monthly_spent = compute_monthly_spent(&kv).await;
                let daily_spent = compute_daily_spent(&kv).await;

                let now = Utc::now();
                let day_of_month = now.day() as f64;
                let burn_rate_daily = if day_of_month > 0.0 {
                    monthly_spent / day_of_month
                } else {
                    0.0
                };
                let projected_monthly = burn_rate_daily * 30.0;

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
                    "alert_threshold": alert_threshold,
                    "action_on_exceed": action,
                    "status": status,
                    "burn_rate_daily": burn_rate_daily,
                    "projected_monthly": projected_monthly
                })))
            }
        },
    );
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

                let mappings = [
                    ("monthly_limit", "budget_monthly"),
                    ("daily_limit", "budget_daily"),
                    ("session_limit", "budget_session"),
                    ("alert_threshold", "budget_alert_threshold"),
                ];

                for (input_key, config_key) in &mappings {
                    if let Some(val) = input.get(*input_key) {
                        kv.set("config", config_key, val).await.map_err(kv_err)?;
                        updated.push(config_key.to_string());
                    }
                }

                if let Some(val) = input.get("action") {
                    kv.set("config", "budget_action", val)
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
                alerts.truncate(limit);

                Ok(api_response(json!({
                    "alerts": alerts,
                    "total": alerts.len()
                })))
            }
        },
    );
}
