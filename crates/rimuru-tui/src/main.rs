use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
    Frame, Terminal,
};
use serde_json::Value;

mod client;
mod state;
mod theme;

use client::ApiClient;
use state::{App, Tab};
use theme::Theme;

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
    let client = ApiClient::new("http://127.0.0.1:3100");
    let mut app = App::new();
    let mut last_fetch = Instant::now() - Duration::from_secs(10);

    loop {
        if last_fetch.elapsed() >= Duration::from_secs(3) {
            app.fetch(&client).await;
            last_fetch = Instant::now();
        }

        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('1') => app.tab = Tab::Dashboard,
                    KeyCode::Char('2') => app.tab = Tab::Agents,
                    KeyCode::Char('3') => app.tab = Tab::Sessions,
                    KeyCode::Char('4') => app.tab = Tab::Costs,
                    KeyCode::Char('5') => app.tab = Tab::Models,
                    KeyCode::Char('6') => app.tab = Tab::Advisor,
                    KeyCode::Char('7') => app.tab = Tab::Hooks,
                    KeyCode::Char('8') => app.tab = Tab::Plugins,
                    KeyCode::Char('9') => app.tab = Tab::Mcp,
                    KeyCode::Char('0') => app.tab = Tab::Metrics,
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::Char('t') => app.next_theme(),
                    KeyCode::Char('r') => {
                        app.fetch(&client).await;
                        last_fetch = Instant::now();
                    }
                    _ => {}
                }
            }
        }
    }
}

fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);

    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Agents => draw_agents(f, app, chunks[1]),
        Tab::Sessions => draw_sessions(f, app, chunks[1]),
        Tab::Costs => draw_costs(f, app, chunks[1]),
        Tab::Models => draw_models(f, app, chunks[1]),
        Tab::Advisor => draw_advisor(f, app, chunks[1]),
        Tab::Hooks => draw_hooks(f, app, chunks[1]),
        Tab::Plugins => draw_plugins(f, app, chunks[1]),
        Tab::Mcp => draw_mcp(f, app, chunks[1]),
        Tab::Metrics => draw_metrics(f, app, chunks[1]),
    }

    draw_footer(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                Style::default().fg(th.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th.text_dim)
            };
            Line::from(Span::styled(t.name(), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title(Span::styled(
                    format!(" Rimuru \u{2014} {} ", th.name),
                    Style::default().fg(th.accent).add_modifier(Modifier::BOLD),
                )),
        )
        .select(app.tab.index())
        .highlight_style(Style::default().fg(th.accent));

    f.render_widget(tabs, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let status = if app.connected {
        Span::styled(" LIVE ", Style::default().fg(th.success))
    } else {
        Span::styled(" OFFLINE ", Style::default().fg(th.error))
    };

    let help = Span::styled(
        " Tab:switch  1-0:jump  j/k:scroll  t:theme  r:refresh  q:quit ",
        Style::default().fg(th.text_dim),
    );

    let line = Line::from(vec![status, help]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    let stats_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(chunks[0]);

    let stats = &app.stats;
    let stat_cards = [
        (
            "Total Cost",
            format!("${:.2}", val_f64(stats, "total_cost")),
            th.accent,
        ),
        (
            "Active Agents",
            format!("{}", val_u64(stats, "active_agents")),
            th.success,
        ),
        (
            "Sessions",
            format!("{}", val_u64(stats, "total_sessions")),
            th.warning,
        ),
        (
            "Tokens",
            format_tokens(val_u64(stats, "total_tokens")),
            th.error,
        ),
        ("Savings", format!("${:.2}", app.total_savings), th.success),
    ];

    for (i, (label, value, color)) in stat_cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" {} ", label),
                Style::default().fg(th.text_dim),
            ));
        let content = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        )))
        .block(block);
        f.render_widget(content, stats_row[i]);
    }

    let hw_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    let hw = &app.hardware;
    let hw_cards = [
        ("CPU", val_str(hw, "cpu_brand"), th.accent),
        (
            "RAM",
            format!("{} GB", val_u64(hw, "total_ram_mb") / 1024),
            th.success,
        ),
        (
            "GPU",
            hw.get("gpu")
                .and_then(|g| g.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("None")
                .to_string(),
            th.warning,
        ),
        ("Backend", val_str(hw, "backend").to_uppercase(), th.info),
    ];

    for (i, (label, value, color)) in hw_cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" {} ", label),
                Style::default().fg(th.text_dim),
            ));
        let text = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color),
        )))
        .block(block);
        f.render_widget(text, hw_row[i]);
    }

    let activity_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th.border))
        .title(Span::styled(" Activity ", Style::default().fg(th.text)));

    let items: Vec<ListItem> = app
        .activity
        .iter()
        .take(20)
        .map(|evt| {
            let msg = evt.get("message").and_then(|m| m.as_str()).unwrap_or("");
            let ts = evt.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
            let short_ts = ts.get(11..19).unwrap_or("");
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", short_ts), Style::default().fg(th.text_dim)),
                Span::styled(msg.to_string(), Style::default().fg(th.text)),
            ]))
        })
        .collect();

    let list = List::new(items).block(activity_block);
    f.render_widget(list, chunks[2]);
}

fn draw_agents(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let header = Row::new(vec!["Name", "Type", "Status", "Model", "Sessions", "Cost"])
        .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let status_color = match a.get("status").and_then(|s| s.as_str()).unwrap_or("") {
                "connected" | "active" => th.success,
                "idle" => th.text_dim,
                _ => th.error,
            };
            let row = Row::new(vec![
                Cell::from(val_str(a, "name")),
                Cell::from(val_str(a, "agent_type")),
                Cell::from(Span::styled(
                    val_str(a, "status"),
                    Style::default().fg(status_color),
                )),
                Cell::from(val_str(a, "model")),
                Cell::from(format!("{}", val_u64(a, "session_count"))),
                Cell::from(format!("${:.4}", val_f64(a, "total_cost"))),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" Agents ({}) ", app.agents.len()),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, area);
}

fn draw_sessions(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let header = Row::new(vec![
        "Agent", "Model", "Status", "Tokens", "Cost", "Started",
    ])
    .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let status = val_str(s, "status");
            let color = match status.as_str() {
                "active" => th.success,
                "completed" => th.accent,
                _ => th.error,
            };
            let tokens = val_u64(s, "input_tokens") + val_u64(s, "output_tokens");
            let ts = val_str(s, "started_at");
            let short = ts.get(..16).unwrap_or(&ts).to_string();
            let row = Row::new(vec![
                Cell::from(val_str(s, "agent_type")),
                Cell::from(val_str(s, "model")),
                Cell::from(Span::styled(status, Style::default().fg(color))),
                Cell::from(format_tokens(tokens)),
                Cell::from(format!("${:.4}", val_f64(s, "total_cost"))),
                Cell::from(short),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(13),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" Sessions ({}) ", app.sessions.len()),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, area);
}

fn draw_costs(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let summary = &app.cost_summary;
    let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_cost = app
        .daily_costs
        .iter()
        .find(|d| {
            d.get("date")
                .and_then(|v| v.as_str())
                .map(|s| s == today_str)
                .unwrap_or(false)
        })
        .map(|d| val_f64(d, "cost"))
        .unwrap_or(0.0);

    let summary_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    let cards = [
        (
            "Total Cost",
            format!("${:.2}", val_f64(summary, "total_cost")),
            th.accent,
        ),
        ("Today", format!("${:.2}", today_cost), th.warning),
        (
            "Input Tokens",
            format_tokens(val_u64(summary, "total_input_tokens")),
            th.success,
        ),
        (
            "Output Tokens",
            format_tokens(val_u64(summary, "total_output_tokens")),
            th.info,
        ),
    ];

    for (i, (label, value, color)) in cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" {} ", label),
                Style::default().fg(th.text_dim),
            ));
        let text = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        )))
        .block(block);
        f.render_widget(text, summary_row[i]);
    }

    let header = Row::new(vec!["Date", "Cost", "Tokens", "Sessions"])
        .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .daily_costs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let row = Row::new(vec![
                Cell::from(val_str(d, "date")),
                Cell::from(format!(
                    "${:.4}",
                    val_f64(d, "cost").max(val_f64(d, "total_cost"))
                )),
                Cell::from(format_tokens(
                    val_u64(d, "input_tokens") + val_u64(d, "output_tokens"),
                )),
                Cell::from(format!("{}", val_u64(d, "sessions"))),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(" Daily Costs ", Style::default().fg(th.text))),
    );

    f.render_widget(table, chunks[1]);
}

fn draw_models(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let header = Row::new(vec![
        "API Model",
        "Provider",
        "Input/1M",
        "Output/1M",
        "Context",
        "Run Locally?",
        "Local Alternative",
        "tok/s",
    ])
    .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let model_id = val_str(m, "id");
            let adv = app
                .advisories
                .iter()
                .find(|a| val_str(a, "model_id") == model_id);
            let fit = adv.map(|a| val_str(a, "fit_level")).unwrap_or_default();
            let (fit_icon, fit_color) = match fit.as_str() {
                "perfect" => ("\u{2714} YES", th.success),
                "good" => ("\u{2714} YES", th.accent),
                "marginal" => ("~ SLOW", th.warning),
                "too_tight" => ("\u{2718} NO", th.error),
                _ => ("\u{2014}", th.text_dim),
            };
            let tok_s = adv
                .and_then(|a| a.get("estimated_tok_per_sec"))
                .and_then(|v| v.as_f64())
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "\u{2014}".into());
            let local_eq = adv
                .and_then(|a| a.get("local_equivalent"))
                .and_then(|v| v.as_str())
                .unwrap_or("\u{2014}");
            let quant = adv
                .and_then(|a| a.get("best_quantization"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let local_display = if local_eq == "\u{2014}" {
                "\u{2014}".to_string()
            } else if quant.is_empty() {
                local_eq.to_string()
            } else {
                format!("{} ({})", local_eq, quant)
            };

            let row = Row::new(vec![
                Cell::from(val_str(m, "name")),
                Cell::from(val_str(m, "provider")),
                Cell::from(format!("${:.2}", val_f64(m, "input_price_per_million"))),
                Cell::from(format!("${:.2}", val_f64(m, "output_price_per_million"))),
                Cell::from(format_context(val_u64(m, "context_window"))),
                Cell::from(Span::styled(
                    fit_icon.to_string(),
                    Style::default().fg(fit_color),
                )),
                Cell::from(local_display),
                Cell::from(tok_s),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(16),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(8),
            Constraint::Percentage(12),
            Constraint::Percentage(24),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(
                    " Models ({}) \u{2014} Can you replace API models with local ones? ",
                    app.models.len()
                ),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, area);
}

fn draw_advisor(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let summary_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    let (perfect, good, marginal, total) = app.catalog_summary();
    let summary_cards = [
        ("Perfect", format!("{}", perfect), th.success),
        ("Good", format!("{}", good), th.accent),
        ("Marginal", format!("{}", marginal), th.warning),
        ("Catalog", format!("{}", total), th.text_dim),
    ];

    for (i, (label, value, color)) in summary_cards.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" {} ", label),
                Style::default().fg(th.text_dim),
            ));
        let text = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        )))
        .block(block);
        f.render_widget(text, summary_row[i]);
    }

    let header = Row::new(vec![
        "Model",
        "Params",
        "Fit",
        "Quant",
        "VRAM",
        "tok/s",
        "Downloads",
    ])
    .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .catalog
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let fit = val_str(e, "fit_level");
            let fit_color = match fit.as_str() {
                "perfect" => th.success,
                "good" => th.accent,
                "marginal" => th.warning,
                _ => th.text_dim,
            };
            let name = val_str(e, "name");
            let short_name = name.split('/').next_back().unwrap_or(&name).to_string();
            let vram = e
                .get("estimated_vram_mb")
                .and_then(|v| v.as_u64())
                .map(|v| format!("{:.1}G", v as f64 / 1024.0))
                .unwrap_or_else(|| "\u{2014}".into());
            let tok_s = e
                .get("estimated_tok_per_sec")
                .and_then(|v| v.as_f64())
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "\u{2014}".into());
            let downloads = val_u64(e, "hf_downloads");

            let row = Row::new(vec![
                Cell::from(short_name),
                Cell::from(format!("{:.1}B", val_f64(e, "params_b"))),
                Cell::from(Span::styled(fit.clone(), Style::default().fg(fit_color))),
                Cell::from(
                    e.get("best_quantization")
                        .and_then(|v| v.as_str())
                        .unwrap_or("\u{2014}")
                        .to_string(),
                ),
                Cell::from(vram),
                Cell::from(tok_s),
                Cell::from(format_downloads(downloads)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(28),
            Constraint::Percentage(10),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(
                    " Model Advisor \u{2014} {} can run on your hardware ",
                    app.catalog.len()
                ),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, chunks[1]);
}

fn draw_hooks(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let header = Row::new(vec!["Name", "Event", "Matcher", "Plugin", "Enabled"])
        .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .hooks
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let name = val_str(h, "name");
            let display = if name.is_empty() {
                truncate_str(&val_str(h, "id"), 30)
            } else {
                truncate_str(&name, 30)
            };
            let event = val_str(h, "event_type");
            let event_color = match event.as_str() {
                "PreToolUse" | "PostToolUse" => th.accent,
                "Stop" | "SessionStart" | "SessionEnd" => th.success,
                "UserPromptSubmit" => th.warning,
                _ => th.text_dim,
            };
            let enabled = h.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let enabled_color = if enabled { th.success } else { th.text_dim };
            let matcher = h.get("matcher").and_then(|v| v.as_str()).unwrap_or("-");
            let plugin = h.get("plugin_id").and_then(|v| v.as_str()).unwrap_or("-");

            let row = Row::new(vec![
                Cell::from(display),
                Cell::from(Span::styled(event, Style::default().fg(event_color))),
                Cell::from(matcher.to_string()),
                Cell::from(truncate_str(plugin, 20)),
                Cell::from(Span::styled(
                    if enabled { "Yes" } else { "No" },
                    Style::default().fg(enabled_color),
                )),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" Hooks ({}) ", app.hooks.len()),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, area);
}

fn draw_plugins(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let header = Row::new(vec![
        "Name", "Version", "Author", "Language", "Enabled", "Hooks",
    ])
    .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .plugins
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let enabled = p.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let enabled_color = if enabled { th.success } else { th.text_dim };
            let hook_count = p
                .get("hooks")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            let row = Row::new(vec![
                Cell::from(val_str(p, "name")),
                Cell::from(val_str(p, "version")),
                Cell::from(
                    p.get("author")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-")
                        .to_string(),
                ),
                Cell::from(val_str(p, "language")),
                Cell::from(Span::styled(
                    if enabled { "Yes" } else { "No" },
                    Style::default().fg(enabled_color),
                )),
                Cell::from(format!("{}", hook_count)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(13),
            Constraint::Percentage(12),
            Constraint::Percentage(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" Plugins ({}) ", app.plugins.len()),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, area);
}

fn draw_mcp(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();

    if app.mcp_servers.is_empty() {
        let p = Paragraph::new("No MCP servers configured")
            .style(Style::default().fg(th.text_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(th.border))
                    .title(Span::styled(" MCP Servers ", Style::default().fg(th.text))),
            );
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let header = Row::new(vec!["Name", "Command", "Source", "Enabled"])
        .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .mcp_servers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let enabled = s.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let enabled_color = if enabled { th.success } else { th.text_dim };

            let row = Row::new(vec![
                Cell::from(val_str(s, "name")),
                Cell::from(val_str(s, "command")),
                Cell::from(val_str(s, "source")),
                Cell::from(Span::styled(
                    if enabled { "Yes" } else { "No" },
                    Style::default().fg(enabled_color),
                )),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(th.selection))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                format!(" MCP Servers ({}) ", app.mcp_servers.len()),
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(table, chunks[0]);

    let detail_lines: Vec<Line> = if let Some(server) = app.mcp_servers.get(app.scroll) {
        let args = server
            .get("args")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Command:  ", Style::default().fg(th.text_dim)),
                Span::styled(val_str(server, "command"), Style::default().fg(th.text)),
            ]),
            Line::from(vec![
                Span::styled("  Args:     ", Style::default().fg(th.text_dim)),
                Span::styled(args, Style::default().fg(th.text)),
            ]),
            Line::from(vec![
                Span::styled("  Source:   ", Style::default().fg(th.text_dim)),
                Span::styled(val_str(server, "source"), Style::default().fg(th.accent)),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Select a server to view details",
                Style::default().fg(th.text_dim),
            )),
        ]
    };

    let detail = Paragraph::new(detail_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th.border))
            .title(Span::styled(
                " Server Details ",
                Style::default().fg(th.text),
            )),
    );

    f.render_widget(detail, chunks[1]);
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

fn draw_metrics(f: &mut Frame, app: &App, area: Rect) {
    let th = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    let gauge_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    let m = &app.metrics;
    let cpu = m
        .get("cpu_usage_percent")
        .or_else(|| m.get("cpu_percent"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let mem_used = val_f64(m, "memory_used_mb");
    let mem_total = val_f64(m, "memory_total_mb").max(1.0);
    let mem_pct = (mem_used / mem_total * 100.0).min(100.0);
    let active_agents = val_u64(m, "active_agents");
    let uptime = val_u64(m, "uptime_secs");

    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title(Span::styled(" CPU ", Style::default().fg(th.text_dim))),
        )
        .gauge_style(Style::default().fg(themed_gauge_color(th, cpu)))
        .ratio((cpu / 100.0).min(1.0))
        .label(format!("{:.1}%", cpu));
    f.render_widget(cpu_gauge, gauge_row[0]);

    let mem_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th.border))
                .title(Span::styled(" Memory ", Style::default().fg(th.text_dim))),
        )
        .gauge_style(Style::default().fg(themed_gauge_color(th, mem_pct)))
        .ratio((mem_pct / 100.0).min(1.0))
        .label(format!("{:.0}/{:.0} MB", mem_used, mem_total));
    f.render_widget(mem_gauge, gauge_row[1]);

    let agents_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th.border))
        .title(Span::styled(" Agents ", Style::default().fg(th.text_dim)));
    let agents_text = Paragraph::new(Line::from(Span::styled(
        format!("{}", active_agents),
        Style::default().fg(th.success).add_modifier(Modifier::BOLD),
    )))
    .block(agents_block);
    f.render_widget(agents_text, gauge_row[2]);

    let uptime_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th.border))
        .title(Span::styled(" Uptime ", Style::default().fg(th.text_dim)));
    let uptime_text = Paragraph::new(Line::from(Span::styled(
        format_uptime(uptime),
        Style::default().fg(th.accent),
    )))
    .block(uptime_block);
    f.render_widget(uptime_text, gauge_row[3]);

    let hw = &app.hardware;
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("CPU: ", Style::default().fg(th.text_dim)),
        Span::styled(val_str(hw, "cpu_brand"), Style::default().fg(th.text)),
        Span::styled(
            format!(" ({} cores)", val_u64(hw, "cpu_cores")),
            Style::default().fg(th.text_dim),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("RAM: ", Style::default().fg(th.text_dim)),
        Span::styled(
            format!(
                "{} GB total, {} GB available",
                val_u64(hw, "total_ram_mb") / 1024,
                val_u64(hw, "available_ram_mb") / 1024
            ),
            Style::default().fg(th.text),
        ),
    ]));
    let gpu_name = hw
        .get("gpu")
        .and_then(|g| g.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("None");
    let gpu_vram = hw
        .get("gpu")
        .and_then(|g| g.get("vram_mb"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    lines.push(Line::from(vec![
        Span::styled("GPU: ", Style::default().fg(th.text_dim)),
        Span::styled(gpu_name, Style::default().fg(th.warning)),
        Span::styled(
            format!(" ({} GB)", gpu_vram / 1024),
            Style::default().fg(th.text_dim),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Backend: ", Style::default().fg(th.text_dim)),
        Span::styled(
            val_str(hw, "backend").to_uppercase(),
            Style::default().fg(th.info).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  OS: {} / {}", val_str(hw, "os"), val_str(hw, "arch")),
            Style::default().fg(th.text_dim),
        ),
    ]));

    let hw_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th.border))
        .title(Span::styled(
            " System Hardware ",
            Style::default().fg(th.text),
        ));
    let hw_para = Paragraph::new(lines)
        .block(hw_block)
        .wrap(Wrap { trim: false });
    f.render_widget(hw_para, chunks[1]);
}

fn themed_gauge_color(th: &Theme, pct: f64) -> Color {
    if pct < 50.0 {
        th.gauge_low
    } else if pct < 80.0 {
        th.gauge_mid
    } else {
        th.gauge_high
    }
}

fn val_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn val_f64(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn val_u64(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn format_tokens(t: u64) -> String {
    if t >= 1_000_000_000 {
        format!("{:.1}B", t as f64 / 1e9)
    } else if t >= 1_000_000 {
        format!("{:.1}M", t as f64 / 1e6)
    } else if t >= 1_000 {
        format!("{:.1}K", t as f64 / 1e3)
    } else {
        format!("{}", t)
    }
}

fn format_context(t: u64) -> String {
    if t >= 1_000_000 {
        format!("{:.1}M", t as f64 / 1e6)
    } else if t >= 1_000 {
        format!("{}K", t / 1_000)
    } else {
        format!("{}", t)
    }
}

fn format_downloads(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1e6)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{}", n)
    }
}

fn format_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}
