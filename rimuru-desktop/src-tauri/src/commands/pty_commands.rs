use base64::Engine;
use serde::Deserialize;
use tauri::State;

use crate::pty::manager::PtySessionManager;
use crate::pty::session::PtySessionInfo;

#[derive(Deserialize)]
pub struct LaunchRequest {
    pub agent_type: String,
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub working_dir: String,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub initial_prompt: Option<String>,
}

#[tauri::command]
pub async fn launch_session(
    request: LaunchRequest,
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<String, String> {
    state.launch(
        request.agent_type,
        request.executable,
        request.args.unwrap_or_default(),
        request.working_dir,
        request.cols.unwrap_or(120),
        request.rows.unwrap_or(30),
        request.initial_prompt,
    )
}

#[tauri::command]
pub async fn write_to_session(
    session_id: String,
    data_base64: String,
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<(), String> {
    let data = base64::engine::general_purpose::STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    state.write(&session_id, &data)
}

#[tauri::command]
pub async fn resize_session(
    session_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<(), String> {
    state.resize(&session_id, cols, rows)
}

#[tauri::command]
pub async fn terminate_session(
    session_id: String,
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<(), String> {
    state.terminate(&session_id)
}

#[tauri::command]
pub async fn list_live_sessions(
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<Vec<PtySessionInfo>, String> {
    Ok(state.list_active())
}

#[tauri::command]
pub async fn get_live_session(
    session_id: String,
    state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<PtySessionInfo, String> {
    state
        .get_info(&session_id)
        .ok_or_else(|| format!("Session not found: {}", session_id))
}

#[tauri::command]
pub fn create_git_worktree(repo_path: String, branch_name: String) -> Result<String, String> {
    crate::pty::worktree::create_worktree(&repo_path, &branch_name)
}

#[tauri::command]
pub fn cleanup_git_worktree(repo_path: String, worktree_path: String) -> Result<(), String> {
    crate::pty::worktree::cleanup_worktree(&repo_path, &worktree_path)
}

#[tauri::command]
pub fn list_git_worktrees(
    repo_path: String,
) -> Result<Vec<crate::pty::worktree::WorktreeInfo>, String> {
    crate::pty::worktree::list_worktrees(&repo_path)
}

#[tauri::command]
pub fn discover_agent_sessions() -> Vec<crate::pty::discovery::DiscoveredSession> {
    crate::pty::discovery::discover_sessions()
}

#[tauri::command]
pub fn list_playbooks() -> Vec<crate::playbooks::types::Playbook> {
    crate::playbooks::list_playbooks()
}

#[tauri::command]
pub fn load_playbook(path: String) -> Result<crate::playbooks::types::Playbook, String> {
    crate::playbooks::load_playbook_from_path(std::path::Path::new(&path))
}
