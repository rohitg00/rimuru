use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme::Theme;

pub fn highlight_text<'a>(
    text: &'a str,
    query: Option<&str>,
    base_style: Style,
    theme: &dyn Theme,
) -> Vec<Span<'a>> {
    match query {
        Some(q) if !q.is_empty() => {
            let highlight_style = Style::default()
                .bg(theme.warning())
                .fg(theme.background())
                .add_modifier(Modifier::BOLD);

            create_highlighted_spans(text, q, base_style, highlight_style)
        }
        _ => vec![Span::styled(text.to_string(), base_style)],
    }
}

fn create_highlighted_spans<'a>(
    text: &'a str,
    query: &str,
    base_style: Style,
    highlight_style: Style,
) -> Vec<Span<'a>> {
    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&query_lower) {
        if start > last_end {
            spans.push(Span::styled(text[last_end..start].to_string(), base_style));
        }
        spans.push(Span::styled(
            text[start..start + query.len()].to_string(),
            highlight_style,
        ));
        last_end = start + query.len();
    }

    if last_end < text.len() {
        spans.push(Span::styled(text[last_end..].to_string(), base_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), base_style));
    }

    spans
}

pub fn highlight_line<'a>(
    text: &'a str,
    query: Option<&str>,
    base_style: Style,
    theme: &dyn Theme,
) -> Line<'a> {
    Line::from(highlight_text(text, query, base_style, theme))
}

pub fn matches_query(text: &str, query: Option<&str>) -> bool {
    match query {
        Some(q) if !q.is_empty() => text.to_lowercase().contains(&q.to_lowercase()),
        _ => true,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SmoothScroll {
    current: f32,
    target: f32,
    velocity: f32,
    easing: EasingFunction,
    animation_duration_ms: u64,
    animation_start: Option<std::time::Instant>,
    start_position: f32,
}

impl SmoothScroll {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            velocity: 0.0,
            easing: EasingFunction::EaseOut,
            animation_duration_ms: 150,
            animation_start: None,
            start_position: 0.0,
        }
    }

    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_duration_ms(mut self, duration: u64) -> Self {
        self.animation_duration_ms = duration;
        self
    }

    pub fn set_target(&mut self, target: usize) {
        let target_f = target as f32;
        if (self.target - target_f).abs() > 0.01 {
            self.start_position = self.current;
            self.target = target_f;
            self.animation_start = Some(std::time::Instant::now());
        }
    }

    pub fn set_immediate(&mut self, position: usize) {
        self.current = position as f32;
        self.target = position as f32;
        self.animation_start = None;
    }

    pub fn update(&mut self) {
        if let Some(start) = self.animation_start {
            let elapsed = start.elapsed().as_millis() as f32;
            let duration = self.animation_duration_ms as f32;
            let progress = (elapsed / duration).min(1.0);

            let eased_progress = self.easing.apply(progress);
            self.current =
                self.start_position + (self.target - self.start_position) * eased_progress;

            if progress >= 1.0 {
                self.current = self.target;
                self.animation_start = None;
            }
        }
    }

    pub fn position(&self) -> usize {
        self.current.round() as usize
    }

    pub fn exact_position(&self) -> f32 {
        self.current
    }

    pub fn is_animating(&self) -> bool {
        self.animation_start.is_some()
    }

    pub fn target(&self) -> usize {
        self.target as usize
    }
}

impl Default for SmoothScroll {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScrollState {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_height: usize,
    smooth_scroll: SmoothScroll,
    use_smooth_scroll: bool,
}

impl ScrollState {
    pub fn new(visible_height: usize) -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            visible_height,
            smooth_scroll: SmoothScroll::new(),
            use_smooth_scroll: true,
        }
    }

    pub fn with_smooth_scroll(mut self, enabled: bool) -> Self {
        self.use_smooth_scroll = enabled;
        self
    }

    pub fn select(&mut self, index: usize, max_items: usize) {
        let max_index = max_items.saturating_sub(1);
        self.selected_index = index.min(max_index);
        self.adjust_scroll(max_items);
    }

    pub fn select_next(&mut self, max_items: usize) {
        if max_items == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1).min(max_items.saturating_sub(1));
        self.adjust_scroll(max_items);
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
        self.adjust_scroll_for_index();
    }

    pub fn page_down(&mut self, max_items: usize) {
        if max_items == 0 {
            return;
        }
        let page_size = self.visible_height.saturating_sub(1).max(1);
        self.selected_index = (self.selected_index + page_size).min(max_items.saturating_sub(1));
        self.adjust_scroll(max_items);
    }

    pub fn page_up(&mut self) {
        let page_size = self.visible_height.saturating_sub(1).max(1);
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.adjust_scroll_for_index();
    }

    pub fn go_to_top(&mut self) {
        self.selected_index = 0;
        if self.use_smooth_scroll {
            self.smooth_scroll.set_target(0);
        }
        self.scroll_offset = 0;
    }

    pub fn go_to_bottom(&mut self, max_items: usize) {
        if max_items == 0 {
            return;
        }
        self.selected_index = max_items.saturating_sub(1);
        self.adjust_scroll(max_items);
    }

    fn adjust_scroll(&mut self, max_items: usize) {
        let padding = 2;

        if self.selected_index < self.scroll_offset + padding {
            self.scroll_offset = self.selected_index.saturating_sub(padding);
        }

        let visible_end = self.scroll_offset + self.visible_height;
        if self.selected_index >= visible_end.saturating_sub(padding) {
            self.scroll_offset =
                (self.selected_index + padding + 1).saturating_sub(self.visible_height);
        }

        let max_scroll = max_items.saturating_sub(self.visible_height);
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        if self.use_smooth_scroll {
            self.smooth_scroll.set_target(self.scroll_offset);
        }
    }

    fn adjust_scroll_for_index(&mut self) {
        let padding = 2;

        if self.selected_index < self.scroll_offset + padding {
            self.scroll_offset = self.selected_index.saturating_sub(padding);
        }

        if self.use_smooth_scroll {
            self.smooth_scroll.set_target(self.scroll_offset);
        }
    }

    pub fn update(&mut self) {
        if self.use_smooth_scroll {
            self.smooth_scroll.update();
        }
    }

    pub fn effective_scroll_offset(&self) -> usize {
        if self.use_smooth_scroll && self.smooth_scroll.is_animating() {
            self.smooth_scroll.position()
        } else {
            self.scroll_offset
        }
    }

    pub fn is_animating(&self) -> bool {
        self.use_smooth_scroll && self.smooth_scroll.is_animating()
    }

    pub fn set_visible_height(&mut self, height: usize) {
        self.visible_height = height;
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_query() {
        assert!(matches_query("Hello World", Some("world")));
        assert!(matches_query("Hello World", Some("HELLO")));
        assert!(!matches_query("Hello World", Some("xyz")));
        assert!(matches_query("Hello World", None));
        assert!(matches_query("Hello World", Some("")));
    }

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);

        assert_eq!(EasingFunction::Linear.apply(0.0), 0.0);
        assert_eq!(EasingFunction::Linear.apply(1.0), 1.0);
        assert_eq!(EasingFunction::EaseInOut.apply(0.0), 0.0);
        assert_eq!(EasingFunction::EaseInOut.apply(1.0), 1.0);
    }

    #[test]
    fn test_smooth_scroll() {
        let mut scroll = SmoothScroll::new();
        assert_eq!(scroll.position(), 0);

        scroll.set_target(10);
        assert_eq!(scroll.target(), 10);
        assert!(scroll.is_animating());

        scroll.set_immediate(5);
        assert_eq!(scroll.position(), 5);
        assert!(!scroll.is_animating());
    }

    #[test]
    fn test_scroll_state_navigation() {
        let mut state = ScrollState::new(5);

        state.select_next(10);
        assert_eq!(state.selected_index, 1);

        state.select_prev();
        assert_eq!(state.selected_index, 0);

        state.go_to_bottom(10);
        assert_eq!(state.selected_index, 9);

        state.go_to_top();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_scroll_state_paging() {
        let mut state = ScrollState::new(5);

        state.page_down(20);
        assert_eq!(state.selected_index, 4);

        state.page_up();
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_scroll_state_bounds() {
        let mut state = ScrollState::new(5);

        state.select(100, 10);
        assert_eq!(state.selected_index, 9);

        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        state.select_prev();
        assert_eq!(state.selected_index, 0);
    }
}
