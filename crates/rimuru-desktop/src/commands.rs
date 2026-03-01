#[tauri::command]
pub fn get_port() -> u16 {
    std::env::var("RIMURU_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3100)
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
