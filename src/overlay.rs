// Modal overlays: session save prompt, session load picker.

use crossterm::event::{KeyCode, KeyEvent};

/// Active overlay state.
pub enum Overlay {
    SavePrompt(SavePrompt),
    SaveFilePrompt(SaveFilePrompt),
    LoadPicker(LoadPicker),
}

/// Text input for session name.
pub struct SavePrompt {
    pub name: String,
    pub cursor: usize,
}

impl SavePrompt {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            cursor: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Enter => {
                if self.name.is_empty() {
                    OverlayAction::Close
                } else {
                    OverlayAction::Save(self.name.clone())
                }
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.name.remove(self.cursor);
                }
                OverlayAction::None
            }
            KeyCode::Char(c) => {
                self.name.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                OverlayAction::None
            }
            _ => OverlayAction::None,
        }
    }
}

/// Text input for file path to save buffer content.
pub struct SaveFilePrompt {
    pub path: String,
    pub cursor: usize,
}

impl SaveFilePrompt {
    pub fn new(default: &str) -> Self {
        let cursor = default.len();
        Self {
            path: default.to_string(),
            cursor,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Enter => {
                if self.path.is_empty() {
                    OverlayAction::Close
                } else {
                    OverlayAction::SaveFile(self.path.clone())
                }
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.path.remove(self.cursor);
                }
                OverlayAction::None
            }
            KeyCode::Char(c) => {
                self.path.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                OverlayAction::None
            }
            _ => OverlayAction::None,
        }
    }
}

/// List of sessions to pick from.
pub struct LoadPicker {
    pub sessions: Vec<String>,
    pub selected: usize,
}

impl LoadPicker {
    pub fn new(sessions: Vec<String>) -> Self {
        Self {
            sessions,
            selected: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                OverlayAction::None
            }
            KeyCode::Down => {
                let max = self.sessions.len().saturating_sub(1);
                if self.selected < max {
                    self.selected += 1;
                }
                OverlayAction::None
            }
            KeyCode::Enter => match self.sessions.get(self.selected) {
                Some(name) => OverlayAction::Load(name.clone()),
                None => OverlayAction::Close,
            },
            _ => OverlayAction::None,
        }
    }
}

pub enum OverlayAction {
    None,
    Close,
    Save(String),
    SaveFile(String),
    Load(String),
}
