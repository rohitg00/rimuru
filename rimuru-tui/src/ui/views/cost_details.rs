use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
    Frame,
};

use super::costs::{CostTrend, CostsView, TimeRange};
use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostDetailType {
    Agent(usize),
    Model(usize),
}

#[derive(Debug, Clone)]
pub struct SessionCostInfo {
    pub id: &'static str,
    pub task: &'static str,
    pub duration_mins: u32,
    pub tokens: u64,
    pub cost: f64,
    pub time: &'static str,
}

pub struct CostDetailsView;

impl CostDetailsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, detail_type: CostDetailType) {
        match detail_type {
            CostDetailType::Agent(index) => Self::render_agent_details(frame, area, app, index),
            CostDetailType::Model(index) => Self::render_model_details(frame, area, app, index),
        }
    }

    fn render_agent_details(frame: &mut Frame, area: Rect, app: &App, agent_index: usize) {
        let _theme = app.current_theme();
        let agent_costs = CostsView::get_agent_costs(TimeRange::Today);

        let agent = agent_costs
            .get(agent_index)
            .cloned()
            .unwrap_or(super::costs::AgentCostInfo {
                name: "Unknown",
                cost: 0.0,
                tokens: 0,
                sessions: 0,
                trend: CostTrend::Stable,
            });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Min(10),
            ])
            .split(area);

        Self::render_agent_header(frame, chunks[0], app, &agent);
        Self::render_agent_stats(frame, chunks[1], app, &agent);
        Self::render_agent_sessions(frame, chunks[2], app, &agent);
    }

    fn render_agent_header(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        agent: &super::costs::AgentCostInfo,
    ) {
        let theme = app.current_theme();

        let trend_color = match agent.trend {
            CostTrend::Up => theme.error(),
            CostTrend::Down => theme.success(),
            CostTrend::Stable => theme.foreground_dim(),
        };

        let block = Block::default()
            .title(format!(" Agent: {} ", agent.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let left_info = vec![
            Line::from(vec![
                Span::styled("Agent: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    agent.name,
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Cost: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.2}", agent.cost),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Total Tokens: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(agent.tokens),
                    Style::default().fg(theme.accent()),
                ),
            ]),
        ];

        let right_info = vec![
            Line::from(vec![
                Span::styled("Sessions: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{}", agent.sessions),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Trend: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{} {}", agent.trend.icon(), trend_label(agent.trend)),
                    Style::default()
                        .fg(trend_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Avg Cost/Session: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format!("${:.4}", agent.cost / agent.sessions.max(1) as f64),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
        ];

        let left_para = Paragraph::new(left_info).style(Style::default().bg(theme.surface()));
        let right_para = Paragraph::new(right_info).style(Style::default().bg(theme.surface()));

        frame.render_widget(left_para, header_chunks[0]);
        frame.render_widget(right_para, header_chunks[1]);
    }

    fn render_agent_stats(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        agent: &super::costs::AgentCostInfo,
    ) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let cost_breakdown_block = Block::default()
            .title(" Cost Breakdown ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let cb_inner = cost_breakdown_block.inner(chunks[0]);
        frame.render_widget(cost_breakdown_block, chunks[0]);

        let input_cost = agent.cost * 0.35;
        let output_cost = agent.cost * 0.65;

        let breakdown_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Input:  ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.4}", input_cost),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(" (35%)", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(vec![
                Span::styled("  Output: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.4}", output_cost),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(" (65%)", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Total:  ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.4}", agent.cost),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let breakdown_para =
            Paragraph::new(breakdown_info).style(Style::default().bg(theme.surface()));
        frame.render_widget(breakdown_para, cb_inner);

        let hourly_data = get_hourly_cost_data(agent.name);
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(" Hourly Cost Trend ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .data(&hourly_data)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(sparkline, chunks[1]);
    }

    fn render_agent_sessions(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        agent: &super::costs::AgentCostInfo,
    ) {
        let theme = app.current_theme();

        let sessions = get_agent_sessions(agent.name);

        let header = Row::new(vec![
            Cell::from(Span::raw("Session ID")),
            Cell::from(Span::raw("Task")),
            Cell::from(Span::raw("Duration")),
            Cell::from(Span::raw("Tokens")),
            Cell::from(Span::raw("Cost")),
            Cell::from(Span::raw("Time")),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = sessions
            .iter()
            .map(|session| {
                Row::new([
                    Cell::from(Span::styled(
                        session.id,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        truncate_str(session.task, 25),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}m", session.duration_mins),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        format_number(session.tokens),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("${:.4}", session.cost),
                        Style::default().fg(theme.success()),
                    )),
                    Cell::from(Span::styled(
                        session.time,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(10),
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(" Session Breakdown ")
                .title_bottom(
                    Line::from(vec![
                        Span::styled(" Esc ", Style::default().fg(theme.accent())),
                        Span::styled(
                            "back to costs ",
                            Style::default().fg(theme.foreground_dim()),
                        ),
                    ])
                    .alignment(Alignment::Center),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.surface())),
        );

        frame.render_widget(table, area);
    }

    fn render_model_details(frame: &mut Frame, area: Rect, app: &App, model_index: usize) {
        let _theme = app.current_theme();
        let model_costs = CostsView::get_model_costs(TimeRange::Today);

        let model = model_costs
            .get(model_index)
            .cloned()
            .unwrap_or(super::costs::ModelCostInfo {
                name: "Unknown",
                cost: 0.0,
                tokens: 0,
                sessions: 0,
            });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Min(10),
            ])
            .split(area);

        Self::render_model_header(frame, chunks[0], app, &model);
        Self::render_model_stats(frame, chunks[1], app, &model);
        Self::render_model_usage(frame, chunks[2], app, &model);
    }

    fn render_model_header(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        model: &super::costs::ModelCostInfo,
    ) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(format!(" Model: {} ", model.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let pricing = get_model_pricing(model.name);

        let left_info = vec![
            Line::from(vec![
                Span::styled("Model: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    model.name,
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Cost: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.2}", model.cost),
                    Style::default()
                        .fg(theme.success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Total Tokens: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(model.tokens),
                    Style::default().fg(theme.accent()),
                ),
            ]),
        ];

        let right_info = vec![
            Line::from(vec![
                Span::styled("Sessions: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("{}", model.sessions),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Input Rate: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.6}/token", pricing.0),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Output Rate: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format!("${:.6}/token", pricing.1),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
        ];

        let left_para = Paragraph::new(left_info).style(Style::default().bg(theme.surface()));
        let right_para = Paragraph::new(right_info).style(Style::default().bg(theme.surface()));

        frame.render_widget(left_para, header_chunks[0]);
        frame.render_widget(right_para, header_chunks[1]);
    }

    fn render_model_stats(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        model: &super::costs::ModelCostInfo,
    ) {
        let theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let token_breakdown_block = Block::default()
            .title(" Token Breakdown ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let tb_inner = token_breakdown_block.inner(chunks[0]);
        frame.render_widget(token_breakdown_block, chunks[0]);

        let input_tokens = (model.tokens as f64 * 0.6) as u64;
        let output_tokens = (model.tokens as f64 * 0.4) as u64;

        let token_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Input Tokens:  ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(input_tokens),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(" (60%)", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Output Tokens: ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(output_tokens),
                    Style::default().fg(theme.foreground()),
                ),
                Span::styled(" (40%)", Style::default().fg(theme.foreground_dim())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Total Tokens:  ",
                    Style::default().fg(theme.foreground_dim()),
                ),
                Span::styled(
                    format_number(model.tokens),
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let token_para = Paragraph::new(token_info).style(Style::default().bg(theme.surface()));
        frame.render_widget(token_para, tb_inner);

        let usage_data = get_model_usage_data(model.name);
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(" Daily Usage Trend ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .data(&usage_data)
            .style(Style::default().fg(theme.warning()));

        frame.render_widget(sparkline, chunks[1]);
    }

    fn render_model_usage(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        model: &super::costs::ModelCostInfo,
    ) {
        let theme = app.current_theme();

        let agents_using = get_agents_using_model(model.name);

        let header = Row::new(vec![
            Cell::from(Span::raw("Agent")),
            Cell::from(Span::raw("Sessions")),
            Cell::from(Span::raw("Tokens")),
            Cell::from(Span::raw("Cost")),
            Cell::from(Span::raw("% of Model")),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let total_cost: f64 = agents_using.iter().map(|(_, _, _, c)| c).sum();

        let rows: Vec<Row> = agents_using
            .iter()
            .map(|(agent, sessions, tokens, cost)| {
                let pct = (*cost / total_cost * 100.0) as u32;
                Row::new([
                    Cell::from(Span::styled(
                        *agent,
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}", sessions),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        format_number(*tokens),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("${:.4}", cost),
                        Style::default().fg(theme.success()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}%", pct),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Min(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(" Agent Usage Breakdown ")
                .title_bottom(
                    Line::from(vec![
                        Span::styled(" Esc ", Style::default().fg(theme.accent())),
                        Span::styled(
                            "back to costs ",
                            Style::default().fg(theme.foreground_dim()),
                        ),
                    ])
                    .alignment(Alignment::Center),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.surface())),
        );

        frame.render_widget(table, area);
    }
}

fn get_hourly_cost_data(agent_name: &str) -> Vec<u64> {
    match agent_name {
        "Claude Code" => vec![5, 8, 12, 15, 10, 18, 22, 14, 25, 20, 28, 24],
        "OpenCode" => vec![3, 5, 8, 6, 10, 7, 12, 9, 8, 11, 6, 10],
        "Codex" => vec![2, 3, 5, 4, 6, 5, 7, 4, 5, 6, 3, 4],
        "Copilot" => vec![1, 2, 3, 2, 4, 3, 2, 3, 4, 2, 3, 2],
        _ => vec![1, 2, 3, 4, 3, 2, 1, 2, 3, 2, 1, 2],
    }
}

fn get_agent_sessions(agent_name: &str) -> Vec<SessionCostInfo> {
    match agent_name {
        "Claude Code" => vec![
            SessionCostInfo {
                id: "sess_001",
                task: "Implementing TUI costs view",
                duration_mins: 45,
                tokens: 12_450,
                cost: 0.25,
                time: "14:30",
            },
            SessionCostInfo {
                id: "sess_002",
                task: "Code review for PR #42",
                duration_mins: 22,
                tokens: 8_230,
                cost: 0.17,
                time: "15:45",
            },
            SessionCostInfo {
                id: "sess_003",
                task: "Refactoring event handlers",
                duration_mins: 38,
                tokens: 15_200,
                cost: 0.32,
                time: "16:30",
            },
            SessionCostInfo {
                id: "sess_004",
                task: "Documentation updates",
                duration_mins: 15,
                tokens: 5_500,
                cost: 0.12,
                time: "17:20",
            },
        ],
        "OpenCode" => vec![
            SessionCostInfo {
                id: "sess_010",
                task: "Debugging async runtime",
                duration_mins: 30,
                tokens: 9_820,
                cost: 0.18,
                time: "10:15",
            },
            SessionCostInfo {
                id: "sess_011",
                task: "API endpoint testing",
                duration_mins: 25,
                tokens: 7_400,
                cost: 0.14,
                time: "11:30",
            },
        ],
        _ => vec![SessionCostInfo {
            id: "sess_020",
            task: "General task",
            duration_mins: 20,
            tokens: 5_000,
            cost: 0.10,
            time: "09:00",
        }],
    }
}

fn get_model_pricing(model_name: &str) -> (f64, f64) {
    match model_name {
        "claude-3.5-sonnet" => (0.000003, 0.000015),
        "claude-3-haiku" => (0.00000025, 0.00000125),
        "gpt-4-turbo" => (0.00001, 0.00003),
        "gpt-3.5-turbo" => (0.0000005, 0.0000015),
        _ => (0.000001, 0.000002),
    }
}

fn get_model_usage_data(model_name: &str) -> Vec<u64> {
    match model_name {
        "claude-3.5-sonnet" => vec![15, 22, 18, 28, 35, 30, 25],
        "claude-3-haiku" => vec![20, 25, 32, 28, 35, 40, 38],
        "gpt-4-turbo" => vec![8, 12, 10, 15, 18, 14, 12],
        "gpt-3.5-turbo" => vec![25, 30, 28, 35, 40, 38, 32],
        _ => vec![5, 8, 6, 10, 12, 9, 8],
    }
}

fn get_agents_using_model(model_name: &str) -> Vec<(&'static str, u32, u64, f64)> {
    match model_name {
        "claude-3.5-sonnet" => vec![
            ("Claude Code", 8, 45_000, 0.90),
            ("Cursor", 2, 15_000, 0.30),
        ],
        "claude-3-haiku" => vec![
            ("Claude Code", 5, 35_000, 0.45),
            ("OpenCode", 1, 7_000, 0.09),
        ],
        "gpt-4-turbo" => vec![("OpenCode", 3, 12_000, 0.30), ("Copilot", 1, 6_500, 0.13)],
        "gpt-3.5-turbo" => vec![("Copilot", 3, 12_000, 0.15), ("Codex", 1, 3_000, 0.05)],
        _ => vec![("Unknown", 1, 1_000, 0.01)],
    }
}

fn trend_label(trend: CostTrend) -> &'static str {
    match trend {
        CostTrend::Up => "Increasing",
        CostTrend::Down => "Decreasing",
        CostTrend::Stable => "Stable",
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
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
    fn test_cost_detail_type() {
        let agent_detail = CostDetailType::Agent(0);
        let model_detail = CostDetailType::Model(1);

        assert_eq!(agent_detail, CostDetailType::Agent(0));
        assert_eq!(model_detail, CostDetailType::Model(1));
    }

    #[test]
    fn test_get_hourly_cost_data() {
        let claude_data = get_hourly_cost_data("Claude Code");
        let unknown_data = get_hourly_cost_data("Unknown");

        assert_eq!(claude_data.len(), 12);
        assert_eq!(unknown_data.len(), 12);
    }

    #[test]
    fn test_get_agent_sessions() {
        let claude_sessions = get_agent_sessions("Claude Code");
        let opencode_sessions = get_agent_sessions("OpenCode");

        assert_eq!(claude_sessions.len(), 4);
        assert_eq!(opencode_sessions.len(), 2);
    }

    #[test]
    fn test_get_model_pricing() {
        let (input, output) = get_model_pricing("claude-3.5-sonnet");
        assert!(input > 0.0);
        assert!(output > input);
    }

    #[test]
    fn test_trend_label() {
        assert_eq!(trend_label(CostTrend::Up), "Increasing");
        assert_eq!(trend_label(CostTrend::Down), "Decreasing");
        assert_eq!(trend_label(CostTrend::Stable), "Stable");
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("short", 10), "short");
        assert_eq!(truncate_str("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }
}
