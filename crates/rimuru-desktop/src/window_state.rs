use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

pub fn save_window_state<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let Ok(position) = window.outer_position() else {
        return;
    };
    let Ok(size) = window.outer_size() else {
        return;
    };
    let maximized = window.is_maximized().unwrap_or(false);

    let state = WindowState {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
        maximized,
    };

    let Some(home) = dirs::home_dir() else {
        return;
    };
    let dir = home.join(".rimuru");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("window_state.json");

    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = std::fs::write(path, json);
    }
}

pub fn restore_window_state<R: Runtime>(app: &AppHandle<R>) {
    let Some(home) = dirs::home_dir() else {
        return;
    };
    let path = home.join(".rimuru").join("window_state.json");

    let Ok(data) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(state) = serde_json::from_str::<WindowState>(&data) else {
        return;
    };

    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
        x: state.x,
        y: state.y,
    }));
    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
        width: state.width,
        height: state.height,
    }));

    if state.maximized {
        let _ = window.maximize();
    }
}
