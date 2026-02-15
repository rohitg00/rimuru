use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

use crate::app::App;
use rimuru_core::skillkit::{InstalledSkill, Skill, SkillKitAgent, SkillRecommendation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillsTab {
    #[default]
    Installed,
    Marketplace,
    Recommendations,
}

impl SkillsTab {
    pub fn all() -> &'static [SkillsTab] {
        &[
            SkillsTab::Installed,
            SkillsTab::Marketplace,
            SkillsTab::Recommendations,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            SkillsTab::Installed => "Installed",
            SkillsTab::Marketplace => "Marketplace",
            SkillsTab::Recommendations => "Recommendations",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SkillsTab::Installed => SkillsTab::Marketplace,
            SkillsTab::Marketplace => SkillsTab::Recommendations,
            SkillsTab::Recommendations => SkillsTab::Installed,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SkillsTab::Installed => SkillsTab::Recommendations,
            SkillsTab::Marketplace => SkillsTab::Installed,
            SkillsTab::Recommendations => SkillsTab::Marketplace,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            SkillsTab::Installed => 0,
            SkillsTab::Marketplace => 1,
            SkillsTab::Recommendations => 2,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SkillsViewState {
    pub current_tab: SkillsTab,
    pub search_query: Option<String>,
    pub is_searching: bool,
    pub installed_skills: Vec<InstalledSkill>,
    pub marketplace_skills: Vec<Skill>,
    pub recommendations: Vec<SkillRecommendation>,
}

impl SkillsViewState {
    pub fn new() -> Self {
        Self {
            current_tab: SkillsTab::Installed,
            search_query: None,
            is_searching: false,
            installed_skills: Self::get_mock_installed(),
            marketplace_skills: Self::get_mock_marketplace(),
            recommendations: Self::get_mock_recommendations(),
        }
    }

    fn get_mock_installed() -> Vec<InstalledSkill> {
        vec![
            InstalledSkill::new(
                Skill::new(
                    "pro-workflow",
                    "Battle-tested Claude Code workflows from power users",
                )
                .with_tags(vec!["workflow".to_string(), "productivity".to_string()])
                .with_agents(vec![SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor]),
                "~/.skillkit/skills/pro-workflow",
                vec![SkillKitAgent::ClaudeCode],
            ),
            InstalledSkill::new(
                Skill::new(
                    "meta-cognitive-prompting",
                    "Framework for multi-step problem solving",
                )
                .with_tags(vec!["prompting".to_string(), "reasoning".to_string()])
                .with_agents(vec![SkillKitAgent::ClaudeCode, SkillKitAgent::GeminiCli]),
                "~/.skillkit/skills/meta-cognitive-prompting",
                vec![SkillKitAgent::ClaudeCode],
            ),
            InstalledSkill::new(
                Skill::new(
                    "social-content-writer",
                    "Write social media content with Dan Koe style",
                )
                .with_tags(vec!["writing".to_string(), "social".to_string()])
                .with_agents(vec![SkillKitAgent::ClaudeCode]),
                "~/.skillkit/skills/social-content-writer",
                vec![SkillKitAgent::ClaudeCode],
            ),
        ]
    }

    fn get_mock_marketplace() -> Vec<Skill> {
        vec![
            Skill::new(
                "tdd-workflow",
                "Test-driven development workflow with 80%+ coverage",
            )
            .with_tags(vec!["testing".to_string(), "tdd".to_string()])
            .with_agents(vec![
                SkillKitAgent::ClaudeCode,
                SkillKitAgent::Cursor,
                SkillKitAgent::Codex,
            ]),
            Skill::new(
                "django-patterns",
                "Django architecture patterns and REST API design",
            )
            .with_tags(vec![
                "python".to_string(),
                "django".to_string(),
                "api".to_string(),
            ])
            .with_agents(vec![SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor]),
            Skill::new(
                "golang-patterns",
                "Idiomatic Go patterns and best practices",
            )
            .with_tags(vec!["golang".to_string(), "go".to_string()])
            .with_agents(vec![SkillKitAgent::ClaudeCode, SkillKitAgent::OpenCode]),
            Skill::new(
                "security-review",
                "Comprehensive security checklist and patterns",
            )
            .with_tags(vec!["security".to_string(), "owasp".to_string()])
            .with_agents(vec![SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor]),
            Skill::new(
                "postgres-patterns",
                "PostgreSQL query optimization and schema design",
            )
            .with_tags(vec![
                "database".to_string(),
                "postgres".to_string(),
                "sql".to_string(),
            ])
            .with_agents(vec![SkillKitAgent::ClaudeCode]),
            Skill::new("frontend-patterns", "React, Next.js, and UI best practices")
                .with_tags(vec![
                    "react".to_string(),
                    "frontend".to_string(),
                    "nextjs".to_string(),
                ])
                .with_agents(vec![
                    SkillKitAgent::ClaudeCode,
                    SkillKitAgent::Cursor,
                    SkillKitAgent::Windsurf,
                ]),
        ]
    }

    fn get_mock_recommendations() -> Vec<SkillRecommendation> {
        vec![
            SkillRecommendation::new(
                Skill::new("e2e-runner", "End-to-end testing with Playwright").with_tags(vec![
                    "testing".to_string(),
                    "e2e".to_string(),
                    "playwright".to_string(),
                ]),
                "Based on your recent testing workflows",
                0.92,
            ),
            SkillRecommendation::new(
                Skill::new("code-reviewer", "Expert code review specialist")
                    .with_tags(vec!["review".to_string(), "quality".to_string()]),
                "You frequently make code changes",
                0.88,
            ),
            SkillRecommendation::new(
                Skill::new("doc-updater", "Documentation and codemap specialist")
                    .with_tags(vec!["documentation".to_string(), "docs".to_string()]),
                "Keep your docs in sync with code",
                0.75,
            ),
        ]
    }
}

pub struct SkillsView;

impl SkillsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, skills_state: &SkillsViewState) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(10),
            ])
            .split(area);

        Self::render_tabs(frame, chunks[0], app, skills_state);
        Self::render_quick_stats(frame, chunks[1], app, skills_state);

        match skills_state.current_tab {
            SkillsTab::Installed => Self::render_installed_tab(frame, chunks[2], app, skills_state),
            SkillsTab::Marketplace => {
                Self::render_marketplace_tab(frame, chunks[2], app, skills_state)
            }
            SkillsTab::Recommendations => {
                Self::render_recommendations_tab(frame, chunks[2], app, skills_state)
            }
        }
    }

    fn render_tabs(frame: &mut Frame, area: Rect, app: &App, skills_state: &SkillsViewState) {
        let theme = app.current_theme();

        let tab_titles: Vec<Line> = SkillsTab::all()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let style = if i == skills_state.current_tab.index() {
                    Style::default()
                        .fg(theme.accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.foreground_dim())
                };
                Line::styled(tab.name(), style)
            })
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .title(" Skills ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border()))
                    .style(Style::default().bg(theme.surface())),
            )
            .select(skills_state.current_tab.index())
            .highlight_style(
                Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | ");

        frame.render_widget(tabs, area);
    }

    fn render_quick_stats(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        skills_state: &SkillsViewState,
    ) {
        let theme = app.current_theme();

        let installed_count = skills_state.installed_skills.len();
        let enabled_count = skills_state
            .installed_skills
            .iter()
            .filter(|s| s.enabled)
            .count();
        let marketplace_count = skills_state.marketplace_skills.len();
        let recommendations_count = skills_state.recommendations.len();

        let stats_block = Block::default()
            .title(" Quick Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = stats_block.inner(area);
        frame.render_widget(stats_block, area);

        let stat_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(inner);

        let stat_items = [
            ("Installed", format!("{}", installed_count), theme.accent()),
            ("Enabled", format!("{}", enabled_count), theme.success()),
            (
                "Available",
                format!("{}+", marketplace_count),
                theme.foreground(),
            ),
            (
                "Suggested",
                format!("{}", recommendations_count),
                theme.warning(),
            ),
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

    fn render_installed_tab(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        skills_state: &SkillsViewState,
    ) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(skills_state.installed_skills.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Status"),
            Cell::from("Skill"),
            Cell::from("Agents"),
            Cell::from("Tags"),
            Cell::from("Path"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = skills_state
            .installed_skills
            .iter()
            .enumerate()
            .map(|(i, installed)| {
                let is_selected = i == selected_index;
                let status_color = if installed.enabled {
                    theme.success()
                } else {
                    theme.foreground_dim()
                };
                let status_icon = if installed.enabled { "●" } else { "○" };

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let agents: String = installed
                    .installed_for
                    .iter()
                    .map(|a| agent_icon(a))
                    .collect::<Vec<_>>()
                    .join(" ");

                let tags = installed
                    .skill
                    .tags
                    .iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        format!(
                            "{} {}",
                            status_icon,
                            if installed.enabled {
                                "Enabled"
                            } else {
                                "Disabled"
                            }
                        ),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(Span::styled(
                        installed.skill.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(agents),
                    Cell::from(Span::styled(
                        tags,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        truncate_path(&installed.path, 30),
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
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Length(32),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Installed Skills ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("i ", Style::default().fg(theme.accent())),
                            Span::styled("install  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("t ", Style::default().fg(theme.accent())),
                            Span::styled(
                                "translate  ",
                                Style::default().fg(theme.foreground_dim()),
                            ),
                            Span::styled("/ ", Style::default().fg(theme.accent())),
                            Span::styled("search ", Style::default().fg(theme.foreground_dim())),
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

    fn render_marketplace_tab(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        skills_state: &SkillsViewState,
    ) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(skills_state.marketplace_skills.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Skill"),
            Cell::from("Description"),
            Cell::from("Agents"),
            Cell::from("Tags"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = skills_state
            .marketplace_skills
            .iter()
            .enumerate()
            .map(|(i, skill)| {
                let is_selected = i == selected_index;

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let agents: String = skill
                    .agents
                    .iter()
                    .take(3)
                    .map(|a| agent_icon(a))
                    .collect::<Vec<_>>()
                    .join(" ");

                let more_agents = if skill.agents.len() > 3 {
                    format!("+{}", skill.agents.len() - 3)
                } else {
                    String::new()
                };

                let tags = skill
                    .tags
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        skill.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        truncate_str(&skill.description, 40),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(format!("{} {}", agents, more_agents)),
                    Cell::from(Span::styled(
                        tags,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(20),
            Constraint::Min(30),
            Constraint::Length(16),
            Constraint::Length(20),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" Marketplace ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("i ", Style::default().fg(theme.accent())),
                            Span::styled("install  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("/ ", Style::default().fg(theme.accent())),
                            Span::styled("search ", Style::default().fg(theme.foreground_dim())),
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

    fn render_recommendations_tab(
        frame: &mut Frame,
        area: Rect,
        app: &App,
        skills_state: &SkillsViewState,
    ) {
        let theme = app.current_theme();
        let selected_index = app
            .state
            .selected_index
            .min(skills_state.recommendations.len().saturating_sub(1));

        let header = Row::new(vec![
            Cell::from(""),
            Cell::from("Confidence"),
            Cell::from("Skill"),
            Cell::from("Reason"),
            Cell::from("Tags"),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = skills_state
            .recommendations
            .iter()
            .enumerate()
            .map(|(i, rec)| {
                let is_selected = i == selected_index;

                let selection_indicator = if is_selected { "▶" } else { " " };

                let row_style = if is_selected {
                    Style::default()
                        .bg(theme.selection())
                        .fg(theme.foreground())
                } else {
                    Style::default().fg(theme.foreground())
                };

                let confidence_color = if rec.confidence >= 0.9 {
                    theme.success()
                } else if rec.confidence >= 0.7 {
                    theme.warning()
                } else {
                    theme.foreground_dim()
                };

                let confidence_bar = format!(
                    "{} {:>3}%",
                    confidence_bar(rec.confidence),
                    (rec.confidence * 100.0) as u32
                );

                let tags = rec
                    .skill
                    .tags
                    .iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");

                Row::new([
                    Cell::from(Span::styled(
                        selection_indicator,
                        Style::default().fg(theme.accent()),
                    )),
                    Cell::from(Span::styled(
                        confidence_bar,
                        Style::default().fg(confidence_color),
                    )),
                    Cell::from(Span::styled(
                        rec.skill.name.clone(),
                        if is_selected {
                            Style::default()
                                .fg(theme.accent())
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.foreground())
                        },
                    )),
                    Cell::from(Span::styled(
                        truncate_str(&rec.reason, 35),
                        Style::default().fg(theme.foreground_dim()),
                    )),
                    Cell::from(Span::styled(
                        tags,
                        Style::default().fg(theme.foreground_dim()),
                    )),
                ])
                .style(row_style)
                .height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(14),
            Constraint::Length(20),
            Constraint::Min(30),
            Constraint::Length(16),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .title(" AI Recommendations ")
                    .title_bottom(
                        Line::from(vec![
                            Span::styled(" j/k ", Style::default().fg(theme.accent())),
                            Span::styled("navigate  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("Enter ", Style::default().fg(theme.accent())),
                            Span::styled("details  ", Style::default().fg(theme.foreground_dim())),
                            Span::styled("i ", Style::default().fg(theme.accent())),
                            Span::styled("install  ", Style::default().fg(theme.foreground_dim())),
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

    pub fn installed_count(skills_state: &SkillsViewState) -> usize {
        skills_state.installed_skills.len()
    }

    pub fn marketplace_count(skills_state: &SkillsViewState) -> usize {
        skills_state.marketplace_skills.len()
    }

    pub fn recommendations_count(skills_state: &SkillsViewState) -> usize {
        skills_state.recommendations.len()
    }

    pub fn current_list_len(skills_state: &SkillsViewState) -> usize {
        match skills_state.current_tab {
            SkillsTab::Installed => skills_state.installed_skills.len(),
            SkillsTab::Marketplace => skills_state.marketplace_skills.len(),
            SkillsTab::Recommendations => skills_state.recommendations.len(),
        }
    }
}

fn agent_icon(agent: &SkillKitAgent) -> &'static str {
    match agent {
        SkillKitAgent::ClaudeCode => "⟁",
        SkillKitAgent::Cursor => "◫",
        SkillKitAgent::Codex => "◎",
        SkillKitAgent::GeminiCli => "✦",
        SkillKitAgent::OpenCode => "◇",
        SkillKitAgent::GithubCopilot => "◆",
        SkillKitAgent::Windsurf => "⌘",
        SkillKitAgent::Goose => "◈",
        SkillKitAgent::Cline => "◉",
        _ => "○",
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 2 {
            return truncate_str(path, max_len);
        }
        let last_parts = parts[parts.len() - 2..].join("/");
        format!(".../{}", last_parts)
    }
}

fn confidence_bar(confidence: f32) -> String {
    let filled = (confidence * 5.0).round() as usize;
    let empty = 5 - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_tab_navigation() {
        assert_eq!(SkillsTab::Installed.next(), SkillsTab::Marketplace);
        assert_eq!(SkillsTab::Marketplace.next(), SkillsTab::Recommendations);
        assert_eq!(SkillsTab::Recommendations.next(), SkillsTab::Installed);

        assert_eq!(SkillsTab::Installed.prev(), SkillsTab::Recommendations);
        assert_eq!(SkillsTab::Marketplace.prev(), SkillsTab::Installed);
        assert_eq!(SkillsTab::Recommendations.prev(), SkillsTab::Marketplace);
    }

    #[test]
    fn test_skills_tab_index() {
        assert_eq!(SkillsTab::Installed.index(), 0);
        assert_eq!(SkillsTab::Marketplace.index(), 1);
        assert_eq!(SkillsTab::Recommendations.index(), 2);
    }

    #[test]
    fn test_skills_view_state_new() {
        let state = SkillsViewState::new();
        assert_eq!(state.current_tab, SkillsTab::Installed);
        assert!(!state.installed_skills.is_empty());
        assert!(!state.marketplace_skills.is_empty());
        assert!(!state.recommendations.is_empty());
    }

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world test", 10), "hello w...");
    }

    #[test]
    fn test_confidence_bar() {
        assert_eq!(confidence_bar(1.0), "█████");
        assert_eq!(confidence_bar(0.6), "███░░");
        assert_eq!(confidence_bar(0.0), "░░░░░");
    }

    #[test]
    fn test_agent_icon() {
        assert_eq!(agent_icon(&SkillKitAgent::ClaudeCode), "⟁");
        assert_eq!(agent_icon(&SkillKitAgent::Cursor), "◫");
        assert_eq!(agent_icon(&SkillKitAgent::Codex), "◎");
    }

    #[test]
    fn test_current_list_len() {
        let state = SkillsViewState::new();
        assert_eq!(
            SkillsView::current_list_len(&state),
            state.installed_skills.len()
        );
    }
}
