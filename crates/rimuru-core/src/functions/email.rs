#![cfg(feature = "email")]

use chrono::{Duration, Utc};
use iii_sdk::{III, IIIError, RegisterFunctionMessage};
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde_json::{Value, json};

use super::sysutil::{api_response, kv_err};
use crate::models::CostRecord;
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.email.digest".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let enabled = get_bool(&kv, "email.enabled", false).await;
                if !enabled {
                    return Ok(api_response(json!({
                        "sent": false,
                        "reason": "email disabled"
                    })));
                }

                let host = get_str(&kv, "email.smtp_host", "").await;
                let port = get_u64(&kv, "email.port", 587).await as u16;
                let username = get_str(&kv, "email.username", "").await;
                let password = get_str(&kv, "email.password", "").await;
                let from = get_str(&kv, "email.from", "").await;
                let to_list = get_list(&kv, "email.to").await;

                if host.is_empty() || from.is_empty() || to_list.is_empty() {
                    return Err(IIIError::Handler(
                        "email.smtp_host, email.from, and email.to are required".into(),
                    ));
                }

                let records: Vec<CostRecord> =
                    kv.list("cost_records").await.map_err(kv_err)?;
                let cutoff = Utc::now() - Duration::days(7);
                let weekly: Vec<&CostRecord> = records
                    .iter()
                    .filter(|r| r.recorded_at >= cutoff)
                    .collect();

                let total_cost: f64 = weekly.iter().map(|r| r.total_cost).sum();
                let total_input: u64 = weekly.iter().map(|r| r.input_tokens).sum();
                let total_output: u64 = weekly.iter().map(|r| r.output_tokens).sum();

                let body = format!(
                    "Rimuru weekly cost digest\n\n\
                     Window: last 7 days\n\
                     Records: {}\n\
                     Total cost: ${:.4}\n\
                     Input tokens: {}\n\
                     Output tokens: {}\n",
                    weekly.len(),
                    total_cost,
                    total_input,
                    total_output
                );

                let from_mbox: Mailbox = from
                    .parse()
                    .map_err(|e| IIIError::Handler(format!("invalid from: {}", e)))?;

                let mut builder = Message::builder()
                    .from(from_mbox)
                    .subject("Rimuru weekly cost digest");
                for addr in &to_list {
                    let mbox: Mailbox = addr
                        .parse()
                        .map_err(|e| IIIError::Handler(format!("invalid to '{}': {}", addr, e)))?;
                    builder = builder.to(mbox);
                }

                let email = builder
                    .header(ContentType::TEXT_PLAIN)
                    .body(body)
                    .map_err(|e| IIIError::Handler(format!("build email: {}", e)))?;

                let mut mailer = SmtpTransport::relay(&host)
                    .map_err(|e| IIIError::Handler(format!("smtp relay: {}", e)))?
                    .port(port);

                if !username.is_empty() {
                    mailer = mailer.credentials(Credentials::new(username, password));
                }

                mailer
                    .build()
                    .send(&email)
                    .map_err(|e| IIIError::Handler(format!("smtp send: {}", e)))?;

                Ok(api_response(json!({
                    "sent": true,
                    "recipients": to_list,
                    "record_count": weekly.len(),
                    "total_cost": total_cost
                })))
            }
        },
    );
}

async fn get_str(kv: &StateKV, key: &str, default: &str) -> String {
    kv.get::<Value>("config", key)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| default.to_string())
}

async fn get_u64(kv: &StateKV, key: &str, default: u64) -> u64 {
    kv.get::<Value>("config", key)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_u64())
        .unwrap_or(default)
}

async fn get_bool(kv: &StateKV, key: &str, default: bool) -> bool {
    kv.get::<Value>("config", key)
        .await
        .ok()
        .flatten()
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

async fn get_list(kv: &StateKV, key: &str) -> Vec<String> {
    match kv.get::<Value>("config", key).await {
        Ok(Some(Value::Array(arr))) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Ok(Some(Value::String(s))) => s
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}
