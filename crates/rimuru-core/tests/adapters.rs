//! Integration tests for agent adapters.
//!
//! Each adapter is tested against a synthetic config/session directory
//! built in a tempdir. The public surface we exercise is whatever the
//! adapter already exposes on the `AgentAdapter` trait plus any helpers
//! the adapter has made public (e.g. `parse_session_jsonl_full`).

use std::io::Write;
use std::path::{Path, PathBuf};

use rimuru_core::adapters::{AgentAdapter, ClaudeCodeAdapter};

fn write_jsonl(path: &Path, entries: &[serde_json::Value]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(path).unwrap();
    for e in entries {
        writeln!(f, "{}", serde_json::to_string(e).unwrap()).unwrap();
    }
}

fn claude_fixture(dir: &Path, session_id: &str, model: &str, input: u64, output: u64) -> PathBuf {
    let projects = dir.join(".claude").join("projects").join("proj_x");
    let path = projects.join(format!("{session_id}.jsonl"));
    write_jsonl(
        &path,
        &[
            serde_json::json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "sessionId": session_id,
                "message": {
                    "role": "user",
                    "content": [{"type": "text", "text": "hello"}],
                }
            }),
            serde_json::json!({
                "timestamp": "2026-01-01T00:00:05Z",
                "message": {
                    "role": "assistant",
                    "model": model,
                    "usage": {
                        "input_tokens": input,
                        "output_tokens": output,
                        "cache_read_input_tokens": 10,
                        "cache_creation_input_tokens": 5,
                    },
                    "content": [{"type": "text", "text": "world"}],
                }
            }),
        ],
    );
    path
}

#[test]
fn claude_code_adapter_parses_tokens_and_cost() {
    let dir = tempfile::tempdir().unwrap();
    let path = claude_fixture(
        dir.path(),
        "33333333-3333-3333-3333-333333333333",
        "claude-sonnet-4-5",
        200,
        100,
    );

    let mut adapter = ClaudeCodeAdapter::new();
    adapter.set_config_path_for_bench(dir.path().join(".claude"));

    let (session, breakdown) = adapter.parse_session_jsonl_full(&path).unwrap();
    assert_eq!(session.input_tokens, 200);
    assert_eq!(session.output_tokens, 100);
    assert!(session.total_cost > 0.0, "should have non-zero cost");
    assert_eq!(session.model.as_deref(), Some("claude-sonnet-4-5"));
    assert_eq!(breakdown.cache_read_tokens, 10);
    assert_eq!(breakdown.cache_write_tokens, 5);
}

#[test]
fn claude_code_adapter_handles_tool_use_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let session_id = "44444444-4444-4444-4444-444444444444";
    let projects = dir.path().join(".claude").join("projects").join("p");
    let path = projects.join(format!("{session_id}.jsonl"));
    write_jsonl(
        &path,
        &[
            serde_json::json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "sessionId": session_id,
                "message": {
                    "role": "assistant",
                    "model": "claude-sonnet-4-5",
                    "usage": {"input_tokens": 50, "output_tokens": 30},
                    "content": [
                        {"type": "tool_use", "id": "tu1", "name": "Bash", "input": {"cmd": "ls"}},
                    ],
                }
            }),
            serde_json::json!({
                "timestamp": "2026-01-01T00:00:05Z",
                "message": {
                    "role": "user",
                    "content": [
                        {"type": "tool_result", "tool_use_id": "tu1", "content": "file1.txt\nfile2.txt"},
                    ],
                }
            }),
        ],
    );

    let mut adapter = ClaudeCodeAdapter::new();
    adapter.set_config_path_for_bench(dir.path().join(".claude"));
    let (_session, breakdown) = adapter.parse_session_jsonl_full(&path).unwrap();
    assert!(
        breakdown.bash_output_tokens > 0,
        "Bash tool output should be classified: {breakdown:?}"
    );
}

#[test]
fn claude_code_adapter_is_installed_true_when_dir_exists() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    let mut adapter = ClaudeCodeAdapter::new();
    adapter.set_config_path_for_bench(dir.path().join(".claude"));
    assert!(adapter.is_installed());
}

#[test]
fn claude_code_adapter_is_installed_false_when_dir_missing() {
    let dir = tempfile::tempdir().unwrap();
    let mut adapter = ClaudeCodeAdapter::new();
    adapter.set_config_path_for_bench(dir.path().join(".claude_missing"));
    assert!(!adapter.is_installed());
}

#[tokio::test]
async fn claude_code_adapter_scans_session_files() {
    let dir = tempfile::tempdir().unwrap();
    let _ = claude_fixture(
        dir.path(),
        "55555555-5555-5555-5555-555555555555",
        "claude-sonnet-4-5",
        10,
        20,
    );

    let mut adapter = ClaudeCodeAdapter::new();
    adapter.set_config_path_for_bench(dir.path().join(".claude"));
    let sessions = adapter.get_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].input_tokens, 10);
    assert_eq!(sessions[0].output_tokens, 20);
}
