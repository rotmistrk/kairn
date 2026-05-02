//! Multi-line read-only text viewer with scroll, search, and line numbers.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::cell::Style;
use txv::surface::Surface;
use txv::text::display_width;

use crate::scroll_view::ScrollView;
use crate::scrollbar::Scrollbar;
use crate::widget::{EventResult, Widget, WidgetAction};

/// Multi-line read-only text viewer.
pub struct TextArea {
    lines: Vec<String>,
    scroll: ScrollView,
    show_line_numbers: bool,
    search_query: Option<String>,
    search_matches: Vec<(usize, usize)>, // (line, col)
    search_index: usize,
    /// Style for normal text.
    pub text_style: Style,
    /// Style for line numbers.
    pub line_number_style: Style,
    /// Style for search matches.
    pub match_style: Style,
    /// Style for the current search match.
    pub current_match_style: Style,
    /// Whether to show the scrollbar.
    pub show_scrollbar: bool,
}

impl TextArea {
    /// Create a new empty text area.
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll: ScrollView::new(),
            show_line_numbers: true,
            search_query: None,
            search_matches: Vec::new(),
            search_index: 0,
            text_style: Style::default(),
            line_number_style: Style {
                attrs: txv::cell::Attrs {
                    dim: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            match_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            current_match_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    bold: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            show_scrollbar: true,
        }
    }

    /// Set the text content (splits on newlines).
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(String::from).collect();
        self.scroll.set_content_size(self.lines.len(), 0);
        self.scroll.scroll_to_top();
        self.clear_search();
    }

    /// Set content from a vec of lines.
    pub fn set_lines(&mut self, lines: Vec<String>) {
        self.scroll.set_content_size(lines.len(), 0);
        self.lines = lines;
        self.scroll.scroll_to_top();
        self.clear_search();
    }

    /// Get the lines.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Total line count.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Toggle line numbers.
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Current scroll row.
    pub fn scroll_row(&self) -> usize {
        self.scroll.scroll_row
    }

    /// Search for a string. Populates matches and jumps to first.
    pub fn search(&mut self, query: &str) {
        self.search_matches.clear();
        self.search_index = 0;
        if query.is_empty() {
            self.search_query = None;
            return;
        }
        let lower_query = query.to_lowercase();
        self.search_query = Some(query.to_string());
        for (line_idx, line) in self.lines.iter().enumerate() {
            let lower_line = line.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower_line[start..].find(&lower_query) {
                self.search_matches.push((line_idx, start + pos));
                start += pos + 1;
            }
        }
        if let Some(&(line, _)) = self.search_matches.first() {
            self.scroll.ensure_visible(line, 0);
        }
    }

    /// Jump to the next search match.
    pub fn search_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_index = (self.search_index + 1) % self.search_matches.len();
        let (line, _) = self.search_matches[self.search_index];
        self.scroll.ensure_visible(line, 0);
    }

    /// Jump to the previous search match.
    pub fn search_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.search_index = if self.search_index == 0 {
            self.search_matches.len() - 1
        } else {
            self.search_index - 1
        };
        let (line, _) = self.search_matches[self.search_index];
        self.scroll.ensure_visible(line, 0);
    }

    /// Number of search matches.
    pub fn match_count(&self) -> usize {
        self.search_matches.len()
    }

    /// Clear the search.
    pub fn clear_search(&mut self) {
        self.search_query = None;
        self.search_matches.clear();
        self.search_index = 0;
    }

    fn gutter_width(&self) -> u16 {
        if !self.show_line_numbers {
            return 0;
        }
        let digits = digit_count(self.lines.len());
        digits as u16 + 1 // digits + separator space
    }

    fn render_line(&self, surface: &mut Surface<'_>, line_idx: usize, gutter_w: u16) {
        let w = surface.width();
        if self.show_line_numbers && gutter_w > 0 {
            let num = format!("{:>width$}", line_idx + 1, width = (gutter_w - 1) as usize);
            surface.print(0, 0, &num, self.line_number_style);
        }

        let text_start = gutter_w;
        let text_w = w.saturating_sub(text_start);
        if text_w == 0 || line_idx >= self.lines.len() {
            return;
        }

        let line = &self.lines[line_idx];
        // Check for search matches on this line
        let matches_on_line: Vec<(usize, usize)> = self.line_matches(line_idx);

        if matches_on_line.is_empty() {
            surface.print(text_start, 0, line, self.text_style);
            return;
        }

        // Render with highlighted matches
        let query_len = self.search_query.as_ref().map(|q| q.len()).unwrap_or(0);
        let mut col = text_start;
        let mut byte = 0;
        for ch in line.chars() {
            if col >= w {
                break;
            }
            let style = self.char_style(line_idx, byte, query_len, &matches_on_line);
            surface.put(col, 0, ch, style);
            let cw = display_width(&ch.to_string());
            col += cw as u16;
            byte += ch.len_utf8();
        }
    }

    fn line_matches(&self, line_idx: usize) -> Vec<(usize, usize)> {
        self.search_matches
            .iter()
            .enumerate()
            .filter(|(_, (l, _))| *l == line_idx)
            .map(|(match_idx, (_, col))| (match_idx, *col))
            .collect()
    }

    fn char_style(
        &self,
        line_idx: usize,
        byte_pos: usize,
        query_len: usize,
        matches_on_line: &[(usize, usize)],
    ) -> Style {
        for &(match_idx, match_col) in matches_on_line {
            if byte_pos >= match_col && byte_pos < match_col + query_len {
                if match_idx == self.search_index {
                    return self.current_match_style;
                }
                // Check if this is the current match
                let global_idx = self
                    .search_matches
                    .iter()
                    .position(|&(l, c)| l == line_idx && c == match_col);
                if global_idx == Some(self.search_index) {
                    return self.current_match_style;
                }
                return self.match_style;
            }
        }
        self.text_style
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextArea {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let h = surface.height();
        let w = surface.width();
        let gutter_w = self.gutter_width();
        let scrollbar_w: u16 = if self.show_scrollbar { 1 } else { 0 };
        let content_w = w.saturating_sub(scrollbar_w);

        let range = self.scroll.visible_range(h);
        for (row_idx, line_idx) in range.enumerate() {
            let mut row = surface.sub(0, row_idx as u16, content_w, 1);
            self.render_line(&mut row, line_idx, gutter_w);
        }

        // Scrollbar
        if self.show_scrollbar && w > 0 {
            let sb = Scrollbar::new();
            sb.render(
                surface,
                w - 1,
                h,
                self.scroll.scroll_row,
                h as usize,
                self.lines.len(),
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Up => {
                self.scroll.scroll_up(1);
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.scroll.scroll_down(1, 0);
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                self.scroll.page_up(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                self.scroll.page_down(20);
                EventResult::Consumed
            }
            KeyCode::Home if ctrl => {
                self.scroll.scroll_to_top();
                EventResult::Consumed
            }
            KeyCode::End if ctrl => {
                self.scroll.scroll_to_bottom(20);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.scroll.scroll_to_top();
                EventResult::Consumed
            }
            KeyCode::End => {
                self.scroll.scroll_to_bottom(20);
                EventResult::Consumed
            }
            KeyCode::Char('n') if !ctrl => {
                self.search_next();
                EventResult::Consumed
            }
            KeyCode::Char('N') => {
                self.search_prev();
                EventResult::Consumed
            }
            KeyCode::Esc => {
                if self.search_query.is_some() {
                    self.clear_search();
                    EventResult::Consumed
                } else {
                    EventResult::Action(WidgetAction::Cancelled)
                }
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Count decimal digits needed to display a number.
fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut v = n;
    while v > 0 {
        count += 1;
        v /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render_area(area: &TextArea, w: u16, h: u16) -> String {
        let mut screen = Screen::with_color_mode(w, h, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            area.render(&mut s, true);
        }
        screen.to_text()
    }

    fn sample_text() -> &'static str {
        "line one\nline two\nline three\nline four\nline five"
    }

    #[test]
    fn new_is_empty() {
        let ta = TextArea::new();
        assert_eq!(ta.line_count(), 0);
    }

    #[test]
    fn set_text_splits_lines() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        assert_eq!(ta.line_count(), 5);
        assert_eq!(ta.lines()[0], "line one");
    }

    #[test]
    fn set_lines_directly() {
        let mut ta = TextArea::new();
        ta.set_lines(vec!["a".into(), "b".into()]);
        assert_eq!(ta.line_count(), 2);
    }

    #[test]
    fn render_shows_line_numbers() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.show_scrollbar = false;
        let text = render_area(&ta, 30, 5);
        assert!(text.contains("1"));
        assert!(text.contains("line one"));
    }

    #[test]
    fn render_without_line_numbers() {
        let mut ta = TextArea::new();
        ta.set_text("hello");
        ta.set_show_line_numbers(false);
        ta.show_scrollbar = false;
        let text = render_area(&ta, 20, 1);
        assert!(text.contains("hello"));
        // Should not have a number prefix
        assert!(!text.starts_with("1"));
    }

    #[test]
    fn scroll_up_down() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        assert_eq!(ta.scroll_row(), 0);
        ta.handle_key(key(KeyCode::Down));
        assert_eq!(ta.scroll_row(), 1);
        ta.handle_key(key(KeyCode::Up));
        assert_eq!(ta.scroll_row(), 0);
    }

    #[test]
    fn scroll_clamps() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.handle_key(key(KeyCode::Up));
        assert_eq!(ta.scroll_row(), 0); // can't go negative
    }

    #[test]
    fn home_end_scroll() {
        let mut ta = TextArea::new();
        let lines: Vec<String> = (0..50).map(|i| format!("line {i}")).collect();
        ta.set_lines(lines);
        ta.handle_key(key(KeyCode::End));
        assert!(ta.scroll_row() > 0);
        ta.handle_key(key(KeyCode::Home));
        assert_eq!(ta.scroll_row(), 0);
    }

    #[test]
    fn search_finds_matches() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        assert_eq!(ta.match_count(), 5);
    }

    #[test]
    fn search_case_insensitive() {
        let mut ta = TextArea::new();
        ta.set_text("Hello\nhello\nHELLO");
        ta.search("hello");
        assert_eq!(ta.match_count(), 3);
    }

    #[test]
    fn search_next_prev() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        assert_eq!(ta.search_index, 0);
        ta.search_next();
        assert_eq!(ta.search_index, 1);
        ta.search_prev();
        assert_eq!(ta.search_index, 0);
        ta.search_prev(); // wraps
        assert_eq!(ta.search_index, 4);
    }

    #[test]
    fn n_key_next_match() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        ta.handle_key(key(KeyCode::Char('n')));
        assert_eq!(ta.search_index, 1);
    }

    #[test]
    fn shift_n_prev_match() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        ta.handle_key(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT));
        assert_eq!(ta.search_index, 4); // wrapped to last
    }

    #[test]
    fn esc_clears_search_first() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        let result = ta.handle_key(key(KeyCode::Esc));
        assert!(matches!(result, EventResult::Consumed));
        assert_eq!(ta.match_count(), 0);
        // Second Esc produces Cancelled
        let result = ta.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn clear_search() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        assert!(ta.match_count() > 0);
        ta.clear_search();
        assert_eq!(ta.match_count(), 0);
    }

    #[test]
    fn empty_search_clears() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.search("line");
        ta.search("");
        assert_eq!(ta.match_count(), 0);
    }

    #[test]
    fn digit_count_works() {
        assert_eq!(super::digit_count(0), 1);
        assert_eq!(super::digit_count(1), 1);
        assert_eq!(super::digit_count(9), 1);
        assert_eq!(super::digit_count(10), 2);
        assert_eq!(super::digit_count(999), 3);
        assert_eq!(super::digit_count(1000), 4);
    }

    #[test]
    fn render_with_scrollbar() {
        let mut ta = TextArea::new();
        ta.set_text(sample_text());
        ta.show_scrollbar = true;
        let mut screen = Screen::with_color_mode(30, 3, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            ta.render(&mut s, true);
        }
        // Last column should have scrollbar chars
        let ch = screen.cell(29, 0).ch;
        assert!(ch == '█' || ch == '│');
    }

    #[test]
    fn gutter_width_scales() {
        let mut ta = TextArea::new();
        ta.set_text("a");
        assert_eq!(ta.gutter_width(), 2); // "1 "
        let lines: Vec<String> = (0..100).map(|i| format!("line {i}")).collect();
        ta.set_lines(lines);
        assert_eq!(ta.gutter_width(), 4); // "100 "
    }

    #[test]
    fn multiple_matches_per_line() {
        let mut ta = TextArea::new();
        ta.set_text("aaa bbb aaa");
        ta.search("aaa");
        assert_eq!(ta.match_count(), 2);
    }
}
