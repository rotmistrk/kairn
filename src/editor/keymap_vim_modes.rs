//! VimKeymap mode-specific key handlers (insert, visual).

use txv_core::event::{KeyCode, KeyEvent};
use super::command::Command;
use super::keymap_vim::VimKeymap;

impl VimKeymap {
    pub(super) fn insert_key(&self, key: &KeyEvent) -> Command {
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
            KeyCode::Tab => Command::InsertChar('\t'),
            _ => Command::Noop,
        }
    }

    pub(super) fn visual_key(&mut self, key: &KeyEvent) -> Command {
        if key.modifiers.ctrl { return Command::Noop; }
        match &key.code {
            KeyCode::Esc => Command::ExitVisual,
            KeyCode::Char('h') | KeyCode::Left => Command::MoveLeft,
            KeyCode::Char('l') | KeyCode::Right => Command::MoveRight,
            KeyCode::Char('j') | KeyCode::Down => Command::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Command::MoveUp,
            KeyCode::Char('w') => Command::MoveWordForward,
            KeyCode::Char('b') => Command::MoveWordBackward,
            KeyCode::Char('e') => Command::MoveWordEnd,
            KeyCode::Char('$') => Command::MoveLineEnd,
            KeyCode::Char('0') => Command::MoveLineStart,
            KeyCode::Char('^') => Command::MoveFirstNonBlank,
            KeyCode::Char('G') => Command::MoveFileEnd,
            KeyCode::Char('g') => Command::MoveFileStart,
            KeyCode::Char('d') | KeyCode::Char('x') => Command::VisualDelete,
            KeyCode::Char('y') => Command::VisualYank,
            KeyCode::Char('c') => Command::VisualChange,
            KeyCode::Char('>') => Command::VisualIndent,
            KeyCode::Char('<') => Command::VisualUnindent,
            KeyCode::Char(':') => Command::VisualExCommand,
            _ => Command::Noop,
        }
    }
}
