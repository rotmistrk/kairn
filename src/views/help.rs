//! HelpView — topic-based help with cross-reference navigation.

use txv_core::cell::Style;
use txv_core::key_help::KeyHelpEntry;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::help_topics::generate_topic;

pub struct HelpView {
    state: ViewState,
    lines: Vec<String>,
    cursor: usize,
    scroll: usize,
    topic: String,
    bindings: Vec<KeyHelpEntry>,
}

impl HelpView {
    pub fn new(topic: &str, bindings: &[KeyHelpEntry]) -> Self {
        let content = generate_topic(topic, bindings);
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        Self {
            state: ViewState::default(),
            lines,
            cursor: 0,
            scroll: 0,
            topic: topic.to_string(),
            bindings: bindings.to_vec(),
        }
    }

    fn navigate_to(&mut self, topic: &str) {
        let content = generate_topic(topic, &self.bindings);
        self.lines = content.lines().map(|l| l.to_string()).collect();
        self.cursor = 0;
        self.scroll = 0;
        self.topic = topic.to_string();
        self.state.mark_dirty();
    }

    fn visible_rows(&self) -> usize {
        self.state.bounds().h() as usize
    }

    fn sync_scroll(&mut self) {
        let h = self.visible_rows();
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    fn handle_enter(&mut self) -> HandleResult {
        let topic = self
            .lines
            .get(self.cursor)
            .and_then(|line| extract_crossref(line))
            .map(|t| t.to_string());
        if let Some(t) = topic {
            self.navigate_to(&t);
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    fn move_to(&mut self, line: usize) {
        self.cursor = line.min(self.lines.len().saturating_sub(1));
        self.sync_scroll();
        self.state.mark_dirty();
    }

    fn handle_key(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.lines.len() {
                    self.move_to(self.cursor + 1);
                }
                HandleResult::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.move_to(self.cursor - 1);
                }
                HandleResult::Consumed
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.move_to(0);
                HandleResult::Consumed
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.move_to(self.lines.len().saturating_sub(1));
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                let h = self.visible_rows().max(1);
                self.move_to(self.cursor + h);
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                let h = self.visible_rows().max(1);
                self.move_to(self.cursor.saturating_sub(h));
                HandleResult::Consumed
            }
            KeyCode::Enter => self.handle_enter(),
            _ => HandleResult::Ignored,
        }
    }
}

impl View for HelpView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        "Help"
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let normal = Style::default();
        let dim = pal.style(StyleId::Dim);
        let cursor_style = if self.state.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        };
        for row in 0..h as usize {
            let idx = self.scroll + row;
            let y = row as u16;
            if idx >= self.lines.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let line = &self.lines[idx];
            let style = if idx == self.cursor {
                cursor_style
            } else if is_crossref(line) {
                dim
            } else {
                normal
            };
            self.state.buffer_mut().print_line(0, y, line, w, style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            return self.handle_key(key);
        }
        HandleResult::Ignored
    }
}

fn is_crossref(line: &str) -> bool {
    line.trim_start().starts_with("→ :help ")
}

fn extract_crossref(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("→ :help ")?;
    Some(rest.split_whitespace().next().unwrap_or(rest.trim()))
}
