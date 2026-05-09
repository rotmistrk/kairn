//! VimKeymap — vim-style key→command translation.

use txv_core::event::{KeyCode, KeyEvent};

use super::command::Command;
use super::keymap::{EditorMode, Keymap};

pub struct VimKeymap {
    pending: Option<char>,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl VimKeymap {
    pub fn new() -> Self {
        Self { pending: None }
    }

    fn normal_key(&mut self, key: &KeyEvent) -> Command {
        // Handle pending two-key sequences
        if let Some(prefix) = self.pending.take() {
            return self.two_key(prefix, key);
        }

        // Ctrl combos first (before plain char matching)
        if key.modifiers.ctrl {
            return match &key.code {
                KeyCode::Char('d') => Command::HalfPageDown,
                KeyCode::Char('u') => Command::HalfPageUp,
                KeyCode::Char('r') => Command::Redo,
                _ => Command::Noop,
            };
        }

        match &key.code {
            KeyCode::Char('h') => Command::MoveLeft,
            KeyCode::Char('l') => Command::MoveRight,
            KeyCode::Char('j') => Command::MoveDown,
            KeyCode::Char('k') => Command::MoveUp,
            KeyCode::Left => Command::MoveLeft,
            KeyCode::Right => Command::MoveRight,
            KeyCode::Down => Command::MoveDown,
            KeyCode::Up => Command::MoveUp,
            KeyCode::Char('w') => Command::MoveWordForward,
            KeyCode::Char('b') => Command::MoveWordBackward,
            KeyCode::Char('0') => Command::MoveLineStart,
            KeyCode::Char('$') => Command::MoveLineEnd,
            KeyCode::Char('G') => Command::MoveFileEnd,
            KeyCode::Char('x') => Command::DeleteCharForward,
            KeyCode::Char('i') => Command::EnterInsertMode,
            KeyCode::Char('a') => Command::EnterInsertAfter,
            KeyCode::Char('A') => Command::EnterInsertLineEnd,
            KeyCode::Char('o') => Command::EnterInsertBelow,
            KeyCode::Char('O') => Command::EnterInsertAbove,
            KeyCode::Char('u') => Command::Undo,
            KeyCode::Char('p') => Command::Paste,
            KeyCode::Char(':') => Command::ExCommand(String::new()),
            // Two-key prefixes
            KeyCode::Char('d') | KeyCode::Char('g') | KeyCode::Char('y') => {
                if let KeyCode::Char(c) = &key.code {
                    self.pending = Some(*c);
                }
                Command::Noop
            }
            _ => Command::Noop,
        }
    }

    fn two_key(&self, prefix: char, key: &KeyEvent) -> Command {
        let KeyCode::Char(ch) = &key.code else {
            return Command::Noop;
        };
        match (prefix, *ch) {
            ('d', 'd') => Command::DeleteLine,
            ('d', 'w') => Command::DeleteWord,
            ('g', 'g') => Command::MoveFileStart,
            ('y', 'y') => Command::YankLine,
            _ => Command::Noop,
        }
    }

    fn insert_key(&self, key: &KeyEvent) -> Command {
        match &key.code {
            KeyCode::Esc => Command::ExitInsertMode,
            KeyCode::Char(ch) => Command::InsertChar(*ch),
            KeyCode::Enter => Command::InsertNewline,
            KeyCode::Backspace => Command::DeleteCharBackward,
            KeyCode::Delete => Command::DeleteCharForward,
            KeyCode::Left => Command::MoveLeft,
            KeyCode::Right => Command::MoveRight,
            KeyCode::Down => Command::MoveDown,
            KeyCode::Up => Command::MoveUp,
            _ => Command::Noop,
        }
    }
}

impl Keymap for VimKeymap {
    fn handle_key(&mut self, key: &KeyEvent, mode: EditorMode) -> Command {
        match mode {
            EditorMode::Normal => self.normal_key(key),
            EditorMode::Insert => self.insert_key(key),
        }
    }

    fn mode_label(&self, mode: EditorMode) -> &str {
        match mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
        }
    }
}
