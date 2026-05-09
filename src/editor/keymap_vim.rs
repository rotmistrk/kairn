//! VimKeymap — vim-style key→command translation for all modes.

use txv_core::event::{KeyCode, KeyEvent};

use super::command::Command;
use super::keymap::{EditorMode, Keymap};

pub struct VimKeymap {
    pending: Option<char>,
    pending_count: Option<usize>,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl VimKeymap {
    pub fn new() -> Self {
        Self {
            pending: None,
            pending_count: None,
        }
    }

    pub fn count(&mut self) -> usize {
        self.pending_count.take().unwrap_or(1)
    }

    fn normal_key(&mut self, key: &KeyEvent) -> Command {
        // Handle pending two-key sequences
        if let Some(prefix) = self.pending.take() {
            return self.two_key(prefix, key);
        }

        // Ctrl combos
        if key.modifiers.ctrl {
            return match &key.code {
                KeyCode::Char('d') => Command::HalfPageDown,
                KeyCode::Char('u') => Command::HalfPageUp,
                KeyCode::Char('f') => Command::PageDown,
                KeyCode::Char('b') => Command::PageUp,
                KeyCode::Char('r') => Command::Redo,
                _ => Command::Noop,
            };
        }

        // Numeric prefix (1-9 starts, 0 extends if already counting)
        if let KeyCode::Char(c @ '1'..='9') = &key.code {
            let digit = (*c as usize) - ('0' as usize);
            self.pending_count = Some(self.pending_count.unwrap_or(0) * 10 + digit);
            return Command::Noop;
        }
        if key.code == KeyCode::Char('0') && self.pending_count.is_some() {
            let val = self.pending_count.unwrap_or(0) * 10;
            self.pending_count = Some(val);
            return Command::Noop;
        }

        match &key.code {
            // Motions
            KeyCode::Char('h') | KeyCode::Left => Command::MoveLeft,
            KeyCode::Char('l') | KeyCode::Right => Command::MoveRight,
            KeyCode::Char('j') | KeyCode::Down => Command::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Command::MoveUp,
            KeyCode::Char('w') => Command::MoveWordForward,
            KeyCode::Char('b') => Command::MoveWordBackward,
            KeyCode::Char('e') => Command::MoveWordEnd,
            KeyCode::Char('0') => Command::MoveLineStart,
            KeyCode::Char('$') => Command::MoveLineEnd,
            KeyCode::Char('^') => Command::MoveFirstNonBlank,
            KeyCode::Char('G') => {
                if let Some(n) = self.pending_count.take() {
                    Command::GotoLine(n)
                } else {
                    Command::MoveFileEnd
                }
            }
            KeyCode::Char('%') => Command::MatchBracket,
            KeyCode::PageUp => Command::PageUp,
            KeyCode::PageDown => Command::PageDown,

            // Insert entry
            KeyCode::Char('i') => Command::EnterInsertMode,
            KeyCode::Char('a') => Command::EnterInsertAfter,
            KeyCode::Char('A') => Command::EnterInsertLineEnd,
            KeyCode::Char('I') => Command::EnterInsertLineStart,
            KeyCode::Char('o') => Command::EnterInsertBelow,
            KeyCode::Char('O') => Command::EnterInsertAbove,

            // Editing
            KeyCode::Char('x') => Command::DeleteCharForward,
            KeyCode::Char('X') => Command::DeleteCharBackward,
            KeyCode::Char('s') => Command::Substitute,
            KeyCode::Char('S') => Command::SubstituteLine,
            KeyCode::Char('C') => Command::ChangeToEnd,
            KeyCode::Char('D') => Command::DeleteToEnd,
            KeyCode::Char('J') => Command::JoinLines,
            KeyCode::Char('~') => Command::ToggleCase,
            KeyCode::Char('u') => Command::Undo,
            KeyCode::Char('p') => Command::Paste,
            KeyCode::Char('P') => Command::PasteBefore,
            KeyCode::Char('.') => Command::DotRepeat,

            // Indent (>> / <<)
            KeyCode::Char('>') => {
                self.pending = Some('>');
                Command::Noop
            }
            KeyCode::Char('<') => {
                self.pending = Some('<');
                Command::Noop
            }

            // Operators
            KeyCode::Char('d') => {
                self.pending = Some('d');
                Command::Noop
            }
            KeyCode::Char('c') => {
                self.pending = Some('c');
                Command::Noop
            }
            KeyCode::Char('y') => {
                self.pending = Some('y');
                Command::Noop
            }

            // Pending char commands
            KeyCode::Char('r') => {
                self.pending = Some('r');
                Command::Noop
            }
            KeyCode::Char('f') => {
                self.pending = Some('f');
                Command::Noop
            }
            KeyCode::Char('F') => {
                self.pending = Some('F');
                Command::Noop
            }
            KeyCode::Char('t') => {
                self.pending = Some('t');
                Command::Noop
            }
            KeyCode::Char('T') => {
                self.pending = Some('T');
                Command::Noop
            }
            KeyCode::Char('g') => {
                self.pending = Some('g');
                Command::Noop
            }

            // Visual
            KeyCode::Char('v') => Command::EnterVisual,
            KeyCode::Char('V') => Command::EnterVisualLine,

            // Search
            KeyCode::Char('/') => Command::EnterSearchMode,
            KeyCode::Char('n') => Command::SearchNext,
            KeyCode::Char('N') => Command::SearchPrev,
            KeyCode::Char('*') => Command::SearchWordForward,
            KeyCode::Char('#') => Command::SearchWordBackward,
            KeyCode::Char(';') => Command::RepeatFind,
            KeyCode::Char(',') => Command::RepeatFindReverse,

            // Command mode
            KeyCode::Char(':') => Command::EnterCommandMode,

            _ => Command::Noop,
        }
    }

    fn two_key(&mut self, prefix: char, key: &KeyEvent) -> Command {
        let ch = match &key.code {
            KeyCode::Char(c) => *c,
            _ => {
                self.pending_count = None;
                return Command::Noop;
            }
        };
        match (prefix, ch) {
            ('d', 'd') => Command::DeleteLine,
            ('d', 'w') => Command::DeleteWord,
            ('d', 'b') => Command::DeleteWordBackward,
            ('d', '0') => Command::DeleteToStart,
            ('d', '$') => Command::DeleteToEnd,
            ('c', 'c') => Command::ChangeLine,
            ('c', 'w') => Command::ChangeWord,
            ('c', '$') => Command::ChangeToEnd,
            ('y', 'y') => Command::YankLine,
            ('y', 'w') => Command::YankWord,
            ('y', '$') => Command::YankToEnd,
            ('g', 'g') => {
                if let Some(n) = self.pending_count.take() {
                    Command::GotoLine(n)
                } else {
                    Command::MoveFileStart
                }
            }
            ('>', '>') => Command::Indent,
            ('<', '<') => Command::Unindent,
            ('r', _) => Command::ReplaceChar(ch),
            ('f', _) => Command::FindChar(ch),
            ('F', _) => Command::FindCharBack(ch),
            ('t', _) => Command::TillChar(ch),
            ('T', _) => Command::TillCharBack(ch),
            // Operator + motion: return the operator, let editor handle motion
            ('d', 'e') | ('d', '^') => Command::OperatorDelete,
            ('c', 'e') | ('c', 'b') | ('c', '0') | ('c', '^') => Command::OperatorChange,
            ('y', 'e') | ('y', 'b') | ('y', '0') | ('y', '^') => Command::OperatorYank,
            _ => {
                self.pending_count = None;
                Command::Noop
            }
        }
    }
}

impl Keymap for VimKeymap {
    fn handle_key(&mut self, key: &KeyEvent, mode: EditorMode) -> Command {
        let cmd = match mode {
            EditorMode::Normal => self.normal_key(key),
            EditorMode::Insert => self.insert_key(key),
            EditorMode::Visual | EditorMode::VisualLine => self.visual_key(key),
            EditorMode::Command | EditorMode::Search => Command::Noop,
        };
        // Apply pending count to the command
        if cmd != Command::Noop {
            if let Some(n) = self.pending_count.take() {
                return Command::Repeat(n, Box::new(cmd));
            }
        }
        cmd
    }

    fn mode_label(&self, mode: EditorMode) -> &str {
        match mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual => "VISUAL",
            EditorMode::VisualLine => "V-LINE",
            EditorMode::Command => "COMMAND",
            EditorMode::Search => "SEARCH",
        }
    }
}
