use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Serialize, Clone)]
pub struct DiscoveredSession {
    pub provider: String,
    pub project_name: String,
    pub project_path: String,
    pub last_active: Option<String>,
    pub session_count: usize,
}

pub fn discover_sessions() -> Vec<DiscoveredSession> {
    let mut sessions = Vec::new();

    let home = std::env::var("HOME").unwrap_or_default();

    discover_claude_sessions(&home, &mut sessions);
    discover_codex_sessions(&home, &mut sessions);
    discover_goose_sessions(&home, &mut sessions);
    discover_cursor_sessions(&home, &mut sessions);

    sessions.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    sessions
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let naive = chrono::DateTime::from_timestamp(secs as i64, 0).unwrap_or_default();
    naive.to_rfc3339()
}

fn latest_modified_in_dir(dir: &PathBuf) -> Option<SystemTime> {
    let mut latest: Option<SystemTime> = None;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    latest = Some(match latest {
                        Some(prev) if modified > prev => modified,
                        Some(prev) => prev,
                        None => modified,
                    });
                }
            }
        }
    }
    latest
}

fn decode_claude_project_path(dir_name: &str) -> String {
    let decoded = dir_name.replacen('-', "/", 1);
    let decoded = decoded.replace('-', "/");
    if decoded.starts_with('/') {
        decoded
    } else {
        format!("/{}", decoded)
    }
}

fn project_name_from_path(path: &str) -> String {
    path.rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn discover_claude_sessions(home: &str, sessions: &mut Vec<DiscoveredSession>) {
    let projects_dir = PathBuf::from(home).join(".claude").join("projects");
    let entries = match fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        let project_path = decode_claude_project_path(&dir_name);
        let project_name = project_name_from_path(&project_path);

        let session_count = fs::read_dir(entry.path())
            .map(|rd| rd.flatten().count())
            .unwrap_or(0);

        let last_active = latest_modified_in_dir(&entry.path()).map(system_time_to_rfc3339);

        sessions.push(DiscoveredSession {
            provider: "Claude Code".to_string(),
            project_name,
            project_path,
            last_active,
            session_count,
        });
    }
}

fn discover_codex_sessions(home: &str, sessions: &mut Vec<DiscoveredSession>) {
    let codex_dir = PathBuf::from(home).join(".codex");
    if !codex_dir.exists() {
        return;
    }

    let sessions_dir = codex_dir.join("sessions");
    if sessions_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    continue;
                }

                let dir_name = entry.file_name().to_string_lossy().to_string();
                let session_count = fs::read_dir(entry.path())
                    .map(|rd| rd.flatten().count())
                    .unwrap_or(1);
                let last_active = latest_modified_in_dir(&entry.path()).map(system_time_to_rfc3339);

                sessions.push(DiscoveredSession {
                    provider: "Codex".to_string(),
                    project_name: dir_name.clone(),
                    project_path: dir_name,
                    last_active,
                    session_count,
                });
            }
        }
    } else {
        let last_active = fs::metadata(&codex_dir)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(system_time_to_rfc3339);

        sessions.push(DiscoveredSession {
            provider: "Codex".to_string(),
            project_name: "Codex Workspace".to_string(),
            project_path: codex_dir.to_string_lossy().to_string(),
            last_active,
            session_count: 1,
        });
    }
}

fn discover_goose_sessions(home: &str, sessions: &mut Vec<DiscoveredSession>) {
    let sessions_dir = PathBuf::from(home)
        .join(".config")
        .join("goose")
        .join("sessions");

    let entries = match fs::read_dir(&sessions_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_session = path.is_file()
            && path
                .extension()
                .map(|ext| ext == "json" || ext == "jsonl")
                .unwrap_or(false);

        if !is_session {
            continue;
        }

        let file_name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let last_active = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(system_time_to_rfc3339);

        sessions.push(DiscoveredSession {
            provider: "Goose".to_string(),
            project_name: file_name,
            project_path: path.to_string_lossy().to_string(),
            last_active,
            session_count: 1,
        });
    }
}

fn discover_cursor_sessions(home: &str, sessions: &mut Vec<DiscoveredSession>) {
    let workspace_dir = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Cursor")
        .join("User")
        .join("workspaceStorage");

    let entries = match fs::read_dir(&workspace_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let workspace_json = entry.path().join("workspace.json");
        let project_path = if workspace_json.exists() {
            fs::read_to_string(&workspace_json)
                .ok()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
                .and_then(|v| v.get("folder")?.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| entry.file_name().to_string_lossy().to_string())
        } else {
            entry.file_name().to_string_lossy().to_string()
        };

        let project_name = project_name_from_path(&project_path);
        let session_count = fs::read_dir(entry.path())
            .map(|rd| rd.flatten().count())
            .unwrap_or(0);
        let last_active = latest_modified_in_dir(&entry.path()).map(system_time_to_rfc3339);

        sessions.push(DiscoveredSession {
            provider: "Cursor".to_string(),
            project_name,
            project_path,
            last_active,
            session_count,
        });
    }
}
