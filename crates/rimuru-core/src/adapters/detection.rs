use std::path::PathBuf;

use tracing::debug;

use crate::models::AgentType;

struct AgentDetector {
    agent_type: AgentType,
    paths: Vec<PathBuf>,
}

fn build_detectors() -> Vec<AgentDetector> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

    #[cfg(target_os = "macos")]
    let cursor_path = home.join("Library/Application Support/Cursor");
    #[cfg(target_os = "linux")]
    let cursor_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Cursor");
    #[cfg(target_os = "windows")]
    let cursor_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\temp"))
        .join("Cursor");
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let cursor_path = home.join(".cursor");

    let vscode_extensions = home.join(".vscode/extensions");

    #[cfg(target_os = "macos")]
    let vscode_config = home.join("Library/Application Support/Code");
    #[cfg(target_os = "linux")]
    let vscode_config = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Code");
    #[cfg(target_os = "windows")]
    let vscode_config = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\temp"))
        .join("Code");
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let vscode_config = home.join(".vscode");

    let cline_storage = super::cline_base::find_extension_storage("saoudrizwan.claude-dev");
    let roo_storage = super::cline_base::find_extension_storage("rooveterinaryinc.roo-cline");

    vec![
        AgentDetector {
            agent_type: AgentType::ClaudeCode,
            paths: vec![home.join(".claude")],
        },
        AgentDetector {
            agent_type: AgentType::Cursor,
            paths: vec![cursor_path],
        },
        AgentDetector {
            agent_type: AgentType::Copilot,
            paths: vec![
                vscode_extensions,
                vscode_config,
                home.join(".config/github-copilot"),
            ],
        },
        AgentDetector {
            agent_type: AgentType::Codex,
            paths: vec![home.join(".config/codex")],
        },
        AgentDetector {
            agent_type: AgentType::Goose,
            paths: vec![home.join(".config/goose")],
        },
        AgentDetector {
            agent_type: AgentType::OpenCode,
            paths: vec![home.join(".opencode")],
        },
        AgentDetector {
            agent_type: AgentType::GeminiCli,
            paths: vec![home.join(".gemini"), home.join(".config/gemini")],
        },
        AgentDetector {
            agent_type: AgentType::Windsurf,
            paths: vec![
                home.join(".windsurf"),
                home.join(".codeium"),
                home.join(".config/windsurf"),
            ],
        },
        AgentDetector {
            agent_type: AgentType::Cline,
            paths: cline_storage
                .map(|p| vec![p])
                .unwrap_or_else(|| vec![home.join(".vscode/extensions/saoudrizwan.claude-dev")]),
        },
        AgentDetector {
            agent_type: AgentType::Roo,
            paths: roo_storage.map(|p| vec![p]).unwrap_or_else(|| {
                vec![home.join(".vscode/extensions/rooveterinaryinc.roo-cline")]
            }),
        },
        AgentDetector {
            agent_type: AgentType::Amp,
            paths: vec![home.join(".amp"), home.join(".config/amp")],
        },
        AgentDetector {
            agent_type: AgentType::Kiro,
            paths: vec![
                home.join(".kiro"),
                home.join(".config/kiro"),
                home.join(".aws/kiro"),
            ],
        },
    ]
}

fn copilot_extension_exists(extensions_dir: &PathBuf) -> bool {
    if !extensions_dir.exists() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(extensions_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("github.copilot-") {
                return true;
            }
        }
    }
    false
}

pub fn detect_installed_agents() -> Vec<AgentType> {
    let detectors = build_detectors();
    let mut installed = Vec::new();

    for detector in &detectors {
        let found = match detector.agent_type {
            AgentType::Copilot => detector.paths.iter().any(|p| {
                if p.ends_with("extensions") {
                    copilot_extension_exists(p)
                } else {
                    p.exists()
                }
            }),
            _ => detector.paths.iter().any(|p| p.exists()),
        };

        if found {
            debug!("Detected installed agent: {}", detector.agent_type);
            installed.push(detector.agent_type);
        }
    }

    installed
}

pub fn detect_agent_config_path(agent_type: AgentType) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    let path = match agent_type {
        AgentType::ClaudeCode => home.join(".claude"),
        AgentType::Cursor => {
            #[cfg(target_os = "macos")]
            {
                home.join("Library/Application Support/Cursor")
            }
            #[cfg(target_os = "linux")]
            {
                dirs::config_dir()?.join("Cursor")
            }
            #[cfg(target_os = "windows")]
            {
                dirs::config_dir()?.join("Cursor")
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
            {
                home.join(".cursor")
            }
        }
        AgentType::Copilot => home.join(".config/github-copilot"),
        AgentType::Codex => home.join(".config/codex"),
        AgentType::Goose => home.join(".config/goose"),
        AgentType::OpenCode => home.join(".opencode"),
        AgentType::GeminiCli => {
            let primary = home.join(".gemini");
            if primary.exists() {
                primary
            } else {
                home.join(".config/gemini")
            }
        }
        AgentType::Windsurf => {
            for cand in [
                home.join(".windsurf"),
                home.join(".codeium"),
                home.join(".config/windsurf"),
            ] {
                if cand.exists() {
                    return Some(cand);
                }
            }
            return None;
        }
        AgentType::Cline => {
            return super::cline_base::find_extension_storage("saoudrizwan.claude-dev");
        }
        AgentType::Roo => {
            return super::cline_base::find_extension_storage("rooveterinaryinc.roo-cline");
        }
        AgentType::Amp => {
            for cand in [home.join(".amp"), home.join(".config/amp")] {
                if cand.exists() {
                    return Some(cand);
                }
            }
            return None;
        }
        AgentType::Kiro => {
            for cand in [
                home.join(".kiro"),
                home.join(".config/kiro"),
                home.join(".aws/kiro"),
            ] {
                if cand.exists() {
                    return Some(cand);
                }
            }
            return None;
        }
    };

    if path.exists() { Some(path) } else { None }
}

pub fn detect_all_with_paths() -> Vec<(AgentType, PathBuf)> {
    AgentType::all()
        .iter()
        .filter_map(|&at| detect_agent_config_path(at).map(|p| (at, p)))
        .collect()
}
