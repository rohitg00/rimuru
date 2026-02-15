pub mod types;

use std::fs;
use std::path::PathBuf;
use types::*;

pub fn playbooks_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".rimuru").join("playbooks")
}

pub fn list_playbooks() -> Vec<Playbook> {
    let dir = playbooks_dir();
    if !dir.exists() {
        return vec![];
    }

    let mut playbooks = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                if let Ok(pb) = load_playbook_from_path(&path) {
                    playbooks.push(pb);
                }
            }
        }
    }
    playbooks
}

pub fn load_playbook_from_path(path: &std::path::Path) -> Result<Playbook, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: toml::Value = content
        .parse()
        .map_err(|e: toml::de::Error| e.to_string())?;

    let playbook_table = parsed.get("playbook").ok_or("Missing [playbook] section")?;
    let name = playbook_table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed")
        .to_string();
    let description = playbook_table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let steps_array = parsed
        .get("steps")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let steps: Vec<PlaybookStep> = steps_array
        .iter()
        .map(|s| PlaybookStep {
            name: s
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Step")
                .to_string(),
            prompt: s
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            agent_type: s
                .get("agent_type")
                .and_then(|v| v.as_str())
                .unwrap_or("claude_code")
                .to_string(),
            working_dir: s
                .get("working_dir")
                .and_then(|v| v.as_str())
                .map(String::from),
            gate: s
                .get("gate")
                .and_then(|v| v.as_str())
                .unwrap_or("auto")
                .to_string(),
            timeout_secs: s
                .get("timeout_secs")
                .and_then(|v| v.as_integer())
                .map(|v| v as u64),
        })
        .collect();

    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(Playbook {
        id,
        name,
        description,
        steps,
        file_path: path.to_string_lossy().to_string(),
    })
}
