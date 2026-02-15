use axum::{
    extract::Path,
    extract::State as AxumState,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::pty::manager::PtySessionManager;
use crate::pty::session::PtySessionInfo;

pub struct RemoteServer {
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    pub url: String,
    pub token: String,
}

#[derive(Clone)]
struct AppState {
    pty_manager: Arc<PtySessionManager>,
    token: String,
}

const MAX_MESSAGE_LEN: usize = 4096;

impl RemoteServer {
    pub async fn start(port: u16, pty_manager: Arc<PtySessionManager>) -> Result<Self, String> {
        let token = uuid::Uuid::new_v4().to_string();

        let state = AppState {
            pty_manager,
            token: token.clone(),
        };

        let local_ip = local_ip_address::local_ip().map_err(|e| e.to_string())?;
        let url = format!("http://{}:{}?token={}", local_ip, port, token);

        let origin = format!("http://{}:{}", local_ip, port);
        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::exact(
                origin.parse().map_err(|_| "Invalid origin".to_string())?,
            ))
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ]);

        let app = Router::new()
            .route("/", get(serve_mobile_page))
            .route("/api/sessions", get(list_sessions))
            .route("/api/sessions/{id}/message", post(send_message))
            .route("/api/sessions/{id}/terminate", post(terminate_session))
            .route("/api/status", get(app_status))
            .layer(cors)
            .with_state(state);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .map_err(|e| e.to_string())?;

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        Ok(Self {
            shutdown_tx: Some(shutdown_tx),
            url,
            token,
        })
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

fn verify_token(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let provided = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    match provided {
        Some(t) if t == state.token => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn serve_mobile_page() -> axum::response::Html<&'static str> {
    axum::response::Html(MOBILE_HTML)
}

async fn list_sessions(
    headers: HeaderMap,
    AxumState(state): AxumState<AppState>,
) -> Result<Json<Vec<PtySessionInfo>>, StatusCode> {
    verify_token(&headers, &state)?;
    Ok(Json(state.pty_manager.list_active()))
}

#[derive(serde::Deserialize)]
struct MessageBody {
    content: String,
}

async fn send_message(
    headers: HeaderMap,
    AxumState(state): AxumState<AppState>,
    Path(id): Path<String>,
    Json(body): Json<MessageBody>,
) -> Result<Json<()>, (StatusCode, String)> {
    verify_token(&headers, &state).map_err(|s| (s, "Unauthorized".to_string()))?;

    if body.content.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Message cannot be empty".to_string(),
        ));
    }
    if body.content.len() > MAX_MESSAGE_LEN {
        return Err((StatusCode::BAD_REQUEST, "Message too long".to_string()));
    }
    if body
        .content
        .bytes()
        .any(|b| b < 0x20 && b != b'\t' && b != b'\n')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid characters in message".to_string(),
        ));
    }

    let data = format!("{}\n", body.content);
    state
        .pty_manager
        .write(&id, data.as_bytes())
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(()))
}

async fn terminate_session(
    headers: HeaderMap,
    AxumState(state): AxumState<AppState>,
    Path(id): Path<String>,
) -> Result<Json<()>, (StatusCode, String)> {
    verify_token(&headers, &state).map_err(|s| (s, "Unauthorized".to_string()))?;
    state
        .pty_manager
        .terminate(&id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(()))
}

async fn app_status() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "app": "Rimuru" }))
}

pub fn generate_qr_svg(url: &str) -> String {
    use qrcode::render::svg;
    use qrcode::QrCode;

    let code = match QrCode::new(url.as_bytes()) {
        Ok(c) => c,
        Err(_) => return String::from("<svg></svg>"),
    };
    code.render::<svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#c0caf5"))
        .light_color(svg::Color("#1a1b26"))
        .build()
}

const MOBILE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no">
<title>Rimuru Remote</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,sans-serif;background:#1a1b26;color:#c0caf5;min-height:100vh}
.header{padding:16px;background:#16161e;border-bottom:1px solid #292e42;display:flex;align-items:center;gap:8px}
.header h1{font-size:18px;font-weight:600}
.dot{width:8px;height:8px;border-radius:50%;background:#9ece6a}
.sessions{padding:12px}
.session{background:#1f2335;border:1px solid #292e42;border-radius:8px;padding:12px;margin-bottom:8px}
.session-header{display:flex;justify-content:space-between;align-items:center;margin-bottom:8px}
.session-name{font-weight:500;font-size:14px}
.badge{font-size:10px;padding:2px 8px;border-radius:12px;font-weight:600}
.badge-running{background:rgba(158,206,106,0.2);color:#9ece6a}
.badge-completed{background:rgba(122,162,247,0.2);color:#7aa2f7}
.badge-failed{background:rgba(247,118,142,0.2);color:#f7768e}
.session-meta{font-size:12px;color:#565f89;margin-bottom:8px}
.input-row{display:flex;gap:8px}
.input-row input{flex:1;background:#16161e;border:1px solid #292e42;border-radius:6px;padding:8px;color:#c0caf5;font-size:14px}
.input-row input:focus{outline:none;border-color:#7aa2f7}
.btn{padding:8px 16px;border-radius:6px;border:none;font-size:13px;font-weight:500;cursor:pointer}
.btn-send{background:#7aa2f7;color:#1a1b26}
.btn-terminate{background:#f7768e;color:#1a1b26;width:100%;margin-top:8px}
.empty{text-align:center;padding:48px 24px;color:#565f89}
.auth-error{text-align:center;padding:48px 24px;color:#f7768e}
</style>
</head>
<body>
<div class="header"><div class="dot"></div><h1>Rimuru Remote</h1></div>
<div class="sessions" id="sessions"><div class="empty">Loading sessions...</div></div>
<script>
function esc(s){var d=document.createElement('div');d.textContent=String(s);return d.innerHTML}
function getToken(){return new URLSearchParams(window.location.search).get('token')||''}
function authHeaders(){return{'Authorization':'Bearer '+getToken(),'Content-Type':'application/json'}}
async function load(){
  try{
    const r=await fetch('/api/sessions',{headers:authHeaders()});
    if(r.status===401){document.getElementById('sessions').innerHTML='<div class="auth-error">Unauthorized. Scan QR code again.</div>';return}
    const sessions=await r.json();
    const el=document.getElementById('sessions');
    if(!sessions.length){el.innerHTML='<div class="empty">No active sessions</div>';return}
    el.innerHTML=sessions.map(s=>`
      <div class="session">
        <div class="session-header">
          <span class="session-name">${esc(s.agent_name)}</span>
          <span class="badge badge-${esc(s.status.toLowerCase())}">${esc(s.status)}</span>
        </div>
        <div class="session-meta">$${esc(s.cumulative_cost_usd.toFixed(4))} &middot; ${esc(s.token_count)} tokens</div>
        <div class="input-row">
          <input id="msg-${esc(s.id)}" placeholder="Send message..." onkeydown="if(event.key==='Enter')send('${esc(s.id)}')">
          <button class="btn btn-send" onclick="send('${esc(s.id)}')">Send</button>
        </div>
        ${s.status==='Running'?`<button class="btn btn-terminate" onclick="terminate('${esc(s.id)}')">Terminate</button>`:''}
      </div>
    `).join('');
  }catch(e){document.getElementById('sessions').innerHTML='<div class="empty">Connection error</div>'}
}
async function send(id){
  const input=document.getElementById('msg-'+id);
  if(!input||!input.value.trim())return;
  await fetch('/api/sessions/'+encodeURIComponent(id)+'/message',{method:'POST',headers:authHeaders(),body:JSON.stringify({content:input.value})});
  input.value='';
}
async function terminate(id){
  if(!confirm('Terminate this session?'))return;
  await fetch('/api/sessions/'+encodeURIComponent(id)+'/terminate',{method:'POST',headers:authHeaders()});
  load();
}
load();setInterval(load,3000);
</script>
</body>
</html>"#;
