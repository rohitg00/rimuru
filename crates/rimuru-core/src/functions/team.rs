use std::collections::HashMap;

use chrono::{DateTime, Utc};
use iii_sdk::{III, IIIError, RegisterFunctionMessage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use super::jwt::authorize;
use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::models::CostRecord;
use crate::state::StateKV;

pub const TEAM_SCOPE: &str = "team";
pub const TEAM_MEMBER_SCOPE: &str = "team_member";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub budget_limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub team_id: String,
    pub user_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub joined_at: DateTime<Utc>,
}

/// Composite key for team_member entries. Format: `{team_id}:{user_id}`.
pub fn member_key(team_id: &str, user_id: &str) -> String {
    format!("{}:{}", team_id, user_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBreakdown {
    pub user_id: String,
    pub display_name: Option<String>,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub record_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCostAggregation {
    pub team_id: String,
    pub member_count: usize,
    pub grand_total: f64,
    pub total_records: u64,
    pub per_user: Vec<UserBreakdown>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

pub fn aggregate(
    members: &[TeamMember],
    records: &[CostRecord],
    team_id: &str,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> TeamCostAggregation {
    let mut by_user: HashMap<String, UserBreakdown> = HashMap::new();
    for m in members {
        by_user.insert(
            m.user_id.clone(),
            UserBreakdown {
                user_id: m.user_id.clone(),
                display_name: m.display_name.clone(),
                total_cost: 0.0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                record_count: 0,
            },
        );
    }

    let mut total_records: u64 = 0;
    for r in records {
        let user = match &r.user_id {
            Some(u) => u,
            None => continue,
        };
        let matches_team = r.team_id.as_deref() == Some(team_id);
        let known_member = by_user.contains_key(user);
        if !matches_team && !known_member {
            continue;
        }
        if let Some(s) = since
            && r.recorded_at < s
        {
            continue;
        }
        if let Some(u) = until
            && r.recorded_at > u
        {
            continue;
        }

        let entry = by_user
            .entry(user.clone())
            .or_insert_with(|| UserBreakdown {
                user_id: user.clone(),
                display_name: None,
                total_cost: 0.0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                record_count: 0,
            });
        entry.total_cost += r.total_cost;
        entry.total_input_tokens += r.input_tokens;
        entry.total_output_tokens += r.output_tokens;
        entry.record_count += 1;
        total_records += 1;
    }

    let mut per_user: Vec<UserBreakdown> = by_user.into_values().collect();
    per_user.sort_by(|a, b| {
        b.total_cost
            .partial_cmp(&a.total_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let grand_total: f64 = per_user.iter().map(|u| u.total_cost).sum();

    TeamCostAggregation {
        team_id: team_id.to_string(),
        member_count: members.len(),
        grand_total,
        total_records,
        per_user,
        period_start: since,
        period_end: until,
    }
}

fn parse_rfc3339(input: &Value, key: &str) -> Result<Option<DateTime<Utc>>, IIIError> {
    match input.get(key).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => DateTime::parse_from_rfc3339(s)
            .map(|dt| Some(dt.with_timezone(&Utc)))
            .map_err(|e| IIIError::Handler(format!("invalid {} datetime: {}", key, e))),
        _ => Ok(None),
    }
}

pub fn register(iii: &III, kv: &StateKV) {
    register_create(iii, kv);
    register_add_user(iii, kv);
    register_costs(iii, kv);
    register_leaderboard(iii, kv);
}

fn register_create(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.team.create".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                authorize(&input)?;
                let body = extract_input(input);
                let name = require_str(&body, "name")?;
                let budget_limit = body
                    .get("budget_limit")
                    .and_then(|v| v.as_f64())
                    .filter(|n| n.is_finite() && *n >= 0.0);

                let team = Team {
                    id: Uuid::new_v4().to_string(),
                    name,
                    created_at: Utc::now(),
                    budget_limit,
                };

                kv.set(TEAM_SCOPE, &team.id, &team)
                    .await
                    .map_err(kv_err)?;

                Ok(api_response(json!(team)))
            }
        },
    );
}

fn register_add_user(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.team.add_user".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                authorize(&input)?;
                let body = extract_input(input);
                let team_id = require_str(&body, "team_id")?;
                let user_id = require_str(&body, "user_id")?;
                let display_name = body
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let team: Option<Team> = kv.get(TEAM_SCOPE, &team_id).await.map_err(kv_err)?;
                if team.is_none() {
                    return Err(IIIError::Handler(format!("team {} not found", team_id)));
                }

                let member = TeamMember {
                    team_id: team_id.clone(),
                    user_id: user_id.clone(),
                    display_name,
                    joined_at: Utc::now(),
                };
                let key = member_key(&team_id, &user_id);
                kv.set(TEAM_MEMBER_SCOPE, &key, &member)
                    .await
                    .map_err(kv_err)?;

                Ok(api_response(json!(member)))
            }
        },
    );
}

async fn load_members(kv: &StateKV, team_id: &str) -> Result<Vec<TeamMember>, IIIError> {
    let all: Vec<TeamMember> = kv.list(TEAM_MEMBER_SCOPE).await.map_err(kv_err)?;
    Ok(all.into_iter().filter(|m| m.team_id == team_id).collect())
}

fn register_costs(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.team.costs".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                authorize(&input)?;
                let body = extract_input(input);
                let team_id = require_str(&body, "team_id")?;
                let since = parse_rfc3339(&body, "from")?;
                let until = parse_rfc3339(&body, "to")?;

                let members = load_members(&kv, &team_id).await?;
                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let agg = aggregate(&members, &records, &team_id, since, until);
                Ok(api_response(json!(agg)))
            }
        },
    );
}

fn register_leaderboard(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.team.leaderboard".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                authorize(&input)?;
                let body = extract_input(input);
                let team_id = require_str(&body, "team_id")?;
                let since = parse_rfc3339(&body, "from")?;
                let until = parse_rfc3339(&body, "to")?;

                let members = load_members(&kv, &team_id).await?;
                let records: Vec<CostRecord> = kv.list("cost_records").await.map_err(kv_err)?;

                let agg = aggregate(&members, &records, &team_id, since, until);
                let top = agg.per_user.first().cloned();

                Ok(api_response(json!({
                    "team_id": team_id,
                    "member_count": agg.member_count,
                    "grand_total": agg.grand_total,
                    "top_spender": top,
                    "leaderboard": agg.per_user,
                    "period_start": agg.period_start,
                    "period_end": agg.period_end,
                })))
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::super::jwt::{Claims, encode_hs256, verify_hs256};
    use super::*;
    use crate::models::AgentType;

    fn sample_record(user: &str, team: Option<&str>, cost: f64) -> CostRecord {
        let mut r = CostRecord::new(
            Uuid::new_v4(),
            AgentType::ClaudeCode,
            "claude-opus".to_string(),
            "anthropic".to_string(),
            100,
            200,
            cost / 2.0,
            cost / 2.0,
        );
        r.user_id = Some(user.to_string());
        r.team_id = team.map(String::from);
        r
    }

    #[test]
    fn member_key_format() {
        assert_eq!(member_key("team-1", "alice"), "team-1:alice");
    }

    #[test]
    fn aggregate_team_costs() {
        let members = vec![
            TeamMember {
                team_id: "t1".into(),
                user_id: "alice".into(),
                display_name: Some("Alice".into()),
                joined_at: Utc::now(),
            },
            TeamMember {
                team_id: "t1".into(),
                user_id: "bob".into(),
                display_name: None,
                joined_at: Utc::now(),
            },
        ];
        let records = vec![
            sample_record("alice", Some("t1"), 1.50),
            sample_record("alice", Some("t1"), 0.50),
            sample_record("bob", Some("t1"), 3.00),
            sample_record("carol", Some("t2"), 99.0),
            sample_record("dave", None, 10.0),
        ];

        let agg = aggregate(&members, &records, "t1", None, None);
        assert_eq!(agg.member_count, 2);
        assert_eq!(agg.total_records, 3);
        assert!((agg.grand_total - 5.0).abs() < 1e-9);

        let top = &agg.per_user[0];
        assert_eq!(top.user_id, "bob");
        assert!((top.total_cost - 3.0).abs() < 1e-9);
        let alice = agg.per_user.iter().find(|u| u.user_id == "alice").unwrap();
        assert_eq!(alice.record_count, 2);
        assert!((alice.total_cost - 2.0).abs() < 1e-9);
    }

    #[test]
    fn jwt_roundtrip_and_reject_bad_sig() {
        let secret = b"super-secret-key";
        let claims = Claims {
            sub: Some("alice".into()),
            user_id: Some("alice".into()),
            team_id: Some("t1".into()),
            exp: Some(Utc::now().timestamp() + 3600),
            extra: Default::default(),
        };
        let token = encode_hs256(&claims, secret).unwrap();
        let parsed = verify_hs256(&token, secret).unwrap();
        assert_eq!(parsed.user(), Some("alice".into()));
        assert_eq!(parsed.team_id.as_deref(), Some("t1"));

        let wrong = verify_hs256(&token, b"not-the-same");
        assert!(wrong.is_err());
    }

    #[test]
    fn jwt_rejects_expired() {
        let secret = b"k";
        let claims = Claims {
            sub: Some("x".into()),
            user_id: None,
            team_id: None,
            exp: Some(Utc::now().timestamp() - 10),
            extra: Default::default(),
        };
        let token = encode_hs256(&claims, secret).unwrap();
        assert!(verify_hs256(&token, secret).is_err());
    }

    #[test]
    fn aggregate_applies_date_window() {
        let members = vec![TeamMember {
            team_id: "t1".into(),
            user_id: "alice".into(),
            display_name: None,
            joined_at: Utc::now(),
        }];
        let now = Utc::now();
        let mut old = sample_record("alice", Some("t1"), 10.0);
        old.recorded_at = now - chrono::Duration::days(30);
        let mut recent = sample_record("alice", Some("t1"), 5.0);
        recent.recorded_at = now - chrono::Duration::days(1);
        let records = vec![old, recent];

        let cutoff = now - chrono::Duration::days(7);
        let agg = aggregate(&members, &records, "t1", Some(cutoff), None);
        assert_eq!(agg.total_records, 1);
        assert!((agg.grand_total - 5.0).abs() < 1e-9);
    }
}
