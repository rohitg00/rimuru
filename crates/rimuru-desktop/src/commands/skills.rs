use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn search_skills(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u64>,
) -> Result<Value, String> {
    let mut input = json!({ "query": query });
    if let Some(l) = limit {
        input["limit"] = json!(l);
    }
    state.call("rimuru.skillkit.search", input).await
}

#[tauri::command]
pub async fn install_skill(
    state: State<'_, AppState>,
    skill: String,
    agent: Option<String>,
) -> Result<Value, String> {
    let mut input = json!({ "skill": skill });
    if let Some(a) = agent {
        input["agent"] = json!(a);
    }
    state.call("rimuru.skillkit.install", input).await
}

#[tauri::command]
pub async fn translate_skill(
    state: State<'_, AppState>,
    skill: String,
    target_agent: String,
) -> Result<Value, String> {
    state
        .call(
            "rimuru.skillkit.translate",
            json!({ "skill": skill, "target_agent": target_agent }),
        )
        .await
}

#[tauri::command]
pub async fn recommend_skills(
    state: State<'_, AppState>,
    context: Option<String>,
    agent: Option<String>,
    limit: Option<u64>,
) -> Result<Value, String> {
    let mut input = json!({});
    if let Some(c) = context {
        input["context"] = json!(c);
    }
    if let Some(a) = agent {
        input["agent"] = json!(a);
    }
    if let Some(l) = limit {
        input["limit"] = json!(l);
    }
    state.call("rimuru.skillkit.recommend", input).await
}
