mod agents;
mod context;
mod footer;
mod header;
mod home;
mod mcp;
mod quota;
mod sessions;
mod tokens;
mod views;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::state::{App, View};

pub const MIN_WIDTH: u16 = 100;
pub const MIN_HEIGHT: u16 = 24;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let w = area.width;
    let h = area.height;
    let theme = app.theme();

    if w < MIN_WIDTH || h < MIN_HEIGHT {
        let msg = vec![
            Line::from(Span::styled(
                "Terminal too small for rimuru-tui",
                Style::default().fg(theme.main_fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("current {}x{}  needed {}x{}", w, h, MIN_WIDTH, MIN_HEIGHT),
                Style::default().fg(theme.graph_text),
            )),
        ];
        let y = h / 2;
        let r = Rect { x: 0, y, width: w, height: 2.min(h.saturating_sub(y)) };
        f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), r);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    header::draw(f, app, chunks[0]);

    match app.view {
        View::Home => home::draw(f, app, chunks[1]),
        View::Costs => views::draw_costs(f, app, chunks[1]),
        View::Budget => views::draw_budget(f, app, chunks[1]),
        View::Models => views::draw_models(f, app, chunks[1]),
        View::Advisor => views::draw_advisor(f, app, chunks[1]),
        View::Context => views::draw_context(f, app, chunks[1]),
        View::McpProxy => views::draw_mcp_proxy(f, app, chunks[1]),
        View::Hooks => views::draw_hooks(f, app, chunks[1]),
        View::Plugins => views::draw_plugins(f, app, chunks[1]),
        View::Mcp => views::draw_mcp(f, app, chunks[1]),
        View::Metrics => views::draw_metrics(f, app, chunks[1]),
    }

    footer::draw(f, app, chunks[2]);
}
