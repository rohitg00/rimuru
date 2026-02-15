use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;
use rimuru_core::skillkit::{InstalledSkill, Skill, SkillKitAgent, SkillRecommendation};

use super::skills::SkillsViewState;

#[derive(Debug, Clone)]
pub enum SkillDetailSource {
    Installed(InstalledSkill),
    Marketplace(Skill),
    Recommendation(SkillRecommendation),
}

impl SkillDetailSource {
    pub fn skill(&self) -> &Skill {
        match self {
            SkillDetailSource::Installed(i) => &i.skill,
            SkillDetailSource::Marketplace(s) => s,
            SkillDetailSource::Recommendation(r) => &r.skill,
        }
    }

    pub fn is_installed(&self) -> bool {
        matches!(self, SkillDetailSource::Installed(_))
    }

    pub fn installed_agents(&self) -> Option<&[SkillKitAgent]> {
        if let SkillDetailSource::Installed(i) = self {
            Some(&i.installed_for)
        } else {
            None
        }
    }
}

pub struct SkillDetailsView;

impl SkillDetailsView {
    pub fn render(frame: &mut Frame, area: Rect, app: &App, source: &SkillDetailSource) {
        let _theme = app.current_theme();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Min(5),
            ])
            .split(area);

        Self::render_header_info(frame, chunks[0], app, source);
        Self::render_compatibility(frame, chunks[1], app, source);
        Self::render_actions(frame, chunks[2], app, source);
    }

    fn render_header_info(frame: &mut Frame, area: Rect, app: &App, source: &SkillDetailSource) {
        let theme = app.current_theme();
        let skill = source.skill();

        let status_text = if source.is_installed() {
            "● Installed"
        } else {
            "○ Not Installed"
        };
        let status_color = if source.is_installed() {
            theme.success()
        } else {
            theme.foreground_dim()
        };

        let block = Block::default()
            .title(format!(" {} ", skill.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        let left_info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    &skill.name,
                    Style::default()
                        .fg(theme.foreground())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Slug: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(&skill.slug, Style::default().fg(theme.foreground())),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(status_text, Style::default().fg(status_color)),
            ]),
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    skill.author.as_deref().unwrap_or("Unknown"),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    skill.version.as_deref().unwrap_or("1.0.0"),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
        ];

        let tags_str = if skill.tags.is_empty() {
            "None".to_string()
        } else {
            skill.tags.join(", ")
        };

        let right_info = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Tags: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(tags_str, Style::default().fg(theme.accent())),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Downloads: ", Style::default().fg(theme.foreground_dim())),
                Span::styled(
                    format_number(skill.downloads.unwrap_or(0)),
                    Style::default().fg(theme.foreground()),
                ),
            ]),
            Line::from(""),
        ];

        let left_para = Paragraph::new(left_info).style(Style::default().bg(theme.surface()));
        let right_para = Paragraph::new(right_info).style(Style::default().bg(theme.surface()));

        frame.render_widget(left_para, info_chunks[0]);
        frame.render_widget(right_para, info_chunks[1]);
    }

    fn render_compatibility(frame: &mut Frame, area: Rect, app: &App, source: &SkillDetailSource) {
        let theme = app.current_theme();
        let skill = source.skill();
        let installed_for = source.installed_agents();

        let block = Block::default()
            .title(" Compatible Agents ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let agents = if skill.agents.is_empty() {
            SkillKitAgent::all().to_vec()
        } else {
            skill.agents.clone()
        };

        let header = Row::new(vec![
            Cell::from("Agent"),
            Cell::from("Status"),
            Cell::from(""),
        ])
        .style(
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

        let rows: Vec<Row> = agents
            .iter()
            .take(6)
            .map(|agent| {
                let is_installed = installed_for
                    .map(|agents| agents.contains(agent))
                    .unwrap_or(false);

                let status_icon = if is_installed { "●" } else { "○" };
                let status_text = if is_installed {
                    "Installed"
                } else {
                    "Available"
                };
                let status_color = if is_installed {
                    theme.success()
                } else {
                    theme.foreground_dim()
                };

                Row::new([
                    Cell::from(Span::styled(
                        format!("{} {}", agent_icon(agent), agent.display_name()),
                        Style::default().fg(theme.foreground()),
                    )),
                    Cell::from(Span::styled(
                        format!("{} {}", status_icon, status_text),
                        Style::default().fg(status_color),
                    )),
                    Cell::from(""),
                ])
                .height(1)
            })
            .collect();

        let _more_text = if agents.len() > 6 {
            format!(" +{} more agents ", agents.len() - 6)
        } else {
            String::new()
        };

        let widths = [
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ];

        let table = Table::new(rows, widths).header(header);

        frame.render_widget(table, inner);
    }

    fn render_actions(frame: &mut Frame, area: Rect, app: &App, source: &SkillDetailSource) {
        let theme = app.current_theme();

        let block = Block::default()
            .title(" Actions ")
            .title_bottom(
                Line::from(vec![
                    Span::styled(" Esc ", Style::default().fg(theme.accent())),
                    Span::styled("back  ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("i ", Style::default().fg(theme.accent())),
                    Span::styled("install  ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("t ", Style::default().fg(theme.accent())),
                    Span::styled("translate  ", Style::default().fg(theme.foreground_dim())),
                    Span::styled("u ", Style::default().fg(theme.accent())),
                    Span::styled("uninstall ", Style::default().fg(theme.foreground_dim())),
                ])
                .alignment(Alignment::Center),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.surface()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let actions = if source.is_installed() {
            vec![
                (" t  Translate ", "Translate skill to another agent"),
                (" u  Uninstall ", "Remove skill from current agent"),
                (" e  Enable/Disable ", "Toggle skill activation"),
                (" o  Open ", "Open skill in editor"),
            ]
        } else {
            vec![
                (" i  Install ", "Install skill for selected agent"),
                (" a  Install All ", "Install skill for all agents"),
                (" v  View Source ", "Open skill repository"),
            ]
        };

        if let SkillDetailSource::Recommendation(rec) = source {
            let rec_info = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Why this is recommended: ",
                    Style::default().fg(theme.accent()),
                )]),
                Line::from(vec![Span::styled(
                    &rec.reason,
                    Style::default().fg(theme.foreground()),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Confidence: ", Style::default().fg(theme.foreground_dim())),
                    Span::styled(
                        format!("{}%", (rec.confidence * 100.0) as u32),
                        Style::default().fg(if rec.confidence >= 0.8 {
                            theme.success()
                        } else {
                            theme.warning()
                        }),
                    ),
                ]),
            ];
            let rec_para = Paragraph::new(rec_info)
                .wrap(Wrap { trim: true })
                .style(Style::default().bg(theme.surface()));
            frame.render_widget(rec_para, inner);
            return;
        }

        let action_items: Vec<ListItem> = actions
            .iter()
            .map(|(key, desc)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        *key,
                        Style::default()
                            .fg(theme.accent())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" - {}", desc),
                        Style::default().fg(theme.foreground_dim()),
                    ),
                ]))
            })
            .collect();

        let actions_list = List::new(action_items).style(Style::default().bg(theme.surface()));

        frame.render_widget(actions_list, inner);
    }

    pub fn get_skill_from_state(
        skills_state: &SkillsViewState,
        index: usize,
    ) -> Option<SkillDetailSource> {
        use super::skills::SkillsTab;

        match skills_state.current_tab {
            SkillsTab::Installed => skills_state
                .installed_skills
                .get(index)
                .cloned()
                .map(SkillDetailSource::Installed),
            SkillsTab::Marketplace => skills_state
                .marketplace_skills
                .get(index)
                .cloned()
                .map(SkillDetailSource::Marketplace),
            SkillsTab::Recommendations => skills_state
                .recommendations
                .get(index)
                .cloned()
                .map(SkillDetailSource::Recommendation),
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
    fn test_skill_detail_source_skill() {
        let skill = Skill::new("test-skill", "A test skill");
        let source = SkillDetailSource::Marketplace(skill.clone());
        assert_eq!(source.skill().name, "test-skill");
        assert!(!source.is_installed());
    }

    #[test]
    fn test_skill_detail_source_installed() {
        let skill = Skill::new("test-skill", "A test skill");
        let installed =
            InstalledSkill::new(skill, "/path/to/skill", vec![SkillKitAgent::ClaudeCode]);
        let source = SkillDetailSource::Installed(installed);
        assert!(source.is_installed());
        assert!(source.installed_agents().is_some());
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(500), "500");
        assert_eq!(format_number(1_500), "1.5K");
        assert_eq!(format_number(1_500_000), "1.5M");
    }

    #[test]
    fn test_agent_icon() {
        assert_eq!(agent_icon(&SkillKitAgent::ClaudeCode), "⟁");
        assert_eq!(agent_icon(&SkillKitAgent::Cursor), "◫");
    }

    #[test]
    fn test_get_skill_from_state() {
        let state = SkillsViewState::new();
        let source = SkillDetailsView::get_skill_from_state(&state, 0);
        assert!(source.is_some());
        assert!(source.unwrap().is_installed());
    }
}
