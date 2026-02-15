use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use tauri::AppHandle;

use super::agents::get_agent_config;
use super::reader::{spawn_reader_thread, spawn_wait_thread};
use super::session::{PtySessionInfo, PtySessionStatus};

const ALLOWED_BINARIES: &[&str] = &[
    "claude", "codex", "goose", "opencode", "cursor", "copilot", "bash", "zsh", "sh", "fish",
    "node", "python", "python3",
];

const MAX_SESSIONS: usize = 20;

fn is_allowed_binary(binary: &str) -> bool {
    let base = std::path::Path::new(binary)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(binary);
    ALLOWED_BINARIES.contains(&base)
}

struct PtySessionInner {
    info: PtySessionInfo,
    writer: Mutex<Box<dyn Write + Send>>,
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    status: Arc<RwLock<PtySessionStatus>>,
}

pub struct PtySessionManager {
    sessions: RwLock<HashMap<String, PtySessionInner>>,
    app_handle: RwLock<Option<AppHandle>>,
}

impl PtySessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            app_handle: RwLock::new(None),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.write() = Some(handle);
    }

    pub fn launch(
        &self,
        agent_type: String,
        executable: Option<String>,
        args: Vec<String>,
        cwd: String,
        cols: u16,
        rows: u16,
        initial_prompt: Option<String>,
    ) -> Result<String, String> {
        if self.sessions.read().len() >= MAX_SESSIONS {
            return Err("Maximum session limit reached (20)".to_string());
        }

        let app_handle = self
            .app_handle
            .read()
            .clone()
            .ok_or_else(|| "App handle not set".to_string())?;

        let has_custom_executable = executable.is_some();
        let (binary, final_args) = if let Some(exec) = executable {
            if !is_allowed_binary(&exec) {
                return Err(format!(
                    "Executable '{}' is not in the allowlist. Allowed: {:?}",
                    exec, ALLOWED_BINARIES
                ));
            }
            (exec, args)
        } else {
            let config = get_agent_config(&agent_type)
                .ok_or_else(|| format!("Unknown agent type: {}", agent_type))?;
            let mut combined = config.default_args;
            if let (Some(flag), Some(prompt)) = (&config.prompt_flag, &initial_prompt) {
                combined.push(flag.clone());
                combined.push(prompt.clone());
            }
            combined.extend(args);
            (config.binary, combined)
        };

        let cwd = if cwd.starts_with('~') {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            cwd.replacen('~', &home, 1)
        } else if cwd.is_empty() {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        } else {
            cwd
        };

        tracing::info!(
            "Launching PTY session: binary={}, cwd={}, agent_type={}",
            binary,
            cwd,
            agent_type
        );

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(&binary);
        cmd.args(&final_args);
        cmd.cwd(&cwd);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let pid = child.process_id();

        let master_arc = Arc::new(Mutex::new(pair.master));
        let child_arc = Arc::new(Mutex::new(child));
        let status_arc = Arc::new(RwLock::new(PtySessionStatus::Running));

        spawn_reader_thread(reader, session_id.clone(), app_handle.clone());
        spawn_wait_thread(
            child_arc.clone(),
            session_id.clone(),
            app_handle,
            status_arc.clone(),
        );

        if has_custom_executable {
            if let Some(prompt) = &initial_prompt {
                let mut w = writer;
                let data = format!("{}\n", prompt);
                w.write_all(data.as_bytes())
                    .map_err(|e| format!("Failed to write initial prompt: {}", e))?;
                w.flush()
                    .map_err(|e| format!("Failed to flush initial prompt: {}", e))?;

                let info = PtySessionInfo {
                    id: session_id.clone(),
                    agent_type: agent_type.clone(),
                    agent_name: binary.clone(),
                    working_dir: cwd,
                    started_at: chrono::Utc::now().to_rfc3339(),
                    status: PtySessionStatus::Running,
                    pid,
                    cumulative_cost_usd: 0.0,
                    token_count: 0,
                };

                let inner = PtySessionInner {
                    info,
                    writer: Mutex::new(w),
                    master: master_arc,
                    child: child_arc,
                    status: status_arc,
                };

                self.sessions.write().insert(session_id.clone(), inner);
                return Ok(session_id);
            }
        }

        let info = PtySessionInfo {
            id: session_id.clone(),
            agent_type: agent_type.clone(),
            agent_name: binary.clone(),
            working_dir: cwd,
            started_at: chrono::Utc::now().to_rfc3339(),
            status: PtySessionStatus::Running,
            pid,
            cumulative_cost_usd: 0.0,
            token_count: 0,
        };

        let inner = PtySessionInner {
            info,
            writer: Mutex::new(writer),
            master: master_arc,
            child: child_arc,
            status: status_arc,
        };

        self.sessions.write().insert(session_id.clone(), inner);
        Ok(session_id)
    }

    pub fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.read();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        let mut writer = session.writer.lock();
        writer
            .write_all(data)
            .map_err(|e| format!("Write failed: {}", e))?;
        writer.flush().map_err(|e| format!("Flush failed: {}", e))?;
        Ok(())
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.read();
        let session = sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;
        let master = session.master.lock();
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize: {}", e))?;
        Ok(())
    }

    pub fn terminate(&self, session_id: &str) -> Result<(), String> {
        {
            let sessions = self.sessions.read();
            let session = sessions
                .get(session_id)
                .ok_or_else(|| format!("Session not found: {}", session_id))?;
            session
                .child
                .lock()
                .kill()
                .map_err(|e| format!("Failed to kill process: {}", e))?;
            *session.status.write() = PtySessionStatus::Terminated;
        }
        self.cleanup_finished();
        Ok(())
    }

    fn cleanup_finished(&self) {
        let mut sessions = self.sessions.write();
        sessions.retain(|_, inner| {
            let status = inner.status.read().clone();
            matches!(status, PtySessionStatus::Running)
        });
    }

    pub fn list_active(&self) -> Vec<PtySessionInfo> {
        self.sessions
            .read()
            .values()
            .map(|s| {
                let mut info = s.info.clone();
                info.status = s.status.read().clone();
                info
            })
            .collect()
    }

    pub fn get_info(&self, session_id: &str) -> Option<PtySessionInfo> {
        self.sessions.read().get(session_id).map(|s| {
            let mut info = s.info.clone();
            info.status = s.status.read().clone();
            info
        })
    }
}
