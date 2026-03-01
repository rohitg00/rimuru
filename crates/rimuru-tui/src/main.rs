#![allow(dead_code)]

mod app;
mod client;
mod event;
mod theme;
mod ui;
mod views;

use app::App;
use client::ApiClient;
use clap::Parser;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use event::{AppEvent, EventReader};
use ratatui::prelude::*;
use std::io;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "rimuru-tui", about = "Rimuru Terminal UI")]
struct Args {
    #[arg(short, long, default_value_t = 3100)]
    port: u16,

    #[arg(short, long, default_value_t = 0)]
    theme: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Arc::new(ApiClient::new(args.port));
    let events = EventReader::new(50);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(App::new()));
    if args.theme < theme::THEMES.len() {
        app.lock().await.theme_index = args.theme;
    }

    let mut last_refresh = Instant::now();
    let refresh_interval = std::time::Duration::from_secs(2);
    spawn_refresh(Arc::clone(&app), Arc::clone(&client));

    loop {
        {
            let app_guard = app.lock().await;
            terminal.draw(|f| ui::render(f, &app_guard))?;
            if !app_guard.running {
                break;
            }
        }

        match events.next()? {
            AppEvent::Key(key) => {
                let mut app_guard = app.lock().await;

                if app_guard.searching {
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.searching = false;
                            app_guard.search_query.clear();
                        }
                        KeyCode::Enter => {
                            app_guard.searching = false;
                        }
                        KeyCode::Backspace => {
                            app_guard.search_query.pop();
                        }
                        KeyCode::Char(c) => {
                            app_guard.search_query.push(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app_guard.running = false;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app_guard.running = false;
                    }
                    KeyCode::Tab => app_guard.next_tab(),
                    KeyCode::BackTab => app_guard.prev_tab(),
                    KeyCode::Char('j') | KeyCode::Down => app_guard.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => app_guard.scroll_up(),
                    KeyCode::Char('t') => app_guard.next_theme(),
                    KeyCode::Char('r') => {
                        drop(app_guard);
                        spawn_refresh(Arc::clone(&app), Arc::clone(&client));
                        last_refresh = Instant::now();
                    }
                    KeyCode::Char('/') => {
                        app_guard.searching = true;
                        app_guard.search_query.clear();
                    }
                    KeyCode::Char('?') => app_guard.switch_tab(app::Tab::Help),
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if let Some(tab) = app::Tab::from_key(c) {
                            app_guard.switch_tab(tab);
                            drop(app_guard);
                            spawn_refresh(Arc::clone(&app), Arc::clone(&client));
                            last_refresh = Instant::now();
                        }
                    }
                    KeyCode::Enter => {
                        let tab = app_guard.current_tab;
                        let idx = app_guard.selected_index;
                        handle_enter(&mut app_guard, tab, idx, &client).await;
                    }
                    _ => {}
                }
            }
            AppEvent::Tick => {
                if last_refresh.elapsed() >= refresh_interval {
                    spawn_refresh(Arc::clone(&app), Arc::clone(&client));
                    last_refresh = Instant::now();
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn spawn_refresh(app: Arc<Mutex<App>>, client: Arc<ApiClient>) {
    tokio::spawn(async move {
        let tab = {
            let guard = app.lock().await;
            guard.current_tab
        };
        let results = client.refresh_for_tab(tab).await;
        let mut guard = app.lock().await;
        guard.apply_refresh(results);
    });
}

async fn handle_enter(
    app: &mut App,
    tab: app::Tab,
    idx: usize,
    client: &ApiClient,
) {
    match tab {
        app::Tab::Agents => {
            if let Some(agent) = app.agents.get(idx) {
                let id = agent.id.clone();
                let is_connected =
                    agent.status == "Connected" || agent.status == "Active";
                let result = if is_connected {
                    client.disconnect_agent(&id).await
                } else {
                    client.connect_agent(&id).await
                };
                match result {
                    Ok(_) => app.status_message = Some("Agent toggled".to_string()),
                    Err(e) => app.status_message = Some(format!("Error: {}", e)),
                }
            }
        }
        app::Tab::Plugins => {
            if let Some(plugin) = app.plugins.get(idx) {
                let id = plugin.id.clone();
                let action = if plugin.enabled { "disable" } else { "enable" };
                match client.toggle_plugin(&id, action).await {
                    Ok(_) => app.status_message = Some(format!("Plugin {}", action)),
                    Err(e) => app.status_message = Some(format!("Error: {}", e)),
                }
            }
        }
        app::Tab::Models => match client.sync_models().await {
            Ok(_) => app.status_message = Some("Models synced".to_string()),
            Err(e) => app.status_message = Some(format!("Error: {}", e)),
        },
        _ => {}
    }
}
