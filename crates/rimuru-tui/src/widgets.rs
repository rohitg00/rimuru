// Portions adapted from MIT-licensed terminal UI code.
// Copyright (c) 2026 Tae Hwan Jung — used under the MIT License.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::theme::Theme;

pub const BRAILLE_UP: [&str; 25] = [
    " ", "⢀", "⢠", "⢰", "⢸",
    "⡀", "⣀", "⣠", "⣰", "⣸",
    "⡄", "⣄", "⣤", "⣴", "⣼",
    "⡆", "⣆", "⣦", "⣶", "⣾",
    "⡇", "⣇", "⣧", "⣷", "⣿",
];

pub fn make_gradient(start: (u8, u8, u8), mid: (u8, u8, u8), end: (u8, u8, u8)) -> [Color; 101] {
    let mut out = [Color::Reset; 101];
    for i in 0..=100 {
        let (s, e, offset, range) = if i <= 50 {
            (start, mid, 0, 50)
        } else {
            (mid, end, 50, 50)
        };
        let t = i - offset;
        let r = s.0 as i32 + t as i32 * (e.0 as i32 - s.0 as i32) / range;
        let g = s.1 as i32 + t as i32 * (e.1 as i32 - s.1 as i32) / range;
        let b = s.2 as i32 + t as i32 * (e.2 as i32 - s.2 as i32) / range;
        out[i] = Color::Rgb(r.clamp(0, 255) as u8, g.clamp(0, 255) as u8, b.clamp(0, 255) as u8);
    }
    out
}

pub fn grad_at(gradient: &[Color; 101], pct: f64) -> Color {
    let idx = pct.clamp(0.0, 100.0).round() as usize;
    gradient[idx.min(100)]
}

pub fn meter_bar(pct: f64, width: usize, gradient: &[Color; 101], meter_bg: Color) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let clamped = pct.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * width as f64).round() as usize;
    let mut spans = Vec::new();
    for i in 0..width {
        if i < filled {
            let cell_pct = (i as f64 / width as f64) * 100.0;
            spans.push(Span::styled("■", Style::default().fg(grad_at(gradient, cell_pct))));
        } else {
            spans.push(Span::styled("■", Style::default().fg(meter_bg)));
        }
    }
    spans
}

pub fn remaining_bar(remaining_pct: f64, width: usize, gradient: &[Color; 101], meter_bg: Color) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let clamped = remaining_pct.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * width as f64).round() as usize;
    let used_pct = 100.0 - clamped;
    let mut spans = Vec::new();
    for i in 0..width {
        if i < filled {
            spans.push(Span::styled("■", Style::default().fg(grad_at(gradient, used_pct))));
        } else {
            spans.push(Span::styled("■", Style::default().fg(meter_bg)));
        }
    }
    spans
}

pub fn braille_sparkline(data: &[f64], width: usize, gradient: &[Color; 101], graph_text: Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    if data.is_empty() || width == 0 {
        for _ in 0..width {
            spans.push(Span::styled(" ", Style::default().fg(graph_text)));
        }
        return spans;
    }

    let needed = width * 2;
    let sampled: Vec<f64> = if data.len() >= needed {
        data[data.len() - needed..].to_vec()
    } else {
        let mut v = vec![0.0; needed - data.len()];
        v.extend_from_slice(data);
        v
    };

    for i in 0..width {
        let prev = (sampled[i * 2].clamp(0.0, 1.0) * 4.0).round() as usize;
        let cur = (sampled[i * 2 + 1].clamp(0.0, 1.0) * 4.0).round() as usize;
        let idx = prev * 5 + cur;
        let pct = sampled[i * 2 + 1] * 100.0;
        let color = grad_at(gradient, pct);
        spans.push(Span::styled(BRAILLE_UP[idx.min(24)].to_string(), Style::default().fg(color)));
    }
    spans
}

pub fn braille_graph_multirow(
    data: &[f64],
    width: usize,
    height: usize,
    gradient: &[Color; 101],
    graph_text: Color,
) -> Vec<Vec<Span<'static>>> {
    if height == 0 || width == 0 {
        return vec![vec![]; height];
    }

    let total_vres = height * 4;
    let needed = width * 2;

    let sampled: Vec<f64> = if data.len() >= needed {
        data[data.len() - needed..].to_vec()
    } else {
        let mut v = vec![0.0; needed - data.len()];
        v.extend_from_slice(data);
        v
    };

    let heights: Vec<usize> = sampled
        .iter()
        .map(|&v| (v.clamp(0.0, 1.0) * total_vres as f64).round() as usize)
        .collect();

    let left_bits: [u32; 4] = [0x40, 0x04, 0x02, 0x01];
    let right_bits: [u32; 4] = [0x80, 0x20, 0x10, 0x08];

    let mut rows: Vec<Vec<Span<'static>>> = Vec::with_capacity(height);

    for row in 0..height {
        let mut spans = Vec::with_capacity(width);
        let inv_row = height - 1 - row;
        let base_y = inv_row * 4;

        for col in 0..width {
            let left_h = heights[col * 2];
            let right_h = heights[col * 2 + 1];

            let mut pattern: u32 = 0;
            for dot_row in 0..4u32 {
                let y_pos = base_y + dot_row as usize;
                if left_h > y_pos {
                    pattern |= left_bits[dot_row as usize];
                }
                if right_h > y_pos {
                    pattern |= right_bits[dot_row as usize];
                }
            }

            let ch = char::from_u32(0x2800 + pattern).unwrap_or(' ');
            let max_val = sampled[col * 2].max(sampled[col * 2 + 1]);
            let color = if pattern == 0 {
                graph_text
            } else {
                grad_at(gradient, max_val * 100.0)
            };
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        rows.push(spans);
    }

    rows
}

pub fn btop_block(title: &str, number: &str, box_color: Color, theme: &Theme) -> Block<'static> {
    Block::default()
        .title(Line::from(vec![
            Span::styled("┐", Style::default().fg(box_color)),
            Span::styled(number.to_string(), Style::default().fg(theme.hi_fg).add_modifier(Modifier::BOLD)),
            Span::styled(title.to_string(), Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
            Span::styled("┌", Style::default().fg(box_color)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(box_color))
}

pub fn styled_label(text: &str, graph_text: Color) -> Span<'static> {
    Span::styled(text.to_string(), Style::default().fg(graph_text))
}

pub fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

pub fn fmt_dollars(v: f64) -> String {
    if v >= 1000.0 {
        format!("${:.0}", v)
    } else if v >= 1.0 {
        format!("${:.2}", v)
    } else {
        format!("${:.3}", v)
    }
}

pub fn truncate_str(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
