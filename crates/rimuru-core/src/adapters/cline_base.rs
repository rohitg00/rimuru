//! Shared parsing helpers for Cline and its forks (Roo Code, etc.)
//!
//! Cline and Roo Code are both VS Code extensions and store their
//! conversation history under the editor's `globalStorage` directory
//! in a tasks subtree. Each task has its own folder containing
//! `api_conversation_history.json` (or `.jsonl`) plus a `ui_messages.json`.
//! The on-disk format is close enough that parsing logic lives here
//! once instead of being duplicated in two adapters.
//!
//! The two surfaces differ in:
//!
//! - Extension ID: `saoudrizwan.claude-dev` vs `rooveterinaryinc.roo-cline`
//! - Display name: `Cline` vs `Roo Code`
//! - AgentType variant emitted on the parsed Session
//!
//! Everything else (path layout, JSON shape, token fields) is shared.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::Value;
use tracing::warn;
use uuid::Uuid;

use crate::error::RimuruError;
use crate::models::{AgentType, Session};

type Result<T> = std::result::Result<T, RimuruError>;

/// Where VS Code (and forks like Cursor / VSCodium) keep extension
/// global storage on each platform. Used to locate Cline / Roo task
/// directories without depending on a particular IDE distribution.
pub fn vscode_global_storage_candidates() -> Vec<PathBuf> {
    // No home dir → no candidates. The $home-relative paths below
    // wouldn't match anything under /tmp either, so returning an
    // empty list is cleaner than producing misleading entries.
    let Some(home) = dirs::home_dir() else {
        tracing::warn!("cline_base: home directory not available, skipping VS Code candidates");
        return Vec::new();
    };
    let mut out = Vec::new();

    #[cfg(target_os = "macos")]
    {
        out.push(home.join("Library/Application Support/Code/User/globalStorage"));
        out.push(home.join("Library/Application Support/Code - Insiders/User/globalStorage"));
        out.push(home.join("Library/Application Support/VSCodium/User/globalStorage"));
        out.push(home.join("Library/Application Support/Cursor/User/globalStorage"));
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(cfg) = dirs::config_dir() {
            out.push(cfg.join("Code/User/globalStorage"));
            out.push(cfg.join("Code - Insiders/User/globalStorage"));
            out.push(cfg.join("VSCodium/User/globalStorage"));
            out.push(cfg.join("Cursor/User/globalStorage"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(cfg) = dirs::config_dir() {
            out.push(cfg.join("Code/User/globalStorage"));
            out.push(cfg.join("Code - Insiders/User/globalStorage"));
            out.push(cfg.join("VSCodium/User/globalStorage"));
            out.push(cfg.join("Cursor/User/globalStorage"));
        }
    }

    out.push(home.join(".vscode/extensions"));
    out
}

/// Resolve the on-disk extension storage directory for `extension_id`
/// across every known IDE storage root. Returns the first match that
/// actually exists, or None when the extension isn't installed.
pub fn find_extension_storage(extension_id: &str) -> Option<PathBuf> {
    for root in vscode_global_storage_candidates() {
        let candidate = root.join(extension_id);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Canonical (possibly non-existent) globalStorage path for an
/// extension. Used as a fallback `config_path` when we haven't found
/// a real one — keeps the path shape consistent with what
/// `scan_task_dirs` expects, so `is_installed()` reports false
/// instead of misdirecting session discovery into a non-storage dir
/// like `~/.vscode/extensions/<id>`.
pub fn canonical_extension_storage(extension_id: &str) -> PathBuf {
    vscode_global_storage_candidates()
        .into_iter()
        .next()
        .map(|root| root.join(extension_id))
        .unwrap_or_else(|| PathBuf::from(extension_id))
}

/// Walk `<extension_storage>/tasks/<task_id>/` and return every task
/// directory that contains an api_conversation_history file.
pub fn scan_task_dirs(storage: &Path) -> Result<Vec<PathBuf>> {
    let tasks_root = storage.join("tasks");
    if !tasks_root.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&tasks_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Either .json or .jsonl flavour is acceptable.
            let json = path.join("api_conversation_history.json");
            let jsonl = path.join("api_conversation_history.jsonl");
            if json.exists() || jsonl.exists() {
                out.push(path);
            }
        }
    }
    Ok(out)
}

/// Parse one task directory into a Session. The task ID (folder name)
/// is used as the session ID when it parses as a UUID — Cline writes
/// timestamp-prefixed names by default, so this is best-effort.
pub fn parse_task_dir(task_dir: &Path, agent_id: Uuid, agent_type: AgentType) -> Result<Session> {
    let mut session = Session::new(agent_id, agent_type);

    // Try .json then .jsonl
    let json = task_dir.join("api_conversation_history.json");
    let jsonl = task_dir.join("api_conversation_history.jsonl");

    let (content, is_jsonl) = if json.exists() {
        (std::fs::read_to_string(&json)?, false)
    } else if jsonl.exists() {
        (std::fs::read_to_string(&jsonl)?, true)
    } else {
        return Err(RimuruError::Adapter(format!(
            "no api_conversation_history file in {}",
            task_dir.display()
        )));
    };

    // Folder name as session id (best effort)
    if let Some(name) = task_dir.file_name().and_then(|n| n.to_str())
        && let Ok(parsed) = uuid::Uuid::parse_str(name)
    {
        session.id = parsed;
    }

    // Folder mtime as start time
    if let Ok(meta) = std::fs::metadata(task_dir)
        && let Ok(created) = meta.created()
    {
        let dt: chrono::DateTime<Utc> = created.into();
        session.started_at = dt;
    }

    let mut total_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut total_cache_write: u64 = 0;
    let mut total_cache_read: u64 = 0;
    let mut msg_count: u64 = 0;
    let mut last_model: Option<String> = None;

    let consume = |entry: &Value,
                   total_input: &mut u64,
                   total_output: &mut u64,
                   total_cache_write: &mut u64,
                   total_cache_read: &mut u64,
                   msg_count: &mut u64,
                   last_model: &mut Option<String>| {
        *msg_count += 1;
        if let Some(usage) = entry.get("usage") {
            *total_input += usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            *total_output += usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            *total_cache_write += usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            *total_cache_read += usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }
        if let Some(m) = entry.get("model").and_then(|v| v.as_str()) {
            *last_model = Some(m.to_string());
        }
    };

    if is_jsonl {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<Value>(line) {
                consume(
                    &entry,
                    &mut total_input,
                    &mut total_output,
                    &mut total_cache_write,
                    &mut total_cache_read,
                    &mut msg_count,
                    &mut last_model,
                );
            }
        }
    } else {
        match serde_json::from_str::<Value>(&content) {
            Ok(Value::Array(entries)) => {
                for entry in entries {
                    consume(
                        &entry,
                        &mut total_input,
                        &mut total_output,
                        &mut total_cache_write,
                        &mut total_cache_read,
                        &mut msg_count,
                        &mut last_model,
                    );
                }
            }
            Ok(_) | Err(_) => {
                warn!(
                    "Cline/Roo task at {} has unexpected JSON shape, skipping usage extraction",
                    task_dir.display()
                );
            }
        }
    }

    session.messages = msg_count;
    session.input_tokens = total_input + total_cache_write + total_cache_read;
    session.output_tokens = total_output;
    session.total_tokens = session.input_tokens + session.output_tokens;
    session.model = last_model;

    Ok(session)
}
