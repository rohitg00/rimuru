use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::theme::Theme;

pub struct Logo;

impl Logo {
    pub fn ascii_art() -> &'static str {
        r#"
██████╗ ██╗███╗   ███╗██╗   ██╗██████╗ ██╗   ██╗
██╔══██╗██║████╗ ████║██║   ██║██╔══██╗██║   ██║
██████╔╝██║██╔████╔██║██║   ██║██████╔╝██║   ██║
██╔══██╗██║██║╚██╔╝██║██║   ██║██╔══██╗██║   ██║
██║  ██║██║██║ ╚═╝ ██║╚██████╔╝██║  ██║╚██████╔╝
╚═╝  ╚═╝╚═╝╚═╝     ╚═╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝
"#
    }

    pub fn compact_art() -> &'static str {
        "りむる Rimuru"
    }

    pub fn render<'a>(theme: &dyn Theme, compact: bool) -> Paragraph<'a> {
        if compact {
            Paragraph::new(Line::from(vec![
                Span::styled("りむる ", Style::default().fg(theme.accent())),
                Span::styled("Rimuru", Style::default().fg(theme.foreground())),
            ]))
        } else {
            let lines: Vec<Line> = Self::ascii_art()
                .lines()
                .map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(theme.accent()),
                    ))
                })
                .collect();
            Paragraph::new(lines)
        }
    }
}
