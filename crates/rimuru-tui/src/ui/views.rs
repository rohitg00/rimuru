use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use serde_json::Value;

use crate::state::App;
use crate::widgets::{btop_block, fmt_dollars, fmt_tokens, grad_at, make_gradient, meter_bar, truncate_str};

fn val_str(v: &Value, k: &str) -> String {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
}
fn val_u64(v: &Value, k: &str) -> u64 {
    v.get(k).and_then(|x| x.as_u64()).unwrap_or(0)
}
fn val_f64(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0)
}
fn val_bool(v: &Value, k: &str) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or(false)
}

fn stat_cards(f: &mut Frame, app: &App, area: Rect, cards: &[(&str, String, Color)]) {
    let theme = app.theme();
    let n = cards.len() as u16;
    if n == 0 { return; }
    let pct = 100 / n;
    let cs: Vec<Constraint> = (0..n).map(|_| Constraint::Percentage(pct as u16)).collect();
    let split = Layout::default().direction(Direction::Horizontal).constraints(cs).split(area);

    for (i, (label, value, color)) in cards.iter().enumerate() {
        let block = btop_block(label, "", theme.div_line, theme);
        let para = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color).add_modifier(Modifier::BOLD),
        )))
        .block(block);
        f.render_widget(para, split[i]);
    }
}

// ── costs ────────────────────────────────────────────────────────────────────
pub(super) fn draw_costs(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let summary = &app.cost_summary;
    let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_cost = app
        .daily_costs
        .iter()
        .find(|d| val_str(d, "date") == today_str)
        .map(|d| val_f64(d, "cost").max(val_f64(d, "total_cost")))
        .unwrap_or(0.0);

    stat_cards(f, app, chunks[0], &[
        ("Total", fmt_dollars(val_f64(summary, "total_cost")), theme.hi_fg),
        ("Today", fmt_dollars(today_cost), theme.warning_fg),
        ("Input Tokens", fmt_tokens(val_u64(summary, "total_input_tokens")), theme.proc_misc),
        ("Output Tokens", fmt_tokens(val_u64(summary, "total_output_tokens")), theme.main_fg),
    ]);

    let header = Row::new(vec!["Date", "Cost", "Tokens", "Sessions"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .daily_costs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let row = Row::new(vec![
                Cell::from(val_str(d, "date")),
                Cell::from(fmt_dollars(val_f64(d, "cost").max(val_f64(d, "total_cost")))),
                Cell::from(fmt_tokens(val_u64(d, "input_tokens") + val_u64(d, "output_tokens"))),
                Cell::from(format!("{}", val_u64(d, "sessions"))),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .header(header)
    .block(btop_block("daily costs", "", theme.mem_box, theme));
    f.render_widget(table, chunks[1]);
}

// ── budget ───────────────────────────────────────────────────────────────────
pub(super) fn draw_budget(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let status = &app.budget_status;
    let monthly_limit = val_f64(status, "monthly_limit");
    let monthly_spent = val_f64(status, "monthly_spent");
    let daily_limit = val_f64(status, "daily_limit");
    let daily_spent = val_f64(status, "daily_spent");
    let session_limit = val_f64(status, "session_limit");
    let agent_daily_limit = val_f64(status, "agent_daily_limit");
    let burn_rate = val_f64(status, "burn_rate_daily");
    let projected = val_f64(status, "projected_monthly");
    let action = val_str(status, "action_on_exceed");
    let status_label = val_str(status, "status");
    let threshold = val_f64(status, "alert_threshold");

    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);

    let caps: [(&str, Option<f64>, f64); 4] = [
        ("Monthly", Some(monthly_spent), monthly_limit),
        ("Daily", Some(daily_spent), daily_limit),
        ("Per Session", None, session_limit),
        ("Per Agent/Day", None, agent_daily_limit),
    ];

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25); 4])
        .split(chunks[0]);

    for (i, (label, spent, limit)) in caps.iter().copied().enumerate() {
        let area_i = split[i];
        let block = btop_block(label, "", theme.div_line, theme);
        let mut lines: Vec<Line> = Vec::new();

        if limit <= 0.0 {
            lines.push(Line::from(Span::styled("—", Style::default().fg(theme.inactive_fg))));
            lines.push(Line::from(Span::styled("disabled", Style::default().fg(theme.inactive_fg))));
        } else if let Some(sp) = spent {
            let pct = (sp / limit * 100.0).clamp(0.0, 100.0);
            let cap_color = if sp >= limit {
                theme.status_fg
            } else if sp >= limit * threshold {
                theme.warning_fg
            } else {
                theme.proc_misc
            };
            lines.push(Line::from(vec![
                Span::styled(fmt_dollars(sp), Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" / {}", fmt_dollars(limit)), Style::default().fg(theme.graph_text)),
            ]));
            let bar_w = (area_i.width as usize).saturating_sub(4).clamp(3, 20);
            let mut bar_line = vec![Span::raw(" ")];
            bar_line.extend(meter_bar(pct, bar_w, &cpu_grad, theme.meter_bg));
            bar_line.push(Span::styled(format!(" {:.0}%", pct), Style::default().fg(cap_color)));
            lines.push(Line::from(bar_line));
        } else {
            lines.push(Line::from(vec![
                Span::styled("— ", Style::default().fg(theme.inactive_fg)),
                Span::styled(format!("/ {}", fmt_dollars(limit)), Style::default().fg(theme.graph_text)),
            ]));
            lines.push(Line::from(Span::styled("not aggregated", Style::default().fg(theme.inactive_fg))));
        }

        f.render_widget(Paragraph::new(lines).block(block), area_i);
    }

    let status_color = match status_label.as_str() {
        "exceeded" => theme.status_fg,
        "warning" => theme.warning_fg,
        _ => theme.proc_misc,
    };

    let status_line = Line::from(vec![
        Span::styled(" status ", Style::default().fg(theme.graph_text)),
        Span::styled(status_label.to_uppercase(), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::styled("   burn ", Style::default().fg(theme.graph_text)),
        Span::styled(format!("{}/day", fmt_dollars(burn_rate)), Style::default().fg(theme.main_fg)),
        Span::styled("   projected ", Style::default().fg(theme.graph_text)),
        Span::styled(
            fmt_dollars(projected),
            Style::default().fg(if monthly_limit > 0.0 && projected > monthly_limit {
                theme.status_fg
            } else {
                theme.main_fg
            }),
        ),
        Span::styled("   on-exceed ", Style::default().fg(theme.graph_text)),
        Span::styled(action.to_uppercase(), Style::default().fg(theme.hi_fg)),
    ]);
    f.render_widget(Paragraph::new(status_line), chunks[1]);

    let alerts = app
        .budget_alerts
        .get("alerts")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let header = Row::new(vec!["Time", "Type", "Hit", "Monthly", "Daily", "Message"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = alerts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let at = val_str(a, "alert_type");
            let color = match at.as_str() {
                "exceeded" => theme.status_fg,
                "warning" => theme.warning_fg,
                _ => theme.proc_misc,
            };
            let row = Row::new(vec![
                Cell::from(val_str(a, "timestamp")),
                Cell::from(Span::styled(at.to_uppercase(), Style::default().fg(color))),
                Cell::from(val_str(a, "limit_hit")),
                Cell::from(fmt_dollars(val_f64(a, "monthly_spent"))),
                Cell::from(fmt_dollars(val_f64(a, "daily_spent"))),
                Cell::from(val_str(a, "message")),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(&format!("alerts ({})", alerts.len()), "", theme.mem_box, theme);
    let table = Table::new(rows, [
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(20),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, chunks[2]);
}

// ── models ───────────────────────────────────────────────────────────────────
pub(super) fn draw_models(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();

    let header = Row::new(vec!["Model", "Provider", "Input/1M", "Output/1M", "Ctx", "Local?", "Alternative", "tok/s"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let id = val_str(m, "id");
            let adv = app.advisories.iter().find(|a| val_str(a, "model_id") == id);
            let fit = adv.map(|a| val_str(a, "fit_level")).unwrap_or_default();
            let (fit_icon, fit_color) = match fit.as_str() {
                "perfect" => ("✓ YES", theme.proc_misc),
                "good" => ("✓ yes", theme.hi_fg),
                "marginal" => ("~ slow", theme.warning_fg),
                "too_tight" => ("✗ NO", theme.status_fg),
                _ => ("—", theme.inactive_fg),
            };
            let tok_s = adv
                .and_then(|a| a.get("estimated_tok_per_sec"))
                .and_then(|v| v.as_f64())
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "—".into());
            let local_eq = adv
                .and_then(|a| a.get("local_equivalent"))
                .and_then(|v| v.as_str())
                .unwrap_or("—");
            let quant = adv
                .and_then(|a| a.get("best_quantization"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let local_display = if local_eq == "—" {
                "—".to_string()
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
                Cell::from(fmt_ctx(val_u64(m, "context_window"))),
                Cell::from(Span::styled(fit_icon.to_string(), Style::default().fg(fit_color))),
                Cell::from(local_display),
                Cell::from(tok_s),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("models ({}) — can local replace API?", app.models.len()),
        "",
        theme.proc_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(16),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(8),
        Constraint::Percentage(12),
        Constraint::Percentage(24),
        Constraint::Percentage(10),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, area);
}

// ── advisor ──────────────────────────────────────────────────────────────────
pub(super) fn draw_advisor(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let (perfect, good, marginal, total) = app.catalog_summary_cache;
    stat_cards(f, app, chunks[0], &[
        ("Perfect", format!("{}", perfect), theme.proc_misc),
        ("Good", format!("{}", good), theme.hi_fg),
        ("Marginal", format!("{}", marginal), theme.warning_fg),
        ("Catalog", format!("{}", total), theme.inactive_fg),
    ]);

    let header = Row::new(vec!["Model", "Params", "Fit", "Quant", "VRAM", "tok/s", "Downloads"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .catalog
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let fit = val_str(e, "fit_level");
            let fit_color = match fit.as_str() {
                "perfect" => theme.proc_misc,
                "good" => theme.hi_fg,
                "marginal" => theme.warning_fg,
                _ => theme.inactive_fg,
            };
            let name = val_str(e, "name");
            let short = name.split('/').next_back().unwrap_or(&name).to_string();
            let vram = e
                .get("estimated_vram_mb")
                .and_then(|v| v.as_u64())
                .map(|v| format!("{:.1}G", v as f64 / 1024.0))
                .unwrap_or_else(|| "—".into());
            let tok_s = e
                .get("estimated_tok_per_sec")
                .and_then(|v| v.as_f64())
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "—".into());
            let downloads = val_u64(e, "hf_downloads");
            let row = Row::new(vec![
                Cell::from(short),
                Cell::from(format!("{:.1}B", val_f64(e, "params_b"))),
                Cell::from(Span::styled(fit.clone(), Style::default().fg(fit_color))),
                Cell::from(
                    e.get("best_quantization")
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_string(),
                ),
                Cell::from(vram),
                Cell::from(tok_s),
                Cell::from(fmt_downloads(downloads)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("catalog ({})", app.catalog.len()),
        "",
        theme.proc_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(28),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(22),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, chunks[1]);
}

// ── context ──────────────────────────────────────────────────────────────────
pub(super) fn draw_context(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let waste = &app.context_waste;
    stat_cards(f, app, chunks[0], &[
        ("Wasted Tokens", fmt_tokens(val_u64(waste, "wasted_tokens")), theme.warning_fg),
        ("Wasted Cost", fmt_dollars(val_f64(waste, "wasted_cost")), theme.status_fg),
        ("Repeated Reads", format!("{}", val_u64(waste, "repeated_reads")), theme.hi_fg),
        ("Sessions Tracked", format!("{}", val_u64(waste, "sessions_tracked")), theme.inactive_fg),
    ]);

    let utilizations = app
        .context_utilization
        .get("utilizations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let header = Row::new(vec!["Session", "Agent", "Used", "Total", "Pct", "Tool Calls"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);

    let rows: Vec<Row> = utilizations
        .iter()
        .enumerate()
        .map(|(i, u)| {
            let sid = val_str(u, "session_id");
            let sid_short = if sid.len() >= 8 { sid[..8].to_string() } else { sid };
            let pct = val_f64(u, "utilization_percent").max(val_f64(u, "percent"));
            let used = val_u64(u, "tokens_used").max(val_u64(u, "used_tokens"));
            let total = val_u64(u, "context_window_size").max(val_u64(u, "context_window"));
            let tool_calls = val_u64(u, "tool_calls");
            let agent = val_str(u, "agent_type");
            let color = grad_at(&cpu_grad, pct);
            let row = Row::new(vec![
                Cell::from(sid_short),
                Cell::from(agent),
                Cell::from(fmt_tokens(used)),
                Cell::from(fmt_tokens(total)),
                Cell::from(Span::styled(format!("{:.0}%", pct), Style::default().fg(color))),
                Cell::from(format!("{}", tool_calls)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("context utilization ({})", utilizations.len()),
        "",
        theme.cpu_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Length(10),
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Min(10),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, chunks[1]);
}

// ── mcp proxy ────────────────────────────────────────────────────────────────
pub(super) fn draw_mcp_proxy(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let tools = app
        .mcp_proxy_stats
        .get("tools")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let header = Row::new(vec!["Server", "Tool", "Calls", "Last Call"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = tools
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let row = Row::new(vec![
                Cell::from(val_str(t, "server")),
                Cell::from(val_str(t, "name")),
                Cell::from(format!("{}", val_u64(t, "calls"))),
                Cell::from(val_str(t, "last_call")),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("mcp proxy tools ({})", tools.len()),
        "",
        theme.net_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(40),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, area);
}

// ── hooks ────────────────────────────────────────────────────────────────────
pub(super) fn draw_hooks(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let header = Row::new(vec!["Name", "Event", "Agent", "Fires"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .hooks
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let row = Row::new(vec![
                Cell::from(val_str(h, "name")),
                Cell::from(val_str(h, "event")),
                Cell::from(val_str(h, "agent")),
                Cell::from(format!("{}", val_u64(h, "fire_count"))),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("hooks ({})", app.hooks.len()),
        "",
        theme.mem_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, area);
}

// ── plugins ──────────────────────────────────────────────────────────────────
pub(super) fn draw_plugins(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let header = Row::new(vec!["Name", "Version", "Status", "Description"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .plugins
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let status = val_str(p, "status");
            let enabled = val_bool(p, "enabled");
            let (label, color) = if enabled {
                ("● enabled", theme.proc_misc)
            } else if status.is_empty() {
                ("○ disabled", theme.inactive_fg)
            } else {
                ("◌ other", theme.warning_fg)
            };
            let row = Row::new(vec![
                Cell::from(val_str(p, "name")),
                Cell::from(val_str(p, "version")),
                Cell::from(Span::styled(label.to_string(), Style::default().fg(color))),
                Cell::from(truncate_str(&val_str(p, "description"), 80)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("plugins ({})", app.plugins.len()),
        "",
        theme.mem_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(12),
        Constraint::Percentage(15),
        Constraint::Percentage(53),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, area);
}

// ── mcp ──────────────────────────────────────────────────────────────────────
pub(super) fn draw_mcp(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let header = Row::new(vec!["Name", "Transport", "Status", "URL/Command"])
        .style(Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .mcp_servers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let status = val_str(s, "status");
            let color = match status.as_str() {
                "connected" | "running" | "ready" | "active" => theme.proc_misc,
                "" | "stopped" | "disconnected" => theme.inactive_fg,
                _ => theme.warning_fg,
            };
            let url = val_str(s, "url");
            let cmd = val_str(s, "command");
            let endpoint = if !url.is_empty() { url } else { cmd };
            let row = Row::new(vec![
                Cell::from(val_str(s, "name")),
                Cell::from(val_str(s, "transport")),
                Cell::from(Span::styled(status, Style::default().fg(color))),
                Cell::from(truncate_str(&endpoint, 80)),
            ]);
            if i == app.scroll {
                row.style(Style::default().bg(theme.selected_bg).fg(theme.selected_fg))
            } else {
                row
            }
        })
        .collect();

    let block = btop_block(
        &format!("mcp servers ({})", app.mcp_servers.len()),
        "",
        theme.net_box,
        theme,
    );
    let table = Table::new(rows, [
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(50),
    ])
    .header(header)
    .block(block);
    f.render_widget(table, area);
}

// ── metrics ──────────────────────────────────────────────────────────────────
pub(super) fn draw_metrics(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let m = &app.metrics;
    let cpu = val_f64(m, "cpu_usage_percent");
    let mem_used = val_u64(m, "memory_used_mb");
    let mem_total = val_u64(m, "memory_total_mb").max(1);
    let mem_pct = (mem_used as f64 / mem_total as f64 * 100.0).min(100.0);
    let active_agents = val_u64(m, "active_agents");
    let active_sessions = val_u64(m, "active_sessions");
    let uptime = val_u64(m, "uptime_secs");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    stat_cards(f, app, chunks[0], &[
        ("CPU", format!("{:.1}%", cpu), theme.hi_fg),
        ("Memory", format!("{}M / {}M", mem_used, mem_total), theme.proc_misc),
        ("Agents", format!("{}", active_agents), theme.warning_fg),
        ("Sessions", format!("{}", active_sessions), theme.main_fg),
    ]);

    let cpu_grad = make_gradient(theme.cpu_grad.start, theme.cpu_grad.mid, theme.cpu_grad.end);
    let mem_grad = make_gradient(theme.used_grad.start, theme.used_grad.mid, theme.used_grad.end);

    let bar_w = (area.width as usize).saturating_sub(20).clamp(10, 80);
    let mut cpu_line = vec![Span::styled(" CPU    ", Style::default().fg(theme.graph_text))];
    cpu_line.extend(meter_bar(cpu, bar_w, &cpu_grad, theme.meter_bg));
    cpu_line.push(Span::styled(format!(" {:.1}%", cpu), Style::default().fg(theme.main_fg)));

    let mut mem_line = vec![Span::styled(" Memory ", Style::default().fg(theme.graph_text))];
    mem_line.extend(meter_bar(mem_pct, bar_w, &mem_grad, theme.meter_bg));
    mem_line.push(Span::styled(format!(" {:.1}%", mem_pct), Style::default().fg(theme.main_fg)));

    let uptime_str = format!(
        "{}h {:02}m {:02}s",
        uptime / 3600,
        (uptime % 3600) / 60,
        uptime % 60
    );

    let mut lines = vec![
        Line::from(cpu_line),
        Line::from(mem_line),
        Line::from(""),
        Line::from(vec![
            Span::styled(" uptime ", Style::default().fg(theme.graph_text)),
            Span::styled(uptime_str, Style::default().fg(theme.main_fg)),
            Span::styled("  error rate ", Style::default().fg(theme.graph_text)),
            Span::styled(
                format!("{:.2}%", val_f64(m, "error_rate") * 100.0),
                Style::default().fg(theme.main_fg),
            ),
        ]),
    ];

    let hw = &app.hardware;
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " HARDWARE",
        Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  cpu ", Style::default().fg(theme.graph_text)),
        Span::styled(val_str(hw, "cpu_brand"), Style::default().fg(theme.main_fg)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ram ", Style::default().fg(theme.graph_text)),
        Span::styled(
            format!("{} GB", val_u64(hw, "total_ram_mb") / 1024),
            Style::default().fg(theme.main_fg),
        ),
    ]));
    let gpu_name = hw
        .get("gpu")
        .and_then(|g| g.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("none");
    lines.push(Line::from(vec![
        Span::styled("  gpu ", Style::default().fg(theme.graph_text)),
        Span::styled(gpu_name.to_string(), Style::default().fg(theme.main_fg)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  backend ", Style::default().fg(theme.graph_text)),
        Span::styled(val_str(hw, "backend").to_uppercase(), Style::default().fg(theme.hi_fg)),
    ]));

    let block = btop_block("metrics", "", theme.cpu_box, theme);
    f.render_widget(Paragraph::new(lines).block(block), chunks[1]);
}

fn fmt_ctx(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}k", n / 1_000)
    } else {
        format!("{}", n)
    }
}

fn fmt_downloads(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
