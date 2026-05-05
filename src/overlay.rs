// Modal overlays: session save prompt, session load picker.

use crossterm::event::{KeyCode, KeyEvent};

/// Active overlay state.
pub enum Overlay {
    SavePrompt(SavePrompt),
    SaveFilePrompt(SaveFilePrompt),
    LoadPicker(LoadPicker),
    CommandPalette(CommandPalette),
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
    Execute(String),
}

/// Command palette: fuzzy-filtered list of all commands.
pub struct CommandPalette {
    pub query: String,
    pub cursor: usize,
    pub commands: Vec<PaletteEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

/// An entry in the command palette.
#[derive(Clone)]
pub struct PaletteEntry {
    pub name: String,
    pub description: String,
}

impl CommandPalette {
    /// Create a new command palette with the given entries.
    pub fn new(commands: Vec<PaletteEntry>) -> Self {
        let filtered: Vec<usize> = (0..commands.len()).collect();
        Self {
            query: String::new(),
            cursor: 0,
            commands,
            filtered,
            selected: 0,
        }
    }

    /// Handle a key event, returning an action.
    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Esc => OverlayAction::Close,
            KeyCode::Enter => {
                if let Some(&idx) = self.filtered.get(self.selected) {
                    OverlayAction::Execute(self.commands[idx].name.clone())
                } else {
                    OverlayAction::Close
                }
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                OverlayAction::None
            }
            KeyCode::Down => {
                let max = self.filtered.len().saturating_sub(1);
                if self.selected < max {
                    self.selected += 1;
                }
                OverlayAction::None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.query.remove(self.cursor);
                    self.refilter();
                }
                OverlayAction::None
            }
            KeyCode::Char(c) => {
                self.query.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.refilter();
                OverlayAction::None
            }
            _ => OverlayAction::None,
        }
    }

    /// Refilter the command list based on current query.
    fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if q.is_empty() {
                    return true;
                }
                let name = e.name.to_lowercase();
                let desc = e.description.to_lowercase();
                fuzzy_match(&q, &name) || fuzzy_match(&q, &desc)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
    }
}

/// Simple fuzzy match: all chars of needle appear in order in haystack.
fn fuzzy_match(needle: &str, haystack: &str) -> bool {
    let mut hay_iter = haystack.chars();
    for nc in needle.chars() {
        loop {
            match hay_iter.next() {
                Some(hc) if hc == nc => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn palette() -> CommandPalette {
        CommandPalette::new(vec![
            PaletteEntry {
                name: "quit".into(),
                description: "Exit kairn".into(),
            },
            PaletteEntry {
                name: "save".into(),
                description: "Save current buffer".into(),
            },
            PaletteEntry {
                name: "toggle-tree".into(),
                description: "Show/hide file tree".into(),
            },
        ])
    }

    #[test]
    fn palette_shows_all_initially() {
        let p = palette();
        assert_eq!(p.filtered.len(), 3);
    }

    #[test]
    fn palette_filters_on_type() {
        let mut p = palette();
        p.handle_key(key(KeyCode::Char('q')));
        assert_eq!(p.query, "q");
        assert!(p.filtered.contains(&0)); // "quit"
        assert!(!p.filtered.contains(&1)); // "save" doesn't match
    }

    #[test]
    fn palette_enter_executes() {
        let mut p = palette();
        let action = p.handle_key(key(KeyCode::Enter));
        assert!(matches!(action, OverlayAction::Execute(ref s) if s == "quit"));
    }

    #[test]
    fn palette_esc_closes() {
        let mut p = palette();
        let action = p.handle_key(key(KeyCode::Esc));
        assert!(matches!(action, OverlayAction::Close));
    }

    #[test]
    fn palette_nav_down() {
        let mut p = palette();
        assert_eq!(p.selected, 0);
        p.handle_key(key(KeyCode::Down));
        assert_eq!(p.selected, 1);
    }

    #[test]
    fn fuzzy_match_works() {
        assert!(fuzzy_match("qt", "quit"));
        assert!(fuzzy_match("sv", "save"));
        assert!(!fuzzy_match("xyz", "quit"));
        assert!(fuzzy_match("", "anything"));
    }
}
