use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Style,
    widgets::Block,
    Frame,
};

use crate::app::{App, View};
use crate::ui::views::{
    AgentDetailsView, AgentsView, CostDetailType, CostDetailsView, CostsView, DashboardView,
    HelpView, HooksView, MetricsView, PluginsView, SessionDetailsView, SessionsView,
    SkillDetailsView, SkillsView,
};
use crate::ui::widgets::{Footer, Header, HelpModal};

pub struct MainLayout;

impl MainLayout {
    pub fn render(frame: &mut Frame, app: &App) {
        let theme = app.current_theme();
        let size = frame.area();

        frame.render_widget(
            Block::default().style(
                Style::default()
                    .bg(theme.background())
                    .fg(theme.foreground()),
            ),
            size,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(size);

        Header::render(frame, chunks[0], app);

        let content_area = chunks[1].inner(Margin::new(1, 0));

        match app.current_view {
            View::Dashboard => DashboardView::render(frame, content_area, app),
            View::Agents => AgentsView::render(frame, content_area, app),
            View::AgentDetails(index) => AgentDetailsView::render(frame, content_area, app, index),
            View::Sessions => SessionsView::render(frame, content_area, app),
            View::SessionDetails(index) => {
                SessionDetailsView::render(frame, content_area, app, index)
            }
            View::Costs => CostsView::render(frame, content_area, app),
            View::CostDetails(index) => {
                CostDetailsView::render(frame, content_area, app, CostDetailType::Agent(index))
            }
            View::Metrics => MetricsView::render(frame, content_area, app),
            View::Skills => SkillsView::render(frame, content_area, app, &app.skills_state),
            View::SkillDetails(index) => {
                if let Some(source) =
                    SkillDetailsView::get_skill_from_state(&app.skills_state, index)
                {
                    SkillDetailsView::render(frame, content_area, app, &source);
                } else {
                    SkillsView::render(frame, content_area, app, &app.skills_state);
                }
            }
            View::Plugins | View::PluginDetails(_) => {
                PluginsView::render(frame, content_area, app, &app.plugins_state)
            }
            View::Hooks => HooksView::render(frame, content_area, app, &app.hooks_state),
            View::Help => HelpView::render(frame, content_area, app),
        }

        Footer::render(frame, chunks[2], app);

        app.toast_manager.render(frame, size, theme);

        if let Some(ref dialog) = app.dialog_state.dialog {
            dialog.render(frame, size, theme);
        }

        if app.show_help_modal {
            let mut help_modal = HelpModal::create(theme);
            for _ in 0..app.help_modal_scroll {
                help_modal.scroll_down(20);
            }
            help_modal.render(frame, size, theme);
        }
    }

    pub fn create_three_column_layout(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(area)
            .to_vec()
    }

    pub fn create_two_column_layout(area: Rect, left_percent: u16) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(left_percent),
                Constraint::Percentage(100 - left_percent),
            ])
            .split(area)
            .to_vec()
    }
}
