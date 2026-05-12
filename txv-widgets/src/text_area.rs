//! TextArea — read-only text viewer with line numbers and search.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct TextArea {
    state: ViewState,
    pub lines: Vec<String>,
    pub scroll: ScrollView,
    pub line_numbers: bool,
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub current_match: usize,
    /// Per-line foreground colors (optional, indexed by line number).
    pub line_colors: Vec<Color>,
    searching: bool,
    search_input: String,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            lines: Vec::new(),
            scroll: ScrollView::new(),
            line_numbers: true,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: 0,
            line_colors: Vec::new(),
            searching: false,
            search_input: String::new(),
        }
    }

    pub fn set_content(&mut self, text: &str) {
        self.lines = text.lines().map(String::from).collect();
        self.scroll.set_total(self.lines.len());
        self.state.mark_dirty();
    }

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        if !query.is_empty() {
            for (i, line) in self.lines.iter().enumerate() {
                if line.contains(query) {
                    self.search_matches.push(i);
                }
            }
        }
        self.current_match = 0;
        if let Some(&line) = self.search_matches.first() {
            self.scroll.ensure_visible(line);
        }
        self.state.mark_dirty();
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        let line = self.search_matches[self.current_match];
        self.scroll.ensure_visible(line);
        self.state.mark_dirty();
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        let line = self.search_matches[self.current_match];
        self.scroll.ensure_visible(line);
        self.state.mark_dirty();
    }

    fn gutter_width(&self) -> u16 {
        if !self.line_numbers {
            return 0;
        }
        let digits = if self.lines.is_empty() {
            1
        } else {
            (self.lines.len() as f64).log10() as u16 + 1
        };
        digits + 1 // +1 for separator space
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TextArea {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let gutter_w = self.gutter_width();
        let gutter_style = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let normal = Style::default();
        let highlight = Style {
            bg: Color::Ansi(3),
            ..Style::default()
        };

        let content_h = if self.searching {
            b.h.saturating_sub(1) as usize
        } else {
            b.h as usize
        };

        for row in 0..content_h {
            let line_idx = self.scroll.offset + row;
            let y = b.y + row as u16;
            surface.hline(b.x, y, b.w, ' ', normal);

            if line_idx >= self.lines.len() {
                continue;
            }

            // Line number
            if self.line_numbers {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                surface.print(b.x, y, &num, gutter_style);
            }

            // Line content
            let is_match = self.search_matches.contains(&line_idx);
            let style = if is_match {
                highlight
            } else if let Some(&color) = self.line_colors.get(line_idx) {
                Style {
                    fg: color,
                    ..Style::default()
                }
            } else {
                normal
            };
            let text_x = b.x + gutter_w;
            let avail = b.w.saturating_sub(gutter_w) as usize;
            let line = &self.lines[line_idx];
            let visible: String = line.chars().take(avail).collect();
            surface.print(text_x, y, &visible, style);
        }

        // Search prompt at bottom
        if self.searching {
            let y = b.y + b.h.saturating_sub(1);
            let prompt_style = Style {
                attrs: Attrs {
                    reverse: true,
                    ..Attrs::default()
                },
                ..Style::default()
            };
            surface.hline(b.x, y, b.w, ' ', prompt_style);
            let prompt = format!("/{}", self.search_input);
            surface.print(b.x, y, &prompt, prompt_style);
        }
    }

    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        // Search input mode
        if self.searching {
            match key.code {
                KeyCode::Enter => {
                    self.searching = false;
                    self.search(&self.search_input.clone());
                }
                KeyCode::Esc => {
                    self.searching = false;
                    self.search_input.clear();
                    self.state.mark_dirty();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.state.mark_dirty();
                }
                KeyCode::Char(ch) => {
                    self.search_input.push(ch);
                    self.state.mark_dirty();
                }
                _ => {}
            }
            return HandleResult::Consumed;
        }
        match key.code {
            KeyCode::Up => {
                self.scroll.scroll_up(1);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Down => {
                self.scroll.scroll_down(1);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                self.scroll.scroll_up(page);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                self.scroll.scroll_down(page);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.scroll.scroll_to(0);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::End => {
                let max = self.scroll.max_offset();
                self.scroll.scroll_to(max);
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Char('/') if !key.modifiers.ctrl => {
                self.searching = true;
                self.search_input.clear();
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Char('n') if !key.modifiers.ctrl => {
                self.next_match();
                HandleResult::Consumed
            }
            KeyCode::Char('N') if !key.modifiers.ctrl => {
                self.prev_match();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
