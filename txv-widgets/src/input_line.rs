//! Single-line text input with cursor and history.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::cell::Style;
use txv::surface::Surface;
use txv::text::display_width;

use crate::widget::{EventResult, Widget, WidgetAction};

/// Completion function signature: `(text, cursor_pos) -> candidates`.
pub type CompletionFn = Box<dyn Fn(&str, usize) -> Vec<String>>;

/// Single-line text input with cursor, history, and emacs-style shortcuts.
pub struct InputLine {
    text: String,
    cursor: usize, // character index
    prompt: String,
    history: Vec<String>,
    history_pos: Option<usize>,
    /// Optional completion function called on Tab.
    pub completion_fn: Option<CompletionFn>,
    /// Style for the prompt text.
    pub prompt_style: Style,
    /// Style for the input text.
    pub text_style: Style,
    /// Style for the cursor character.
    pub cursor_style: Style,
}

impl InputLine {
    /// Create a new input line with the given prompt.
    pub fn new(prompt: &str) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            prompt: prompt.to_string(),
            history: Vec::new(),
            history_pos: None,
            completion_fn: None,
            prompt_style: Style::default(),
            text_style: Style::default(),
            cursor_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
        }
    }

    /// Get the current text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text and move cursor to end.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.chars().count();
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.history_pos = None;
    }

    /// Add an entry to history.
    pub fn push_history(&mut self, entry: String) {
        if !entry.is_empty() {
            self.history.push(entry);
        }
    }

    /// Get a reference to the history entries.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Attempt completion using the configured completion function.
    /// Returns the number of candidates found.
    pub fn try_complete(&mut self) -> usize {
        let candidates = match self.completion_fn {
            Some(ref f) => f(&self.text, self.cursor),
            None => return 0,
        };
        match candidates.len() {
            0 => 0,
            1 => {
                self.set_text(&candidates[0]);
                1
            }
            _ => {
                let prefix = longest_common_prefix(&candidates);
                if prefix.len() > self.text.len() {
                    self.set_text(&prefix);
                }
                candidates.len()
            }
        }
    }

    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn byte_offset(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    fn insert_char(&mut self, ch: char) {
        let byte_pos = self.byte_offset();
        self.text.insert(byte_pos, ch);
        self.cursor += 1;
    }

    fn delete_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        let byte_pos = self.byte_offset();
        let next = self
            .text
            .char_indices()
            .nth(self.cursor + 1)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        self.text.replace_range(byte_pos..next, "");
    }

    fn delete_forward(&mut self) {
        if self.cursor >= self.char_count() {
            return;
        }
        let byte_pos = self.byte_offset();
        let next = self
            .text
            .char_indices()
            .nth(self.cursor + 1)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        self.text.replace_range(byte_pos..next, "");
    }

    fn kill_to_end(&mut self) {
        let byte_pos = self.byte_offset();
        self.text.truncate(byte_pos);
    }

    fn kill_to_start(&mut self) {
        let byte_pos = self.byte_offset();
        self.text = self.text[byte_pos..].to_string();
        self.cursor = 0;
    }

    fn kill_word_backward(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.cursor;
        // Skip trailing spaces
        while pos > 0 && chars[pos - 1] == ' ' {
            pos -= 1;
        }
        // Skip word chars
        while pos > 0 && chars[pos - 1] != ' ' {
            pos -= 1;
        }
        let start_byte = self
            .text
            .char_indices()
            .nth(pos)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len());
        let end_byte = self.byte_offset();
        self.text.replace_range(start_byte..end_byte, "");
        self.cursor = pos;
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_pos = match self.history_pos {
            None => self.history.len() - 1,
            Some(0) => return,
            Some(p) => p - 1,
        };
        self.history_pos = Some(new_pos);
        self.text = self.history[new_pos].clone();
        self.cursor = self.char_count();
    }

    fn history_next(&mut self) {
        let pos = match self.history_pos {
            None => return,
            Some(p) => p,
        };
        if pos + 1 >= self.history.len() {
            self.history_pos = None;
            self.text.clear();
            self.cursor = 0;
        } else {
            self.history_pos = Some(pos + 1);
            self.text = self.history[pos + 1].clone();
            self.cursor = self.char_count();
        }
    }
}

impl Widget for InputLine {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let w = surface.width() as usize;
        surface.hline(0, 0, surface.width(), ' ', self.text_style);

        let prompt_w = display_width(&self.prompt);
        surface.print(0, 0, &self.prompt, self.prompt_style);

        let avail = w.saturating_sub(prompt_w);
        if avail == 0 {
            return;
        }
        let col_start = prompt_w as u16;

        // Compute visible window of text
        let cursor_col = display_width(&self.text[..self.byte_offset()]);
        let scroll = if cursor_col >= avail {
            cursor_col - avail + 1
        } else {
            0
        };

        // Render visible portion of text
        let mut col = 0usize;
        for (i, ch) in self.text.chars().enumerate() {
            let cw = display_width(&ch.to_string());
            if col + cw > scroll + avail {
                break;
            }
            if col >= scroll {
                let x = col_start + (col - scroll) as u16;
                let style = if i == self.cursor {
                    self.cursor_style
                } else {
                    self.text_style
                };
                surface.put(x, 0, ch, style);
            }
            col += cw;
        }

        // Draw cursor at end if cursor is past text
        if self.cursor >= self.char_count() && cursor_col >= scroll {
            let x = col_start + (cursor_col - scroll) as u16;
            if (x as usize) < w {
                surface.put(x, 0, ' ', self.cursor_style);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('a') if ctrl => {
                self.cursor = 0;
                EventResult::Consumed
            }
            KeyCode::Char('e') if ctrl => {
                self.cursor = self.char_count();
                EventResult::Consumed
            }
            KeyCode::Char('k') if ctrl => {
                self.kill_to_end();
                EventResult::Consumed
            }
            KeyCode::Char('u') if ctrl => {
                self.kill_to_start();
                EventResult::Consumed
            }
            KeyCode::Char('w') if ctrl => {
                self.kill_word_backward();
                EventResult::Consumed
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                EventResult::Consumed
            }
            KeyCode::Right => {
                self.cursor = self
                    .cursor
                    .min(self.char_count())
                    .min(self.char_count().saturating_sub(0));
                if self.cursor < self.char_count() {
                    self.cursor += 1;
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.cursor = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                self.cursor = self.char_count();
                EventResult::Consumed
            }
            KeyCode::Backspace => {
                self.delete_backward();
                EventResult::Consumed
            }
            KeyCode::Delete => {
                self.delete_forward();
                EventResult::Consumed
            }
            KeyCode::Up => {
                self.history_prev();
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.history_next();
                EventResult::Consumed
            }
            KeyCode::Enter => EventResult::Action(WidgetAction::Confirmed(self.text.clone())),
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            KeyCode::Tab => {
                self.try_complete();
                EventResult::Consumed
            }
            KeyCode::Char(ch) if !ctrl => {
                self.insert_char(ch);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Compute the longest common prefix of a set of strings.
fn longest_common_prefix(strings: &[String]) -> String {
    let Some(first) = strings.first() else {
        return String::new();
    };
    let mut prefix = first.clone();
    for s in &strings[1..] {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return prefix;
            }
        }
    }
    prefix
}

/// Default completion function: matches history entries by prefix.
/// Use with `input.completion_fn = Some(complete_from_history(history))`.
pub fn complete_from_history(history: Vec<String>) -> CompletionFn {
    Box::new(move |text: &str, _cursor: usize| {
        if text.is_empty() {
            return Vec::new();
        }
        let lower = text.to_lowercase();
        history
            .iter()
            .filter(|h| h.to_lowercase().starts_with(&lower))
            .cloned()
            .collect()
    })
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

    fn ctrl(ch: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
    }

    fn render_text(input: &InputLine, width: u16) -> String {
        let mut screen = Screen::with_color_mode(width, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            input.render(&mut s, true);
        }
        screen.to_text().trim_end_matches('\n').to_string()
    }

    #[test]
    fn new_is_empty() {
        let input = InputLine::new("> ");
        assert_eq!(input.text(), "");
    }

    #[test]
    fn insert_chars() {
        let mut input = InputLine::new("");
        input.handle_key(key(KeyCode::Char('a')));
        input.handle_key(key(KeyCode::Char('b')));
        assert_eq!(input.text(), "ab");
    }

    #[test]
    fn backspace_deletes() {
        let mut input = InputLine::new("");
        input.set_text("abc");
        input.handle_key(key(KeyCode::Backspace));
        assert_eq!(input.text(), "ab");
    }

    #[test]
    fn delete_forward() {
        let mut input = InputLine::new("");
        input.set_text("abc");
        input.cursor = 0;
        input.handle_key(key(KeyCode::Delete));
        assert_eq!(input.text(), "bc");
    }

    #[test]
    fn cursor_movement() {
        let mut input = InputLine::new("");
        input.set_text("abc");
        assert_eq!(input.cursor, 3);
        input.handle_key(key(KeyCode::Left));
        assert_eq!(input.cursor, 2);
        input.handle_key(key(KeyCode::Home));
        assert_eq!(input.cursor, 0);
        input.handle_key(key(KeyCode::End));
        assert_eq!(input.cursor, 3);
        input.handle_key(key(KeyCode::Right));
        assert_eq!(input.cursor, 3); // clamped
    }

    #[test]
    fn ctrl_a_e() {
        let mut input = InputLine::new("");
        input.set_text("hello");
        input.handle_key(ctrl('a'));
        assert_eq!(input.cursor, 0);
        input.handle_key(ctrl('e'));
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn ctrl_k_kill_to_end() {
        let mut input = InputLine::new("");
        input.set_text("hello");
        input.cursor = 2;
        input.handle_key(ctrl('k'));
        assert_eq!(input.text(), "he");
    }

    #[test]
    fn ctrl_u_kill_to_start() {
        let mut input = InputLine::new("");
        input.set_text("hello");
        input.cursor = 2;
        input.handle_key(ctrl('u'));
        assert_eq!(input.text(), "llo");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn ctrl_w_kill_word() {
        let mut input = InputLine::new("");
        input.set_text("hello world");
        input.handle_key(ctrl('w'));
        assert_eq!(input.text(), "hello ");
    }

    #[test]
    fn history_navigation() {
        let mut input = InputLine::new("");
        input.push_history("first".into());
        input.push_history("second".into());
        input.handle_key(key(KeyCode::Up));
        assert_eq!(input.text(), "second");
        input.handle_key(key(KeyCode::Up));
        assert_eq!(input.text(), "first");
        input.handle_key(key(KeyCode::Up)); // at start, no change
        assert_eq!(input.text(), "first");
        input.handle_key(key(KeyCode::Down));
        assert_eq!(input.text(), "second");
        input.handle_key(key(KeyCode::Down)); // past end, clears
        assert_eq!(input.text(), "");
    }

    #[test]
    fn enter_confirms() {
        let mut input = InputLine::new("");
        input.set_text("done");
        let result = input.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Confirmed(s)) if s == "done"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut input = InputLine::new("");
        let result = input.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn render_with_prompt() {
        let mut input = InputLine::new("> ");
        input.set_text("hi");
        let text = render_text(&input, 20);
        assert!(text.starts_with("> hi"));
    }

    #[test]
    fn set_text_and_clear() {
        let mut input = InputLine::new("");
        input.set_text("test");
        assert_eq!(input.text(), "test");
        input.clear();
        assert_eq!(input.text(), "");
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn insert_in_middle() {
        let mut input = InputLine::new("");
        input.set_text("ac");
        input.cursor = 1;
        input.handle_key(key(KeyCode::Char('b')));
        assert_eq!(input.text(), "abc");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn empty_history_noop() {
        let mut input = InputLine::new("");
        input.handle_key(key(KeyCode::Up));
        assert_eq!(input.text(), "");
    }

    #[test]
    fn tab_with_no_completion_fn() {
        let mut input = InputLine::new("");
        input.set_text("he");
        let result = input.handle_key(key(KeyCode::Tab));
        assert!(matches!(result, EventResult::Consumed));
        assert_eq!(input.text(), "he"); // unchanged
    }

    #[test]
    fn tab_single_match_completes() {
        let mut input = InputLine::new("");
        input.completion_fn = Some(Box::new(|_text, _cur| vec!["hello world".into()]));
        input.set_text("he");
        input.handle_key(key(KeyCode::Tab));
        assert_eq!(input.text(), "hello world");
    }

    #[test]
    fn tab_multiple_matches_inserts_common_prefix() {
        let mut input = InputLine::new("");
        input.completion_fn = Some(Box::new(|_text, _cur| {
            vec!["hello".into(), "help".into(), "helicopter".into()]
        }));
        input.set_text("he");
        let count = input.try_complete();
        assert_eq!(count, 3);
        assert_eq!(input.text(), "hel");
    }

    #[test]
    fn tab_no_matches() {
        let mut input = InputLine::new("");
        input.completion_fn = Some(Box::new(|_text, _cur| vec![]));
        input.set_text("zz");
        let count = input.try_complete();
        assert_eq!(count, 0);
        assert_eq!(input.text(), "zz");
    }

    #[test]
    fn complete_from_history_basic() {
        let history = vec!["git commit".into(), "git push".into(), "cargo test".into()];
        let f = super::complete_from_history(history);
        let results = f("git", 3);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&"git commit".to_string()));
        assert!(results.contains(&"git push".to_string()));
    }

    #[test]
    fn complete_from_history_case_insensitive() {
        let history = vec!["Hello".into(), "HELP".into()];
        let f = super::complete_from_history(history);
        let results = f("he", 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn complete_from_history_empty_input() {
        let history = vec!["something".into()];
        let f = super::complete_from_history(history);
        let results = f("", 0);
        assert!(results.is_empty());
    }

    #[test]
    fn longest_common_prefix_works() {
        assert_eq!(
            super::longest_common_prefix(&["abc".into(), "abd".into(), "abx".into()]),
            "ab"
        );
        assert_eq!(
            super::longest_common_prefix(&["same".into(), "same".into()]),
            "same"
        );
        assert_eq!(super::longest_common_prefix(&["only".into()]), "only");
        assert_eq!(super::longest_common_prefix(&[]), "");
    }
}
