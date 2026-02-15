use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
}

impl SessionStatus {
    pub fn icon(&self, tick: u64) -> &'static str {
        match self {
            SessionStatus::Active => {
                if tick.is_multiple_of(2) {
                    "◉"
                } else {
                    "○"
                }
            }
            SessionStatus::Completed => "●",
            SessionStatus::Failed => "✗",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SessionStatus::Active => "Active",
            SessionStatus::Completed => "Completed",
            SessionStatus::Failed => "Failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Status,
    Agent,
    Duration,
    Tokens,
    Cost,
    Started,
}

impl SortColumn {
    pub fn next(&self) -> Self {
        match self {
            SortColumn::Status => SortColumn::Agent,
            SortColumn::Agent => SortColumn::Duration,
            SortColumn::Duration => SortColumn::Tokens,
            SortColumn::Tokens => SortColumn::Cost,
            SortColumn::Cost => SortColumn::Started,
            SortColumn::Started => SortColumn::Status,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortColumn::Status => "Status",
            SortColumn::Agent => "Agent",
            SortColumn::Duration => "Duration",
            SortColumn::Tokens => "Tokens",
            SortColumn::Cost => "Cost",
            SortColumn::Started => "Started",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: &'static str,
    pub agent: &'static str,
    pub status: SessionStatus,
    pub duration_secs: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub started: &'static str,
    pub model: &'static str,
    pub task_summary: &'static str,
}

impl SessionInfo {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn format_duration(&self) -> String {
        let hours = self.duration_secs / 3600;
        let minutes = (self.duration_secs % 3600) / 60;
        let seconds = self.duration_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    None,
    Agent(&'static str),
    Status(SessionStatus),
}

pub struct SessionsViewState {
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub filter: FilterType,
    pub tick: u64,
}

impl Default for SessionsViewState {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Started,
            sort_ascending: false,
            filter: FilterType::None,
            tick: 0,
        }
    }
}

pub struct SessionsView;

impl SessionsView {
    pub fn get_sessions() -> Vec<SessionInfo> {
        vec![
            SessionInfo {
                id: "sess_001",
                agent: "Claude Code",
                status: SessionStatus::Active,
                duration_secs: 932,
                input_tokens: 8_450,
                output_tokens: 4_000,
                cost: 0.25,
                started: "14:30:21",
                model: "claude-3-opus",
                task_summary: "Implementing TUI sessions view",
            },
            SessionInfo {
                id: "sess_002",
                agent: "Claude Code",
                status: SessionStatus::Active,
                duration_secs: 525,
                input_tokens: 5_230,
                output_tokens: 3_000,
                cost: 0.17,
                started: "15:45:12",
                model: "claude-3-sonnet",
                task_summary: "Code review for PR #42",
            },
            SessionInfo {
                id: "sess_003",
                agent: "OpenCode",
                status: SessionStatus::Active,
                duration_secs: 201,
                input_tokens: 2_820,
                output_tokens: 1_300,
                cost: 0.08,
                started: "16:12:34",
                model: "gpt-4-turbo",
                task_summary: "Debugging async runtime issue",
            },
            SessionInfo {
                id: "sess_004",
                agent: "Claude Code",
                status: SessionStatus::Completed,
                duration_secs: 2712,
                input_tokens: 25_200,
                output_tokens: 10_000,
                cost: 0.72,
                started: "Yesterday",
                model: "claude-3-opus",
                task_summary: "Full project refactoring",
            },
            SessionInfo {
                id: "sess_005",
                agent: "OpenCode",
                status: SessionStatus::Completed,
                duration_secs: 1335,
                input_tokens: 12_400,
                output_tokens: 6_500,
                cost: 0.38,
                started: "Yesterday",
                model: "gpt-4-turbo",
                task_summary: "API integration testing",
            },
            SessionInfo {
                id: "sess_006",
                agent: "Codex",
                status: SessionStatus::Completed,
                duration_secs: 630,
                input_tokens: 3_200,
                output_tokens: 2_000,
                cost: 0.12,
                started: "3 days ago",
                model: "code-davinci-002",
                task_summary: "Legacy code migration",
            },
            SessionInfo {
                id: "sess_007",
                agent: "Cursor",
                status: SessionStatus::Failed,
                duration_secs: 45,
                input_tokens: 500,
                output_tokens: 0,
                cost: 0.01,
                started: "2 days ago",
                model: "gpt-4",
                task_summary: "Connection timeout during request",
            },
            SessionInfo {
                id: "sess_008",
                agent: "Claude Code",
                status: SessionStatus::Completed,
                duration_secs: 1890,
                input_tokens: 18_500,
                output_tokens: 8_200,
                cost: 0.55,
                started: "4 days ago",
                model: "claude-3-opus",
                task_summary: "Documentation generation",
            },
        ]
    }

    pub fn sessions_count() -> usize {
        Self::get_sessions().len()
    }

    fn get_sorted_sessions(
        sort_column: SortColumn,
        ascending: bool,
        filter: FilterType,
    ) -> Vec<SessionInfo> {
        let mut sessions = Self::get_sessions();

        match filter {
            FilterType::None => {}
            FilterType::Agent(agent) => {
                sessions.retain(|s| s.agent == agent);
            }
            FilterType::Status(status) => {
                sessions.retain(|s| s.status == status);
            }
        }

        sessions.sort_by(|a, b| {
            let cmp = match sort_column {
                SortColumn::Status => {
                    let status_order = |s: &SessionStatus| match s {
                        SessionStatus::Active => 0,
                        SessionStatus::Completed => 1,
                        SessionStatus::Failed => 2,
                    };
                    status_order(&a.status).cmp(&status_order(&b.status))
                }
                SortColumn::Agent => a.agent.cmp(b.agent),
                SortColumn::Duration => a.duration_secs.cmp(&b.duration_secs),
                SortColumn::Tokens => a.total_tokens().cmp(&b.total_tokens()),
                SortColumn::Cost => a
                    .cost
                    .partial_cmp(&b.cost)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Started => {
                    let time_order = |s: &str| match s {
                        s if s.contains(':') => 0,
                        "Yesterday" => 1,
                        "2 days ago" => 2,
                        "3 days ago" => 3,
                        "4 days ago" => 4,
                        _ => 5,
                    };
                    time_order(a.started).cmp(&time_order(b.started))
                }
            };
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });

        sessions
    }

    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(10)])
            .split(area);

        Self::render_quick_stats(frame, chunks[0], app);
        Self::render_sessions_table(frame, chunks[1], app);
    }

    fn render_quick_stats(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let sessions = Self::get_sessions();

        let active_count = sessions
            .iter()
            .filter(|s| s.status == SessionStatus::Active)
            .count();
        let completed_count = sessions
            .iter()
            .filter(|s| s.status == SessionStatus::Completed)
            .count();
        let failed_count = sessions
            .iter()
            .filter(|s| s.status == SessionStatus::Failed)
            .count();

        let total_tokens: u64 = sessions.iter().map(|s| s.total_tokens()).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.cost).sum();

        let _active_duration: u64 = sessions
            .iter()
            .filter(|s| s.status == SessionStatus::Active)
            .map(|s| s.duration_secs)
            .sum();

        let stats_block = Block::default()
            .title(" Session Summary ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = stats_block.inner(area);
        frame.render_widget(stats_block, area);

        let stat_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(inner);

        let stat_items = [
            ("Active", format!("{}", active_count), theme.success()),
            (
                "Completed",
                format!("{}", completed_count),
                theme.foreground(),
            ),
            (
                "Failed",
                format!("{}", failed_count),
                if failed_count > 0 {
                    theme.error()
                } else {
                    theme.foreground_dim()
                },
            ),
            ("Total Tokens", format_number(total_tokens), theme.accent()),
            ("Total Cost", format!("${:.2}", total_cost), theme.success()),
        ];

        for (i, (label, value, color)) in stat_items.iter().enumerate() {
            let stat_lines = vec![
                Line::from(Span::styled(
                    *label,
                    Style::default().fg(theme.foreground_dim()),
                )),
                Line::from(Span::styled(
                    value.clone(),
                    Style::default().fg(*color).add_modifier(Modifier::BOLD),
                )),
            ];
            let paragraph = Paragraph::new(stat_lines)
                .alignment(Alignment::Center)
                .style(Style::default().bg(theme.surface()));
            frame.render_widget(paragraph, stat_chunks[i]);
        }
    }

    fn render_sessions_table(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();

        let sort_column = SortColumn::Started;
        let sort_ascending = false;
        let filter = FilterType::None;
        let tick = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 500) as u64;

        let sessions = Self::get_sorted_sessions(sort_column, sort_ascending, filter);
        let selected_index = app
            .state
            .selected_index
            .min(sessions.len().saturating_sub(1));

        let sort_indicator = |col: SortColumn| -> String {
            if col == sort_column {
                if sort_ascending { " ↑" } else { " ↓" }.to_string()
            } else {
                String::new()
            }
        };

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from(Span::raw(format!(
                "Status{}",
                sort_indicator(SortColumn::Status)
            ))),
            Cell::from(Span::raw(format!(
                "Agent{}",
                sort_indicator(SortColumn::Agent)
            ))),
            Cell::from(Span::raw(format!(
                "Duration{}",
                sort_indicator(SortColumn::Duration)
            ))),
            Cell::from(Span::raw(format!(
                "Tokens{}",
                sort_indicator(SortColumn::Tokens)
            ))),
            Cell::from(Span::raw(format!(
                "Cost{}",
                sort_indicator(SortColumn::Cost)
            ))),
            Cell::from(Span::raw(format!(
                "Started{}",
                sort_indicator(SortColumn::Started)
            ))),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = i == selected_index;
                let is_active = session.status == SessionStatus::Active;

                let status_color = match session.status {
                    SessionStatus::Active => theme.success(),
                    SessionStatus::Completed => theme.foreground_dim(),
                    SessionStatus::Failed => theme.error(),
                };

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else if is_active {
                    Style::default().fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground_dim())
                };

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!("{} {}", session.status.icon(tick), session.status.label()),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        session.agent,
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        session.format_duration(),
                        if is_active {
                            Style::default().fg(theme.accent())
                        } else {
                            Style::default().fg(theme.foreground_dim())
                        },
                    )),
                    Cell::from(Span::styled(
                        format_number(session.total_tokens()),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("${:.2}", session.cost),
                        if session.cost > 0.5 {
                            Style::default().fg(theme.warning())
                        } else {
                            Style::default().fg(theme.success())
                        },
                    )),
                    Cell::from(Span::styled(
                        session.started,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Sessions ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("s ", Style::default().fg(theme.accent())),
                            Span::styled("sort  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("f ", Style::default().fg(theme.accent())),
                            Span::styled("filter ", Style::default().fg(theme.foreground_dim())),
                        ])
                        .alignment(Alignment::Center),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .row_highlight_style(Style::default().bg(theme.selection()));

        frame.render_widget(table, area);
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_labels() {
        assert_eq!(SessionStatus::Active.label(), "Active");
        assert_eq!(SessionStatus::Completed.label(), "Completed");
        assert_eq!(SessionStatus::Failed.label(), "Failed");
    }

    #[test]
    fn test_session_status_icons_toggle() {
        assert_eq!(SessionStatus::Active.icon(0), "◉");
        assert_eq!(SessionStatus::Active.icon(1), "○");
        assert_eq!(SessionStatus::Active.icon(2), "◉");
        assert_eq!(SessionStatus::Completed.icon(0), "●");
        assert_eq!(SessionStatus::Failed.icon(0), "✗");
    }

    #[test]
    fn test_format_duration() {
        let session = SessionInfo {
            id: "test",
            agent: "Test",
            status: SessionStatus::Active,
            duration_secs: 3665,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            started: "now",
            model: "test",
            task_summary: "test",
        };
        assert_eq!(session.format_duration(), "01:01:05");

        let short_session = SessionInfo {
            id: "test",
            agent: "Test",
            status: SessionStatus::Active,
            duration_secs: 125,
            input_tokens: 0,
            output_tokens: 0,
            cost: 0.0,
            started: "now",
            model: "test",
            task_summary: "test",
        };
        assert_eq!(short_session.format_duration(), "02:05");
    }

    #[test]
    fn test_total_tokens() {
        let session = SessionInfo {
            id: "test",
            agent: "Test",
            status: SessionStatus::Active,
            duration_secs: 0,
            input_tokens: 5000,
            output_tokens: 3000,
            cost: 0.0,
            started: "now",
            model: "test",
            task_summary: "test",
        };
        assert_eq!(session.total_tokens(), 8000);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(85_432), "85.4K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_sessions_count() {
        let count = SessionsView::sessions_count();
        assert!(count >= 5);
    }

    #[test]
    fn test_sort_column_cycle() {
        let col = SortColumn::Status;
        assert_eq!(col.next(), SortColumn::Agent);
        assert_eq!(col.next().next(), SortColumn::Duration);
    }

    #[test]
    fn test_get_sorted_sessions_filter_agent() {
        let sessions = SessionsView::get_sorted_sessions(
            SortColumn::Started,
            false,
            FilterType::Agent("Claude Code"),
        );
        assert!(sessions.iter().all(|s| s.agent == "Claude Code"));
    }

    #[test]
    fn test_get_sorted_sessions_filter_status() {
        let sessions = SessionsView::get_sorted_sessions(
            SortColumn::Started,
            false,
            FilterType::Status(SessionStatus::Active),
        );
        assert!(sessions.iter().all(|s| s.status == SessionStatus::Active));
    }
}
