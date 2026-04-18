use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use iii_sdk::{III, IIIError, RegisterFunctionMessage};
use serde_json::{Value, json};

use super::sysutil::{api_response, extract_input, kv_err};
use crate::models::CostRecord;
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.costs.export".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);

                let format = input
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("csv")
                    .to_lowercase();

                let period = input
                    .get("period")
                    .and_then(|v| v.as_str())
                    .unwrap_or("monthly")
                    .to_lowercase();

                let (default_from, default_to) = period_window(&period)?;

                let from = input
                    .get("from")
                    .and_then(|v| v.as_str())
                    .map(parse_iso)
                    .transpose()?
                    .or(default_from);

                let to = input
                    .get("to")
                    .and_then(|v| v.as_str())
                    .map(parse_iso)
                    .transpose()?
                    .or(default_to);

                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let filtered: Vec<CostRecord> = records
                    .into_iter()
                    .filter(|r| from.is_none_or(|f| r.recorded_at >= f))
                    .filter(|r| to.is_none_or(|t| r.recorded_at <= t))
                    .collect();

                let (body, content_type) = match format.as_str() {
                    "csv" => (render_csv(&filtered)?, "text/csv".to_string()),
                    "json" => (render_json(&filtered)?, "application/json".to_string()),
                    other => {
                        return Err(IIIError::Handler(format!("unsupported format: {}", other)));
                    }
                };

                Ok(api_response(json!({
                    "format": format,
                    "period": period,
                    "from": from.map(|d| d.to_rfc3339()),
                    "to": to.map(|d| d.to_rfc3339()),
                    "record_count": filtered.len(),
                    "content_type": content_type,
                    "body": body
                })))
            }
        },
    );
}

fn parse_iso(s: &str) -> Result<DateTime<Utc>, IIIError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| IIIError::Handler(format!("invalid datetime '{}': {}", s, e)))
}

type TimeWindow = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

fn period_window(period: &str) -> Result<TimeWindow, IIIError> {
    let now = Utc::now();
    match period {
        "daily" => {
            let start = now - Duration::days(1);
            Ok((Some(start), Some(now)))
        }
        "weekly" => {
            let start = now - Duration::days(7);
            Ok((Some(start), Some(now)))
        }
        "monthly" => {
            let first = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
                .ok_or_else(|| IIIError::Handler("failed to compute month start".into()))?;
            let start = first
                .and_hms_opt(0, 0, 0)
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .ok_or_else(|| IIIError::Handler("failed to compute month start".into()))?;
            Ok((Some(start), Some(now)))
        }
        "custom" => Ok((None, None)),
        other => Err(IIIError::Handler(format!("unsupported period: {}", other))),
    }
}

pub fn render_csv(records: &[CostRecord]) -> Result<String, IIIError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "recorded_at",
        "agent_type",
        "model",
        "provider",
        "input_tokens",
        "output_tokens",
        "cache_read_tokens",
        "cache_write_tokens",
        "total_cost",
        "session_id",
        "user_id",
        "team_id",
    ])
    .map_err(|e| IIIError::Handler(format!("csv header: {}", e)))?;

    for r in records {
        let agent_type = serde_json::to_value(r.agent_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        let session_id = r.session_id.map(|s| s.to_string()).unwrap_or_default();

        wtr.write_record([
            r.recorded_at.to_rfc3339(),
            agent_type,
            r.model.clone(),
            r.provider.clone(),
            r.input_tokens.to_string(),
            r.output_tokens.to_string(),
            r.cache_read_tokens.to_string(),
            r.cache_write_tokens.to_string(),
            r.total_cost.to_string(),
            session_id,
            String::new(),
            String::new(),
        ])
        .map_err(|e| IIIError::Handler(format!("csv row: {}", e)))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| IIIError::Handler(format!("csv flush: {}", e)))?;
    String::from_utf8(bytes).map_err(|e| IIIError::Handler(format!("csv utf8: {}", e)))
}

pub fn render_json(records: &[CostRecord]) -> Result<String, IIIError> {
    serde_json::to_string(records).map_err(|e| IIIError::Handler(format!("json: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentType, CostRecord};
    use uuid::Uuid;

    fn sample_record() -> CostRecord {
        let mut r = CostRecord::new(
            Uuid::nil(),
            AgentType::ClaudeCode,
            "claude-sonnet-4".to_string(),
            "anthropic".to_string(),
            1000,
            500,
            0.003,
            0.015,
        );
        r.cache_read_tokens = 42;
        r.cache_write_tokens = 7;
        r
    }

    #[test]
    fn csv_header_and_row() {
        let records = vec![sample_record()];
        let csv = render_csv(&records).unwrap();
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        assert_eq!(
            header,
            "recorded_at,agent_type,model,provider,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,total_cost,session_id,user_id,team_id"
        );
        let row = lines.next().unwrap();
        assert!(row.contains("claude-sonnet-4"));
        assert!(row.contains("anthropic"));
        assert!(row.contains("1000"));
        assert!(row.contains("500"));
        assert!(row.contains("42"));
        assert!(row.contains(",,"));
    }

    #[test]
    fn json_roundtrip() {
        let records = vec![sample_record()];
        let json = render_json(&records).unwrap();
        let parsed: Vec<CostRecord> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].model, "claude-sonnet-4");
        assert_eq!(parsed[0].cache_read_tokens, 42);
    }
}
