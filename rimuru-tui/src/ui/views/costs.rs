use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
    Frame,
};

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeRange {
    #[default]
    Today,
    Week,
    Month,
    Custom,
}

impl TimeRange {
    pub fn label(&self) -> &'static str {
        match self {
            TimeRange::Today => "Today",
            TimeRange::Week => "This Week",
            TimeRange::Month => "This Month",
            TimeRange::Custom => "Custom",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            TimeRange::Today => TimeRange::Week,
            TimeRange::Week => TimeRange::Month,
            TimeRange::Month => TimeRange::Custom,
            TimeRange::Custom => TimeRange::Today,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            TimeRange::Today => TimeRange::Custom,
            TimeRange::Week => TimeRange::Today,
            TimeRange::Month => TimeRange::Week,
            TimeRange::Custom => TimeRange::Month,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrillDownType {
    Agent(&'static str),
    Model(&'static str),
}

#[derive(Debug, Clone)]
pub struct AgentCostInfo {
    pub name: &'static str,
    pub cost: f64,
    pub tokens: u64,
    pub sessions: u32,
    pub trend: CostTrend,
}

#[derive(Debug, Clone)]
pub struct ModelCostInfo {
    pub name: &'static str,
    pub cost: f64,
    pub tokens: u64,
    pub sessions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostTrend {
    Up,
    Down,
    Stable,
}

impl CostTrend {
    pub fn icon(&self) -> &'static str {
        match self {
            CostTrend::Up => "↑",
            CostTrend::Down => "↓",
            CostTrend::Stable => "→",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CostsViewState {
    pub time_range: TimeRange,
    pub selected_tab: usize,
    pub agent_selected_index: usize,
    pub model_selected_index: usize,
    pub tick: u64,
    pub running_total_offset: f64,
}

pub struct CostsView;

impl CostsView {
    pub fn get_agent_costs(time_range: TimeRange) -> Vec<AgentCostInfo> {
        let multiplier = match time_range {
            TimeRange::Today => 1.0,
            TimeRange::Week => 7.0,
            TimeRange::Month => 30.0,
            TimeRange::Custom => 15.0,
        };

        vec![
            AgentCostInfo {
                name: "Claude Code",
                cost: 1.82 * multiplier,
                tokens: (85_432.0 * multiplier) as u64,
                sessions: (12.0 * multiplier) as u32,
                trend: CostTrend::Up,
            },
            AgentCostInfo {
                name: "OpenCode",
                cost: 0.43 * multiplier,
                tokens: (21_500.0 * multiplier) as u64,
                sessions: (5.0 * multiplier) as u32,
                trend: CostTrend::Down,
            },
            AgentCostInfo {
                name: "Codex",
                cost: 0.12 * multiplier,
                tokens: (6_200.0 * multiplier) as u64,
                sessions: (3.0 * multiplier) as u32,
                trend: CostTrend::Stable,
            },
            AgentCostInfo {
                name: "Copilot",
                cost: 0.08 * multiplier,
                tokens: (4_300.0 * multiplier) as u64,
                sessions: (2.0 * multiplier) as u32,
                trend: CostTrend::Down,
            },
        ]
    }

    pub fn get_model_costs(time_range: TimeRange) -> Vec<ModelCostInfo> {
        let multiplier = match time_range {
            TimeRange::Today => 1.0,
            TimeRange::Week => 7.0,
            TimeRange::Month => 30.0,
            TimeRange::Custom => 15.0,
        };

        vec![
            ModelCostInfo {
                name: "claude-3.5-sonnet",
                cost: 1.20 * multiplier,
                tokens: (60_000.0 * multiplier) as u64,
                sessions: (8.0 * multiplier) as u32,
            },
            ModelCostInfo {
                name: "claude-3-haiku",
                cost: 0.62 * multiplier,
                tokens: (42_000.0 * multiplier) as u64,
                sessions: (6.0 * multiplier) as u32,
            },
            ModelCostInfo {
                name: "gpt-4-turbo",
                cost: 0.43 * multiplier,
                tokens: (18_500.0 * multiplier) as u64,
                sessions: (4.0 * multiplier) as u32,
            },
            ModelCostInfo {
                name: "gpt-3.5-turbo",
                cost: 0.20 * multiplier,
                tokens: (15_000.0 * multiplier) as u64,
                sessions: (4.0 * multiplier) as u32,
            },
        ]
    }

    pub fn get_sparkline_data(time_range: TimeRange) -> Vec<u64> {
        match time_range {
            TimeRange::Today => vec![5, 8, 12, 15, 10, 18, 22, 14, 25, 20, 28, 24],
            TimeRange::Week => vec![10, 15, 8, 22, 30, 25, 28],
            TimeRange::Month => vec![50, 65, 80, 75, 90, 85, 95, 88, 100, 92, 110, 105, 115, 120],
            TimeRange::Custom => vec![30, 45, 38, 52, 60, 55, 58, 62, 70, 65],
        }
    }

    pub fn agent_count() -> usize {
        4
    }

    pub fn model_count() -> usize {
        4
    }

    pub fn render(frame: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        Self::render_time_range_selector(frame, chunks[0], app);
        Self::render_main_content(frame, chunks[1], app);
    }

    fn render_time_range_selector(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let time_range = TimeRange::Today;

        let ranges = [
            TimeRange::Today,
            TimeRange::Week,
            TimeRange::Month,
            TimeRange::Custom,
        ];
        let tabs: Vec<Span> = ranges
            .iter()
            .map(|r| {
                let is_selected = *r == time_range;
                if is_selected {
                    Span::styled(
                        format!(" [{}] ", r.label()),
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        format!("  {}  ", r.label()),
                        Style::default().fg(theme.foreground_dim()),
                    )
                }
            })
            .collect();

        let block = Block::default()
            .title(" Time Range ")
            .title_bottom(
                Line::from(vec![
                    Span::styled(" ← ", Style::default().fg(theme.accent())),
                    Span::styled("prev  ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("→ ", Style::default().fg(theme.accent())),
                    Span::styled("next  ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("Tab ", Style::default().fg(theme.accent())),
                    Span::styled("switch panel ", Style::default().fg(theme.foreground_dim())),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let tabs_line = Line::from(tabs);
        let tabs_para = Paragraph::new(tabs_line)
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme.surface()));
        frame.render_widget(tabs_para, inner);
    }

    fn render_main_content(frame: &mut Frame, area: Rect, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        Self::render_summary_panel(frame, chunks[0], app);
        Self::render_breakdown_panel(frame, chunks[1], app);
    }

    fn render_summary_panel(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let time_range = TimeRange::Today;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Min(6),
            ])
            .split(area);

        let agent_costs = Self::get_agent_costs(time_range);
        let total_cost: f64 = agent_costs.iter().map(|a| a.cost).sum();
        let total_tokens: u64 = agent_costs.iter().map(|a| a.tokens).sum();
        let total_sessions: u32 = agent_costs.iter().map(|a| a.sessions).sum();

        let tick = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            / 100) as u64;

        let animated_cost = total_cost + (tick % 10) as f64 * 0.001;

        let today_block = Block::default()
            .title(format!(" {} Total ", time_range.label()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));
        let today_inner = today_block.inner(chunks[0]);
        frame.render_widget(today_block, chunks[0]);

        let cost_display = format!("${:.2}", animated_cost);
        let today_content = vec![
            Line::from(""),
            Line::from(Span::styled(
                &cost_display,
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{} tokens", format_number(total_tokens)),
                Style::default().fg(theme.foreground_dim()),
            )),
            Line::from(Span::styled(
                format!("{} sessions", total_sessions),
                Style::default().fg(theme.foreground_dim()),
            )),
        ];

        let today_para = Paragraph::new(today_content)
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme.surface()));
        frame.render_widget(today_para, today_inner);

        let periods_block = Block::default()
            .title(" Cost Comparison ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));
        let periods_inner = periods_block.inner(chunks[1]);
        frame.render_widget(periods_block, chunks[1]);

        let base_cost = 2.45;
        let periods = [
            ("Today", format!("${:.2}", base_cost), CostTrend::Up),
            (
                "This Week",
                format!("${:.2}", base_cost * 6.24),
                CostTrend::Down,
            ),
            (
                "This Month",
                format!("${:.2}", base_cost * 21.55),
                CostTrend::Up,
            ),
            (
                "All Time",
                format!("${:.2}", base_cost * 95.6),
                CostTrend::Stable,
            ),
        ];

        let period_lines: Vec<Line> = periods
            .iter()
            .map(|(period, cost, trend)| {
                let trend_color = match trend {
                    CostTrend::Up => theme.error(),
                    CostTrend::Down => theme.success(),
                    CostTrend::Stable => theme.foreground_dim(),
                };
                Line::from(vec![
                    Span::styled(
                        format!("  {:<12}", period),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                    Span::styled(
                        format!("{:<10}", cost),
                        Style::default().fg(theme.foreground()),
                    ),
                    Span::styled(
                        trend.icon(),
                        Style::default()
                            .fg(trend_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            })
            .collect();

        let periods_para = Paragraph::new(period_lines).style(Style::default().bg(theme.surface()));
        frame.render_widget(periods_para, periods_inner);

        let trend_data = Self::get_sparkline_data(time_range);
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(" {} Trend ", time_range.label()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .data(&trend_data)
            .style(Style::default().fg(theme.accent()));

        frame.render_widget(sparkline, chunks[2]);
    }

    fn render_breakdown_panel(frame: &mut Frame, area: Rect, app: &App) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        Self::render_agent_breakdown(frame, chunks[0], app);
        Self::render_model_breakdown(frame, chunks[1], app);
    }

    fn render_agent_breakdown(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let time_range = TimeRange::Today;
        let selected_index = app.state.selected_index;

        let agent_costs = Self::get_agent_costs(time_range);
        let total_cost: f64 = agent_costs.iter().map(|a| a.cost).sum();

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from(Span::raw("Agent")),
            Cell::from(Span::raw("Cost")),
            Cell::from(Span::raw("%")),
            Cell::from(Span::raw("Trend")),
            Cell::from(Span::raw("Bar")),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = agent_costs
            .iter()
            .enumerate()
            .map(|(i, agent)| {
                let is_selected = i == selected_index % agent_costs.len();
                let pct = (agent.cost / total_cost * 100.0) as u32;
                let bar_len = (pct as usize).min(20);
                let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);

                let selection_indicator = if is_selected { "▶" } else { " " };
                let trend_color = match agent.trend {
                    CostTrend::Up => theme.error(),
                    CostTrend::Down => theme.success(),
                    CostTrend::Stable => theme.foreground_dim(),
                };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        agent.name,
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        format!("${:.2}", agent.cost),
                        Style::default().fg(theme.success()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}%", pct),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        agent.trend.icon(),
                        Style::default()
                            .fg(trend_color)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Cell::from(Span::styled(bar, Style::default().fg(theme.accent()))),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Min(12),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(22),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Cost by Agent ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled(
                                "drill-down ",
                                Style::default().fg(theme.foreground_dim()),
                            ),
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

    fn render_model_breakdown(frame: &mut Frame, area: Rect, app: &App) {
        let theme = app.current_theme();
        let time_range = TimeRange::Today;

        let model_costs = Self::get_model_costs(time_range);
        let total_cost: f64 = model_costs.iter().map(|m| m.cost).sum();

        let header = Row::new(vec![
            Cell::from(Span::raw("Model")),
            Cell::from(Span::raw("Cost")),
            Cell::from(Span::raw("%")),
            Cell::from(Span::raw("Sessions")),
            Cell::from(Span::raw("Bar")),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = model_costs
            .iter()
            .map(|model| {
                let pct = (model.cost / total_cost * 100.0) as u32;
                let bar_len = (pct as usize).min(20);
                let bar = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);

                Row::new([
                    Cell::from(Span::styled(
                        model.name,
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("${:.2}", model.cost),
                        Style::default().fg(theme.success()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}%", pct),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        format!("{}", model.sessions),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(bar, Style::default().fg(theme.warning()))),
                ])
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Min(18),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(10),
            Constraint::Length(22),
        ];

        let table = Table::new(rows, widths).header(header).block(
            Block::default()
                .title(" Cost by Model ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border()))
                .style(Style::default().bg(theme.surface())),
        );

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
    fn test_time_range_label() {
        assert_eq!(TimeRange::Today.label(), "Today");
        assert_eq!(TimeRange::Week.label(), "This Week");
        assert_eq!(TimeRange::Month.label(), "This Month");
        assert_eq!(TimeRange::Custom.label(), "Custom");
    }

    #[test]
    fn test_time_range_cycle() {
        assert_eq!(TimeRange::Today.next(), TimeRange::Week);
        assert_eq!(TimeRange::Week.next(), TimeRange::Month);
        assert_eq!(TimeRange::Month.next(), TimeRange::Custom);
        assert_eq!(TimeRange::Custom.next(), TimeRange::Today);
    }

    #[test]
    fn test_time_range_cycle_prev() {
        assert_eq!(TimeRange::Today.prev(), TimeRange::Custom);
        assert_eq!(TimeRange::Week.prev(), TimeRange::Today);
        assert_eq!(TimeRange::Month.prev(), TimeRange::Week);
        assert_eq!(TimeRange::Custom.prev(), TimeRange::Month);
    }

    #[test]
    fn test_cost_trend_icon() {
        assert_eq!(CostTrend::Up.icon(), "↑");
        assert_eq!(CostTrend::Down.icon(), "↓");
        assert_eq!(CostTrend::Stable.icon(), "→");
    }

    #[test]
    fn test_get_agent_costs() {
        let today_costs = CostsView::get_agent_costs(TimeRange::Today);
        let week_costs = CostsView::get_agent_costs(TimeRange::Week);

        assert_eq!(today_costs.len(), 4);
        assert_eq!(week_costs.len(), 4);

        assert!(week_costs[0].cost > today_costs[0].cost);
    }

    #[test]
    fn test_get_model_costs() {
        let costs = CostsView::get_model_costs(TimeRange::Today);
        assert_eq!(costs.len(), 4);
        assert_eq!(costs[0].name, "claude-3.5-sonnet");
    }

    #[test]
    fn test_get_sparkline_data() {
        let today_data = CostsView::get_sparkline_data(TimeRange::Today);
        let week_data = CostsView::get_sparkline_data(TimeRange::Week);

        assert_eq!(today_data.len(), 12);
        assert_eq!(week_data.len(), 7);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(85_432), "85.4K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_agent_count() {
        assert_eq!(CostsView::agent_count(), 4);
    }

    #[test]
    fn test_model_count() {
        assert_eq!(CostsView::model_count(), 4);
    }
}
