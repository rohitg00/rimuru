use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use parking_lot::{Mutex, RwLock};
use portable_pty::Child;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::session::PtySessionStatus;

#[derive(Serialize, Clone)]
struct PtyOutputPayload {
    session_id: String,
    data: String,
}

#[derive(Serialize, Clone)]
struct PtyExitPayload {
    session_id: String,
    exit_code: Option<u32>,
    success: bool,
}

pub fn spawn_reader_thread(
    mut master_reader: Box<dyn Read + Send>,
    session_id: String,
    app_handle: AppHandle,
) {
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match master_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                    let payload = PtyOutputPayload {
                        session_id: session_id.clone(),
                        data: encoded,
                    };
                    let event_name = format!("pty-output-{}", session_id);
                    let _ = app_handle.emit(&event_name, payload);
                }
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_wait_thread(
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
    session_id: String,
    app_handle: AppHandle,
    status: Arc<RwLock<PtySessionStatus>>,
) {
    std::thread::spawn(move || loop {
        let result = child.lock().try_wait();
        match result {
            Ok(Some(exit_status)) => {
                let success = exit_status.success();
                let exit_code = None::<u32>;
                if success {
                    *status.write() = PtySessionStatus::Completed;
                } else {
                    *status.write() = PtySessionStatus::Failed;
                }
                let payload = PtyExitPayload {
                    session_id,
                    exit_code,
                    success,
                };
                let _ = app_handle.emit("pty-exit", payload);
                return;
            }
            Ok(None) => {}
            Err(_) => {
                *status.write() = PtySessionStatus::Failed;
                let payload = PtyExitPayload {
                    session_id,
                    exit_code: None,
                    success: false,
                };
                let _ = app_handle.emit("pty-exit", payload);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    });
}
