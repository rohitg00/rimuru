use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetThresholdCtx {
    pub current: f64,
    pub limit: f64,
    pub percent: f64,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostCtx {
    pub session_id: String,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunawayCtx {
    pub agent: String,
    pub session_id: String,
    pub tool_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationCtx {
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub enum NotificationKind {
    BudgetThreshold { level: u8, ctx: BudgetThresholdCtx },
    SessionCostMilestone(SessionCostCtx),
    RunawayDetected(RunawayCtx),
    OptimizationOpportunity(OptimizationCtx),
}

pub fn build_notification(kind: &NotificationKind) -> NotificationPayload {
    match kind {
        NotificationKind::BudgetThreshold { level, ctx } => {
            let title = match level {
                l if *l >= 100 => "Budget exceeded".to_string(),
                l if *l >= 80 => "Budget warning 80%".to_string(),
                _ => "Budget warning 50%".to_string(),
            };
            let agent = ctx.agent.as_deref().unwrap_or("all");
            let body = format!(
                "${:.2} / ${:.2} ({:.0}%) — agent: {}",
                ctx.current, ctx.limit, ctx.percent, agent
            );
            NotificationPayload { title, body }
        }
        NotificationKind::SessionCostMilestone(ctx) => NotificationPayload {
            title: "Session cost milestone".to_string(),
            body: format!("Session {} crossed ${:.2}", ctx.session_id, ctx.cost),
        },
        NotificationKind::RunawayDetected(ctx) => NotificationPayload {
            title: "Runaway loop detected".to_string(),
            body: format!(
                "{} on session {} executed {} tool calls",
                ctx.agent, ctx.session_id, ctx.tool_count
            ),
        },
        NotificationKind::OptimizationOpportunity(ctx) => NotificationPayload {
            title: "Optimization available".to_string(),
            body: ctx.recommendation.clone(),
        },
    }
}

pub struct NotificationDispatcher<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> Clone for NotificationDispatcher<R> {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
        }
    }
}

impl<R: Runtime> NotificationDispatcher<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }

    pub fn dispatch(&self, kind: &NotificationKind) -> Result<(), String> {
        let notifier = self.app.notification();
        match notifier.permission_state() {
            Ok(tauri::plugin::PermissionState::Granted) => {}
            Ok(_) => {
                tracing::warn!(
                    "notification dispatch skipped: permission not granted (call request_permission at startup)"
                );
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("notification permission check failed: {e}");
                return Err(e.to_string());
            }
        }
        let payload = build_notification(kind);
        notifier
            .builder()
            .title(&payload.title)
            .body(&payload.body)
            .show()
            .map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationPreferences {
    #[serde(default = "default_true")]
    pub budget_enabled: bool,
    #[serde(default = "default_true")]
    pub session_cost_enabled: bool,
    #[serde(default = "default_true")]
    pub runaway_enabled: bool,
    #[serde(default = "default_true")]
    pub optimization_enabled: bool,
    #[serde(default = "default_session_cost_threshold")]
    pub session_cost_threshold: f64,
}

fn default_true() -> bool {
    true
}

fn default_session_cost_threshold() -> f64 {
    5.0
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            budget_enabled: true,
            session_cost_enabled: true,
            runaway_enabled: true,
            optimization_enabled: true,
            session_cost_threshold: default_session_cost_threshold(),
        }
    }
}

impl NotificationPreferences {
    pub async fn load(kv: &rimuru_core::StateKV) -> Self {
        let mut prefs = Self::default();
        if let Ok(Some(v)) = kv
            .get::<serde_json::Value>("config", "notifications.budget_enabled")
            .await
            && let Some(b) = v.as_bool()
        {
            prefs.budget_enabled = b;
        }
        if let Ok(Some(v)) = kv
            .get::<serde_json::Value>("config", "notifications.session_cost_enabled")
            .await
            && let Some(b) = v.as_bool()
        {
            prefs.session_cost_enabled = b;
        }
        if let Ok(Some(v)) = kv
            .get::<serde_json::Value>("config", "notifications.runaway_enabled")
            .await
            && let Some(b) = v.as_bool()
        {
            prefs.runaway_enabled = b;
        }
        if let Ok(Some(v)) = kv
            .get::<serde_json::Value>("config", "notifications.optimization_enabled")
            .await
            && let Some(b) = v.as_bool()
        {
            prefs.optimization_enabled = b;
        }
        if let Ok(Some(v)) = kv
            .get::<serde_json::Value>("config", "notifications.session_cost_threshold")
            .await
            && let Some(n) = v.as_f64()
        {
            prefs.session_cost_threshold = n;
        }
        prefs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_budget_threshold_50() {
        let kind = NotificationKind::BudgetThreshold {
            level: 50,
            ctx: BudgetThresholdCtx {
                current: 5.0,
                limit: 10.0,
                percent: 50.0,
                agent: Some("claude_code".to_string()),
            },
        };
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Budget warning 50%");
        assert!(payload.body.contains("$5.00"));
        assert!(payload.body.contains("$10.00"));
        assert!(payload.body.contains("50%"));
        assert!(payload.body.contains("claude_code"));
    }

    #[test]
    fn build_budget_threshold_80() {
        let kind = NotificationKind::BudgetThreshold {
            level: 80,
            ctx: BudgetThresholdCtx {
                current: 8.0,
                limit: 10.0,
                percent: 80.0,
                agent: None,
            },
        };
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Budget warning 80%");
        assert!(payload.body.contains("all"));
    }

    #[test]
    fn build_budget_threshold_exceeded() {
        let kind = NotificationKind::BudgetThreshold {
            level: 100,
            ctx: BudgetThresholdCtx {
                current: 12.5,
                limit: 10.0,
                percent: 125.0,
                agent: None,
            },
        };
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Budget exceeded");
        assert!(payload.body.contains("$12.50"));
    }

    #[test]
    fn build_session_cost_milestone() {
        let kind = NotificationKind::SessionCostMilestone(SessionCostCtx {
            session_id: "sess-42".to_string(),
            cost: 7.25,
        });
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Session cost milestone");
        assert!(payload.body.contains("sess-42"));
        assert!(payload.body.contains("$7.25"));
    }

    #[test]
    fn build_runaway_detected() {
        let kind = NotificationKind::RunawayDetected(RunawayCtx {
            agent: "cursor".to_string(),
            session_id: "sess-1".to_string(),
            tool_count: 42,
        });
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Runaway loop detected");
        assert!(payload.body.contains("cursor"));
        assert!(payload.body.contains("sess-1"));
        assert!(payload.body.contains("42"));
    }

    #[test]
    fn build_optimization_opportunity() {
        let kind = NotificationKind::OptimizationOpportunity(OptimizationCtx {
            recommendation: "Switch to haiku for classification".to_string(),
        });
        let payload = build_notification(&kind);
        assert_eq!(payload.title, "Optimization available");
        assert_eq!(payload.body, "Switch to haiku for classification");
    }

    #[test]
    fn preferences_defaults() {
        let prefs = NotificationPreferences::default();
        assert!(prefs.budget_enabled);
        assert!(prefs.session_cost_enabled);
        assert!(prefs.runaway_enabled);
        assert!(prefs.optimization_enabled);
        assert!((prefs.session_cost_threshold - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn preferences_serde_roundtrip() {
        let prefs = NotificationPreferences {
            budget_enabled: false,
            session_cost_enabled: true,
            runaway_enabled: false,
            optimization_enabled: true,
            session_cost_threshold: 12.5,
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let parsed: NotificationPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(prefs, parsed);
    }

    #[test]
    fn preferences_serde_fills_defaults() {
        let parsed: NotificationPreferences = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed, NotificationPreferences::default());
    }
}
