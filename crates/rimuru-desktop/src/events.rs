use tauri::{AppHandle, Emitter, Runtime};

#[allow(dead_code)]
pub const EVENT_NAVIGATE: &str = "navigate";
#[allow(dead_code)]
pub const EVENT_AGENT_UPDATED: &str = "agent-updated";
#[allow(dead_code)]
pub const EVENT_SYNC_COMPLETE: &str = "sync-complete";
#[allow(dead_code)]
pub const EVENT_COST_ALERT: &str = "cost-alert";
#[allow(dead_code)]
pub const EVENT_HEALTH_CHANGED: &str = "health-changed";

#[allow(dead_code)]
pub fn emit_event<R: Runtime>(app: &AppHandle<R>, event: &str, payload: serde_json::Value) {
    app.emit(event, payload).ok();
}
