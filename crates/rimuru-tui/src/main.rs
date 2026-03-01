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
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use event::{AppEvent, EventReader};
use ratatui::prelude::*;
use std::io;

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
    let client = ApiClient::new(args.port);
    let events = EventReader::new(250);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    if args.theme < theme::THEMES.len() {
        app.theme_index = args.theme;
    }

    app.refresh(&client).await;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        match events.next()? {
            AppEvent::Key(key) => {
                if app.searching {
                    match key.code {
                        KeyCode::Esc => {
                            app.searching = false;
                            app.search_query.clear();
                        }
                        KeyCode::Enter => {
                            app.searching = false;
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) || !app.searching {
                            app.running = false;
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.running = false;
                    }
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                    KeyCode::Char('t') => app.next_theme(),
                    KeyCode::Char('r') => app.refresh(&client).await,
                    KeyCode::Char('/') => {
                        app.searching = true;
                        app.search_query.clear();
                    }
                    KeyCode::Char('?') => app.switch_tab(app::Tab::Help),
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if let Some(tab) = app::Tab::from_key(c) {
                            app.switch_tab(tab);
                            app.refresh(&client).await;
                        }
                    }
                    KeyCode::Enter => {
                        handle_enter(&mut app, &client).await;
                    }
                    _ => {}
                }
            }
            AppEvent::Tick => {
                app.refresh(&client).await;
            }
        }

        if !app.running {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_enter(app: &mut App, client: &ApiClient) {
    match app.current_tab {
        app::Tab::Agents => {
            if let Some(agent) = app.agents.get(app.selected_index) {
                let id = agent.id.clone();
                let action = if agent.status == "Connected" || agent.status == "Active" {
                    client.disconnect_agent(&id).await
                } else {
                    client.connect_agent(&id).await
                };
                match action {
                    Ok(_) => {
                        app.status_message = Some("Agent toggled".to_string());
                        app.refresh(client).await;
                    }
                    Err(e) => app.status_message = Some(format!("Error: {}", e)),
                }
            }
        }
        app::Tab::Plugins => {
            if let Some(plugin) = app.plugins.get(app.selected_index) {
                let id = plugin.id.clone();
                let action = if plugin.enabled { "disable" } else { "enable" };
                match client.toggle_plugin(&id, action).await {
                    Ok(_) => {
                        app.status_message = Some(format!("Plugin {}", action));
                        app.refresh(client).await;
                    }
                    Err(e) => app.status_message = Some(format!("Error: {}", e)),
                }
            }
        }
        app::Tab::Models => {
            match client.sync_models().await {
                Ok(_) => {
                    app.status_message = Some("Models synced".to_string());
                    app.refresh(client).await;
                }
                Err(e) => app.status_message = Some(format!("Error: {}", e)),
            }
        }
        _ => {}
    }
}
