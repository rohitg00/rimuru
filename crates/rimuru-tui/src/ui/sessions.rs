use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Paragraph, Row, Table};

use crate::state::{App, SessionStatusKind};
use crate::widgets::{btop_block, fmt_tokens, grad_at, make_gradient, truncate_str};

pub(super) fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let block = btop_block("sessions", "⁵", theme.proc_box, theme);
    f.render_widget(block, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    let session_rows: u16 = app.sessions.len() as u16 * 2;
    let detail_reserve: u16 = 8.min(inner.height / 2);
    let max_table = inner.height.saturating_sub(detail_reserve);
    let table_h = (1 + session_rows).min(max_table);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(table_h),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    {
        let sep = "─".repeat(chunks[1].width as usize);
        f.render_widget(
            Paragraph::new(Span::styled(sep, Style::default().fg(theme.proc_box))),
            chunks[1],
        );
    }

    draw_table(f, app, chunks[0]);
    draw_detail(f, app, chunks[2]);
}

fn draw_table(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let proc_grad = make_gradient(theme.proc_grad.start, theme.proc_grad.mid, theme.proc_grad.end);

    let w = area.width;
    let show_msgs = w >= 110;
    let show_cost = w >= 100;

    let project_w: u16 = if w >= 120 { 16 } else if w >= 100 { 12 } else { 8 };
    let session_w: u16 = if w >= 110 { 9 } else { 6 };
    let status_w: u16 = 8;
    let model_w: u16 = if w >= 110 { 14 } else { 10 };
    let tokens_w: u16 = 7;
    let ctx_w: u16 = 6;
    let cost_w: u16 = 7;
    let msgs_w: u16 = 5;

    let mut rows: Vec<Row> = Vec::new();
    for (i, s) in app.sessions.iter().enumerate() {
        let selected = i == app.selected;
        let marker = if selected { "►" } else { " " };

        let (status_icon, status_color) = match s.status {
            SessionStatusKind::Active => ("● Work", theme.proc_misc),
            SessionStatusKind::Completed => ("✓ Done", theme.inactive_fg),
            SessionStatusKind::Abandoned => ("◌ Wait", grad_at(&proc_grad, 50.0)),
            SessionStatusKind::Error => ("✗ Err ", theme.status_fg),
            SessionStatusKind::Unknown => ("? None", theme.inactive_fg),
        };

        let ctx_color = grad_at(&proc_grad, s.context_pct);
        let agent_color = agent_color_for(&s.agent_type, theme);

        let row_style = if selected {
            Style::default()
                .bg(theme.selected_bg)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD)
        } else if matches!(s.status, SessionStatusKind::Completed | SessionStatusKind::Abandoned) {
            Style::default().fg(theme.inactive_fg)
        } else {
            Style::default()
        };

        let mut cells = vec![
            Cell::from(Span::styled(marker, Style::default().fg(theme.hi_fg))),
            Cell::from(Span::styled(s.agent_label.clone(), Style::default().fg(agent_color))),
            Cell::from(Span::styled(
                truncate_str(&s.project_name, project_w as usize),
                Style::default().fg(theme.title),
            )),
            Cell::from(Span::styled(
                truncate_str(&s.short_id, session_w as usize),
                Style::default().fg(theme.session_id),
            )),
            Cell::from(Span::styled(status_icon.to_string(), Style::default().fg(status_color))),
            Cell::from(Span::styled(
                truncate_str(&s.model_short, model_w as usize),
                Style::default().fg(if s.model_short == "-" { theme.inactive_fg } else { theme.graph_text }),
            )),
            Cell::from(Span::styled(
                format!("{:.0}%", s.context_pct),
                Style::default().fg(ctx_color),
            )),
            Cell::from(Span::styled(
                fmt_tokens(s.total_tokens),
                Style::default().fg(theme.main_fg),
            )),
        ];
        if show_cost {
            cells.push(Cell::from(Span::styled(
                format!("${:.2}", s.total_cost),
                Style::default().fg(theme.graph_text),
            )));
        }
        if show_msgs {
            cells.push(Cell::from(Span::styled(
                format!("{}", s.messages),
                Style::default().fg(theme.graph_text),
            )));
        }

        rows.push(Row::new(cells).style(row_style).height(1));

        let path_row: Vec<Cell> = (0..(8 + show_cost as usize + show_msgs as usize))
            .map(|j| {
                if j == 2 {
                    let path = if s.project_path.is_empty() { "—" } else { s.project_path.as_str() };
                    let w_col = (project_w + session_w) as usize;
                    Cell::from(Span::styled(
                        format!("└─ {}", truncate_str(path, w_col)),
                        Style::default().fg(theme.graph_text),
                    ))
                } else {
                    Cell::from("")
                }
            })
            .collect();
        rows.push(Row::new(path_row).height(1));
    }

    let header_style = Style::default().fg(theme.main_fg).add_modifier(Modifier::BOLD);
    let mut header_cells = vec![
        Cell::from(""),
        Cell::from(Span::styled("AI", header_style)),
        Cell::from(Span::styled("Project", header_style)),
        Cell::from(Span::styled("Session", header_style)),
        Cell::from(Span::styled("Status", header_style)),
        Cell::from(Span::styled("Model", header_style)),
        Cell::from(Span::styled("Ctx", header_style)),
        Cell::from(Span::styled("Tokens", header_style)),
    ];
    if show_cost {
        header_cells.push(Cell::from(Span::styled("Cost", header_style)));
    }
    if show_msgs {
        header_cells.push(Cell::from(Span::styled("Msgs", header_style)));
    }

    let mut widths: Vec<Constraint> = vec![
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(project_w),
        Constraint::Length(session_w),
        Constraint::Length(status_w),
        Constraint::Length(model_w),
        Constraint::Length(ctx_w),
        Constraint::Length(tokens_w),
    ];
    if show_cost {
        widths.push(Constraint::Length(cost_w));
    }
    if show_msgs {
        widths.push(Constraint::Length(msgs_w));
    }

    let total_rows = app.sessions.len() * 2;
    let visible_rows = area.height.saturating_sub(1) as usize;
    let selected_row_end = app.selected * 2 + 2;
    let scroll_offset = selected_row_end.saturating_sub(visible_rows);

    let visible: Vec<Row> = if scroll_offset < rows.len() {
        rows.into_iter().skip(scroll_offset).collect()
    } else {
        Vec::new()
    };

    let (table_area, sb_area) = if total_rows > visible_rows && area.width > 2 {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        (split[0], Some(split[1]))
    } else {
        (area, None)
    };

    let table = Table::new(visible, widths).header(Row::new(header_cells));
    f.render_widget(table, table_area);

    if let Some(sb) = sb_area {
        draw_scrollbar(f, sb, total_rows, visible_rows, scroll_offset, theme);
    }
}

fn draw_scrollbar(
    f: &mut Frame,
    area: Rect,
    total: usize,
    visible: usize,
    offset: usize,
    theme: &crate::theme::Theme,
) {
    let bar_h = area.height as usize;
    if bar_h == 0 || total == 0 {
        return;
    }
    let thumb = ((visible as f64 / total as f64) * bar_h as f64).ceil().max(1.0) as usize;
    let thumb = thumb.min(bar_h);
    let thumb_pos = if total > visible {
        ((offset as f64 / (total - visible) as f64) * (bar_h - thumb) as f64).round() as usize
    } else {
        0
    };
    let buf = f.buffer_mut();
    for i in 0..bar_h {
        let y = area.y + i as u16;
        let (ch, color) = if i >= thumb_pos && i < thumb_pos + thumb {
            ("┃", theme.main_fg)
        } else {
            ("│", theme.div_line)
        };
        buf[(area.x, y)].set_symbol(ch).set_fg(color);
    }
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    if area.height < 3 {
        return;
    }
    let Some(s) = app.selected_session() else {
        return;
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(" SESSION (►{} · {})", s.short_id, s.project_path),
        Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
    )));

    let started = if s.started_at.len() >= 19 {
        &s.started_at[..19]
    } else {
        s.started_at.as_str()
    };
    let elapsed = elapsed_from(&s.started_at, s.ended_at.as_deref());

    lines.push(Line::from(vec![
        Span::styled("  agent ", Style::default().fg(theme.graph_text)),
        Span::styled(s.agent_label.clone(), Style::default().fg(agent_color_for(&s.agent_type, theme))),
        Span::styled(
            format!(" {}", s.agent_type),
            Style::default().fg(theme.main_fg),
        ),
        Span::styled(
            format!("  · started {} · {}", started, elapsed),
            Style::default().fg(theme.graph_text),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  model ", Style::default().fg(theme.graph_text)),
        Span::styled(
            if s.model.is_empty() { "—" } else { s.model.as_str() },
            Style::default().fg(theme.main_fg),
        ),
        Span::styled(
            format!("  · ctx {:.0}%", s.context_pct),
            Style::default().fg(theme.graph_text),
        ),
        Span::styled(
            format!("  · cost ${:.3}", s.total_cost),
            Style::default().fg(theme.graph_text),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  tokens ", Style::default().fg(theme.graph_text)),
        Span::styled(fmt_tokens(s.total_tokens), Style::default().fg(theme.main_fg)),
        Span::styled(
            format!("  in {}  out {}  cacheR {}  cacheW {}",
                fmt_tokens(s.input_tokens),
                fmt_tokens(s.output_tokens),
                fmt_tokens(s.cache_read),
                fmt_tokens(s.cache_write)),
            Style::default().fg(theme.graph_text),
        ),
    ]));

    let top = top_context_consumers(app, &s.id);
    if !top.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " TOP CONSUMERS",
            Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
        )));
        for (name, pct) in top.iter().take(4) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:>5.1}%  ", pct),
                    Style::default().fg(theme.main_fg),
                ),
                Span::styled(
                    truncate_str(name, area.width.saturating_sub(15) as usize),
                    Style::default().fg(theme.graph_text),
                ),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn top_context_consumers(app: &App, session_id: &str) -> Vec<(String, f64)> {
    let Some(arr) = app.context_utilization.get("utilizations").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let Some(entry) = arr.iter().find(|e| {
        e.get("session_id").and_then(|x| x.as_str()) == Some(session_id)
    }) else {
        return Vec::new();
    };
    let Some(breakdown) = entry.get("breakdown").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out: Vec<(String, f64)> = breakdown
        .iter()
        .filter_map(|b| {
            let name = b.get("name").or_else(|| b.get("tool")).and_then(|x| x.as_str())?;
            let pct = b.get("percent").or_else(|| b.get("pct")).and_then(|x| x.as_f64())?;
            Some((name.to_string(), pct))
        })
        .collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

fn elapsed_from(started_at: &str, ended_at: Option<&str>) -> String {
    let Ok(start) = chrono::DateTime::parse_from_rfc3339(started_at) else {
        return String::from("—");
    };
    let end = ended_at
        .and_then(|e| chrono::DateTime::parse_from_rfc3339(e).ok())
        .map(|d| d.with_timezone(&chrono::Utc))
        .unwrap_or_else(chrono::Utc::now);
    let dur = end.signed_duration_since(start.with_timezone(&chrono::Utc));
    let secs = dur.num_seconds().max(0);
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {:02}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

fn agent_color_for(agent_type: &str, theme: &crate::theme::Theme) -> Color {
    let t = agent_type.to_lowercase();
    if t.contains("claude") {
        Color::Rgb(217, 119, 87)
    } else if t.contains("codex") {
        Color::Rgb(122, 157, 255)
    } else if t.contains("cursor") {
        Color::Rgb(140, 220, 220)
    } else if t.contains("copilot") || t.contains("github") {
        Color::Rgb(180, 180, 180)
    } else if t.contains("gemini") {
        Color::Rgb(100, 200, 255)
    } else if t.contains("opencode") {
        Color::Rgb(160, 220, 120)
    } else {
        theme.main_fg
    }
}

