use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::App;
use crate::widgets::{btop_block, fmt_tokens, truncate_str};

pub(super) fn draw(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let header_style = Style::default().fg(theme.main_fg).add_modifier(Modifier::BOLD);

    let mut lines = vec![Line::from(vec![
        Span::styled(" SERVER   ", header_style),
        Span::styled("TOOLS", header_style),
    ])];

    let tools_by_server = tools_per_server(app);

    for server in app.mcp_servers.iter().take(area.height.saturating_sub(3) as usize) {
        let name = server.get("name").and_then(|v| v.as_str()).unwrap_or("?");
        let status = server.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let tools = tools_by_server
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        let is_connected = matches!(status, "connected" | "running" | "active" | "ready");
        let color = if is_connected { theme.proc_misc } else { theme.inactive_fg };
        let dot = if is_connected { "●" } else { "·" };
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", dot), Style::default().fg(color)),
            Span::styled(
                truncate_str(name, 10),
                Style::default().fg(theme.main_fg),
            ),
            Span::styled(
                format!(" {:>3}", tools),
                Style::default().fg(theme.graph_text),
            ),
        ]));
    }

    let proxy_total = app
        .mcp_proxy_stats
        .get("total_tools")
        .or_else(|| app.mcp_proxy_stats.get("tool_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let proxy_calls = app
        .mcp_proxy_stats
        .get("total_calls")
        .or_else(|| app.mcp_proxy_stats.get("call_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if lines.len() <= 1 {
        lines.push(Line::from(Span::styled(
            " no mcp servers",
            Style::default().fg(theme.inactive_fg),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" proxy ", Style::default().fg(theme.graph_text)),
        Span::styled(
            format!("{} tools ", fmt_tokens(proxy_total)),
            Style::default().fg(theme.main_fg),
        ),
        Span::styled(
            format!("{} calls", fmt_tokens(proxy_calls)),
            Style::default().fg(theme.proc_misc),
        ),
    ]));

    let block = btop_block("mcp", "⁴", theme.net_box, theme);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn tools_per_server(app: &App) -> Vec<(String, usize)> {
    let Some(tools) = app.mcp_proxy_stats.get("tools").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for t in tools {
        let server = t.get("server").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        *counts.entry(server).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}
