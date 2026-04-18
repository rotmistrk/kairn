use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

/// Editing mode for the input line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InputMode {
    #[default]
    Emacs,
    Vi,
}

/// Vi sub-mode when in Vi input mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViMode {
    #[default]
    Normal,
    Insert,
}

/// Where to send the composed input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendTarget {
    Kiro,
    Terminal,
}

/// Result of handling a key in the input line.
#[derive(Debug)]
pub enum InputAction {
    None,
    Send { text: String, target: SendTarget },
}

/// A single-line text editor with vi/emacs keybindings.
pub struct InputLine {
    pub text: String,
    pub cursor: usize,
    pub mode: InputMode,
    pub vi_mode: ViMode,
}

impl InputLine {
    pub fn new(mode: InputMode) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            mode,
            vi_mode: ViMode::default(),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> InputAction {
        match self.mode {
            InputMode::Emacs => self.handle_emacs(key),
            InputMode::Vi => self.handle_vi(key),
        }
    }

    fn handle_emacs(&mut self, key: KeyEvent) -> InputAction {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match (ctrl, alt, key.code) {
            (false, false, KeyCode::Enter) => self.take_input(SendTarget::Kiro),
            (false, true, KeyCode::Enter) => self.take_input(SendTarget::Terminal),
            (true, false, KeyCode::Char('a')) => self.do_move(0),
            (true, false, KeyCode::Char('e')) => self.do_move(self.text.len()),
            (true, false, KeyCode::Char('f')) | (false, false, KeyCode::Right) => self.do_right(),
            (true, false, KeyCode::Char('b')) | (false, false, KeyCode::Left) => self.do_left(),
            (true, false, KeyCode::Char('d')) => self.do_edit(|s| s.delete_forward()),
            (true, false, KeyCode::Char('h')) | (false, false, KeyCode::Backspace) => {
                self.do_edit(|s| s.delete_backward())
            }
            (true, false, KeyCode::Char('k')) => self.do_edit(|s| s.text.truncate(s.cursor)),
            (true, false, KeyCode::Char('u')) => self.do_kill_before(),
            (false, false, KeyCode::Char(c)) => self.do_edit(|s| s.insert_char(c)),
            _ => InputAction::None,
        }
    }

    fn handle_vi(&mut self, key: KeyEvent) -> InputAction {
        match self.vi_mode {
            ViMode::Normal => self.handle_vi_normal(key),
            ViMode::Insert => self.handle_vi_insert(key),
        }
    }

    fn handle_vi_normal(&mut self, key: KeyEvent) -> InputAction {
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match (alt, key.code) {
            (false, KeyCode::Enter) => self.take_input(SendTarget::Kiro),
            (true, KeyCode::Enter) => self.take_input(SendTarget::Terminal),
            (false, KeyCode::Char('i')) => self.enter_vi_insert(),
            (false, KeyCode::Char('a')) => {
                self.move_right();
                self.enter_vi_insert()
            }
            (false, KeyCode::Char('A')) => {
                self.cursor = self.text.len();
                self.enter_vi_insert()
            }
            (false, KeyCode::Char('I')) => {
                self.cursor = 0;
                self.enter_vi_insert()
            }
            (false, KeyCode::Char('h')) | (false, KeyCode::Left) => self.do_left(),
            (false, KeyCode::Char('l')) | (false, KeyCode::Right) => self.do_right(),
            (false, KeyCode::Char('0')) => self.do_move(0),
            (false, KeyCode::Char('$')) => self.do_move(self.text.len().saturating_sub(1)),
            (false, KeyCode::Char('x')) => self.do_edit(|s| s.delete_forward()),
            (false, KeyCode::Char('d')) => self.do_edit(|s| {
                s.text.clear();
                s.cursor = 0;
            }),
            _ => InputAction::None,
        }
    }

    fn handle_vi_insert(&mut self, key: KeyEvent) -> InputAction {
        match key.code {
            KeyCode::Esc => {
                self.vi_mode = ViMode::Normal;
                InputAction::None
            }
            KeyCode::Enter => self.take_input(SendTarget::Kiro),
            KeyCode::Backspace => self.do_edit(|s| s.delete_backward()),
            KeyCode::Left => self.do_left(),
            KeyCode::Right => self.do_right(),
            KeyCode::Char(c) => self.do_edit(|s| s.insert_char(c)),
            _ => InputAction::None,
        }
    }

    // --- Action helpers (return InputAction::None) ---

    fn do_move(&mut self, pos: usize) -> InputAction {
        self.cursor = pos;
        InputAction::None
    }

    fn do_left(&mut self) -> InputAction {
        self.move_left();
        InputAction::None
    }

    fn do_right(&mut self) -> InputAction {
        self.move_right();
        InputAction::None
    }

    fn do_edit(&mut self, f: impl FnOnce(&mut Self)) -> InputAction {
        f(self);
        InputAction::None
    }

    fn do_kill_before(&mut self) -> InputAction {
        self.text = self.text[self.cursor..].to_string();
        self.cursor = 0;
        InputAction::None
    }

    fn enter_vi_insert(&mut self) -> InputAction {
        self.vi_mode = ViMode::Insert;
        InputAction::None
    }

    fn take_input(&mut self, target: SendTarget) -> InputAction {
        if self.text.is_empty() {
            return InputAction::None;
        }
        let text = std::mem::take(&mut self.text);
        self.cursor = 0;
        self.vi_mode = ViMode::default();
        InputAction::Send { text, target }
    }

    // --- Cursor/text primitives ---

    fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
        }
    }

    fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor += self.text[self.cursor..]
                .chars()
                .next()
                .map_or(0, |c| c.len_utf8());
        }
    }

    fn delete_backward(&mut self) {
        if self.cursor > 0 {
            let prev = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
            self.text.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            let len = self.text[self.cursor..]
                .chars()
                .next()
                .map_or(0, |c| c.len_utf8());
            self.text.drain(self.cursor..self.cursor + len);
        }
    }

    /// Display hint for the status bar.
    pub fn mode_indicator(&self) -> &str {
        match self.mode {
            InputMode::Emacs => "emacs",
            InputMode::Vi => match self.vi_mode {
                ViMode::Normal => "vi:N",
                ViMode::Insert => "vi:I",
            },
        }
    }
}
