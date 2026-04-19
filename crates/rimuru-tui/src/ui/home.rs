use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::state::App;

pub(super) fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let h = area.height;

    const CONTEXT_IDEAL: u16 = 8;
    const CONTEXT_MIN: u16 = 5;
    const MID_IDEAL: u16 = 7;
    const SESSIONS_MIN: u16 = 8;

    let context_h = if h >= CONTEXT_IDEAL + MID_IDEAL + SESSIONS_MIN {
        CONTEXT_IDEAL
    } else if h >= CONTEXT_MIN + MID_IDEAL + SESSIONS_MIN {
        CONTEXT_MIN
    } else {
        0
    };
    let mid_h = MID_IDEAL.min(h.saturating_sub(context_h + SESSIONS_MIN).max(MID_IDEAL));

    let mut constraints: Vec<Constraint> = Vec::new();
    if context_h > 0 {
        constraints.push(Constraint::Length(context_h));
    }
    constraints.push(Constraint::Length(mid_h));
    constraints.push(Constraint::Min(SESSIONS_MIN));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0;
    if context_h > 0 {
        super::context::draw(f, app, chunks[idx]);
        idx += 1;
    }

    let mid_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[idx]);

    super::quota::draw(f, app, mid_panels[0]);
    super::tokens::draw(f, app, mid_panels[1]);
    super::agents::draw(f, app, mid_panels[2]);
    super::mcp::draw(f, app, mid_panels[3]);
    idx += 1;

    super::sessions::draw(f, app, chunks[idx]);
}
