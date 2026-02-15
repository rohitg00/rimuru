use tauri::State;

use crate::remote::server::{generate_qr_svg, RemoteServer};
use crate::remote::RemoteStatus;
use crate::state::AppState;

#[tauri::command]
pub async fn start_remote_server(
    state: State<'_, AppState>,
    port: Option<u16>,
) -> Result<RemoteStatus, String> {
    let port = port.unwrap_or(3847);

    {
        let existing = state.remote_server.read().await;
        if existing.is_some() {
            return Err("Remote server is already running".to_string());
        }
    }

    let server = RemoteServer::start(port, state.pty_manager.clone()).await?;
    let url = server.url.clone();
    let qr_svg = generate_qr_svg(&url);

    *state.remote_server.write().await = Some(server);

    Ok(RemoteStatus {
        running: true,
        url: Some(url),
        qr_svg: Some(qr_svg),
    })
}

#[tauri::command]
pub async fn stop_remote_server(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.remote_server.write().await;
    match guard.as_mut() {
        Some(server) => {
            server.stop();
            *guard = None;
            Ok(())
        }
        None => Err("Remote server is not running".to_string()),
    }
}

#[tauri::command]
pub async fn get_remote_status(state: State<'_, AppState>) -> Result<RemoteStatus, String> {
    let guard = state.remote_server.read().await;
    match guard.as_ref() {
        Some(server) => {
            let url = server.url.clone();
            let qr_svg = generate_qr_svg(&url);
            Ok(RemoteStatus {
                running: true,
                url: Some(url),
                qr_svg: Some(qr_svg),
            })
        }
        None => Ok(RemoteStatus {
            running: false,
            url: None,
            qr_svg: None,
        }),
    }
}
