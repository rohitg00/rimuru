use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

mod client;
mod state;
mod theme;
mod ui;
mod widgets;

use client::ApiClient;
use state::{App, View};

#[cfg(test)]
mod smoke_tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn sample_sessions() -> serde_json::Value {
        serde_json::json!({
            "sessions": [
                {
                    "id": "49316303-4e10-4465-bad9-47d2b414f6ab",
                    "agent_id": "5675d736-7f3c",
                    "agent_type": "claude_code",
                    "status": "active",
                    "project_path": "/Users/rohit/rimuru-migrate",
                    "model": "claude-opus-4-7",
                    "total_tokens": 52581,
                    "input_tokens": 6,
                    "output_tokens": 81,
                    "total_cost": 0.725571,
                    "messages": 2,
                    "started_at": "2026-04-18T23:39:38.318Z",
                    "ended_at": null,
                    "metadata": {
                        "turns": [
                            {"cache_read": 15354, "cache_write": 37140, "input_tokens": 6, "output_tokens": 81},
                            {"cache_read": 18000, "cache_write": 12000, "input_tokens": 4, "output_tokens": 120}
                        ]
                    }
                }
            ]
        })
    }

    fn populate(app: &mut App) {
        app.connected = true;
        if let serde_json::Value::Object(_) = sample_sessions() {
            let v = sample_sessions();
            let arr = v.get("sessions").and_then(|x| x.as_array()).cloned().unwrap_or_default();
            app.ingest_sessions_public(arr);
        }
        app.budget_status = serde_json::json!({
            "daily_limit": 50.0,
            "daily_spent": 12.5,
            "monthly_limit": 500.0,
            "monthly_spent": 180.2,
            "burn_rate_daily": 6.0,
            "projected_monthly": 180.0,
            "status": "ok",
            "action_on_exceed": "alert",
            "alert_threshold": 0.8,
        });
        app.context_utilization = serde_json::json!({
            "utilizations": [
                {
                    "session_id": "49316303-4e10-4465-bad9-47d2b414f6ab",
                    "utilization_percent": 26.25,
                    "tokens_used": 52500,
                    "context_window_size": 200000,
                    "model": "claude-opus-4-7"
                }
            ]
        });
        for _ in 0..30 {
            app.push_token_rate_for_test(rand_rate());
        }
    }

    fn rand_rate() -> f64 {
        use std::time::SystemTime;
        let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos();
        ((t % 1000) as f64 / 1000.0) * 500.0
    }

    #[test]
    fn smoke_home_renders() {
        let backend = TestBackend::new(140, 45);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        populate(&mut app);
        terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
    }

    #[test]
    fn smoke_all_views_render() {
        let backend = TestBackend::new(140, 45);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        populate(&mut app);
        for view in [
            View::Home, View::Costs, View::Budget, View::Models, View::Advisor,
            View::Context, View::McpProxy, View::Hooks, View::Plugins, View::Mcp, View::Metrics,
        ] {
            app.set_view(view);
            terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
        }
    }

    #[test]
    fn smoke_empty_renders() {
        let backend = TestBackend::new(140, 45);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
    }

    fn buffer_text(term: &Terminal<TestBackend>) -> String {
        let buf = term.backend().buffer();
        let mut out = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                out.push_str(buf[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn live_render_against_backend() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = ApiClient::from_env();
            let mut app = App::new();
            app.fetch(&client).await;
            if !app.connected {
                eprintln!("skip: rimuru backend not reachable");
                return;
            }
            app.fetch(&client).await;
            let backend = TestBackend::new(180, 50);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
            let dump = buffer_text(&terminal);
            if std::env::var("RIMURU_DUMP_HOME").is_ok() {
                println!("\n===HOME SCREEN DUMP===\n{}\n======================", dump);
            }
            assert!(dump.contains("rimuru v"));
            assert!(dump.contains("sessions"));

            for view in [View::Context, View::Budget, View::Metrics, View::Models, View::Advisor] {
                app.set_view(view);
                app.fetch(&client).await;
                terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
            }
        });
    }

    #[test]
    fn home_contains_expected_labels() {
        let backend = TestBackend::new(160, 48);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();
        populate(&mut app);
        terminal.draw(|f| ui::draw(f, &mut app)).unwrap();
        let text = buffer_text(&terminal);
        for needle in ["rimuru v", "context", "quota", "tokens", "sessions", "claude-opus-4-7"] {
            assert!(text.contains(needle), "missing `{}` in rendered home\n{}", needle, text);
        }
    }
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = rt.block_on(run_app(&mut terminal));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let client = ApiClient::from_env();
    let mut app = App::new();
    let mut last_fetch = Instant::now() - Duration::from_secs(10);
    let fetch_interval = Duration::from_secs(3);

    loop {
        if last_fetch.elapsed() >= fetch_interval {
            app.fetch(&client).await;
            last_fetch = Instant::now();
        }

        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Esc | KeyCode::Char('h') => {
                    if app.view != View::Home {
                        app.set_view(View::Home);
                    }
                }
                KeyCode::Char('t') => app.next_theme(),
                KeyCode::Char('r') => {
                    app.set_status("refreshing…");
                    app.fetch(&client).await;
                    last_fetch = Instant::now();
                    app.set_status("refreshed");
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.view == View::Home {
                        app.select_next();
                    } else {
                        app.scroll_down();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.view == View::Home {
                        app.select_prev();
                    } else {
                        app.scroll_up();
                    }
                }
                KeyCode::PageDown => {
                    for _ in 0..10 {
                        if app.view == View::Home { app.select_next(); } else { app.scroll_down(); }
                    }
                }
                KeyCode::PageUp => {
                    for _ in 0..10 {
                        if app.view == View::Home { app.select_prev(); } else { app.scroll_up(); }
                    }
                }
                KeyCode::Home => {
                    app.selected = 0;
                    app.scroll = 0;
                    app.table_state.select(Some(0));
                }
                KeyCode::End => {
                    if !app.sessions.is_empty() {
                        app.selected = app.sessions.len() - 1;
                        app.table_state.select(Some(app.selected));
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if let Some(v) = View::from_digit(c) {
                        app.set_view(v);
                        app.fetch(&client).await;
                        last_fetch = Instant::now();
                    }
                }
                KeyCode::Enter => {
                    match app.view {
                        View::Home => app.set_view(View::Context),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
