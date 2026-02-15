use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

fn get_state_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|dir| dir.join("window-state.json"))
}

pub fn load_window_state<R: Runtime>(app: &AppHandle<R>) -> WindowState {
    get_state_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_window_state<R: Runtime>(app: &AppHandle<R>, state: &WindowState) {
    if let Some(path) = get_state_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(state) {
            let _ = fs::write(path, content);
        }
    }
}

pub fn restore_window_state<R: Runtime>(window: &WebviewWindow<R>) {
    let app = window.app_handle();
    let state = load_window_state(app);

    let _ = window.set_size(PhysicalSize::new(state.width, state.height));

    if let (Some(x), Some(y)) = (state.x, state.y) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = window.center();
    }

    if state.maximized {
        let _ = window.maximize();
    }
}

pub fn capture_window_state<R: Runtime>(window: &WebviewWindow<R>) -> Option<WindowState> {
    let size = window.outer_size().ok()?;
    let position = window.outer_position().ok()?;
    let maximized = window.is_maximized().ok()?;

    Some(WindowState {
        width: size.width,
        height: size.height,
        x: Some(position.x),
        y: Some(position.y),
        maximized,
    })
}

pub fn setup_window_state_persistence<R: Runtime>(window: &WebviewWindow<R>) {
    let window_clone = window.clone();

    window.on_window_event(move |event| match event {
        tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed => {
            if let Some(state) = capture_window_state(&window_clone) {
                save_window_state(window_clone.app_handle(), &state);
            }
        }
        tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) => {
            if let Some(state) = capture_window_state(&window_clone) {
                save_window_state(window_clone.app_handle(), &state);
            }
        }
        _ => {}
    });
}
