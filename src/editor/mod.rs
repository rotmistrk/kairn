//! Editor — cursor, mode, command execution over a PieceTable buffer.

pub mod command;
pub mod ex;
pub mod keymap;
pub mod keymap_vim;
pub mod save;

use std::path::Path;

use crate::buffer::PieceTable;

use self::command::Command;
use self::keymap::EditorMode;
use self::keymap_vim::VimKeymap;

/// Result of executing a command.
#[derive(Debug, PartialEq, Eq)]
pub enum EditorAction {
    None,
    CursorMoved,
    ContentChanged,
    SaveRequested,
    CloseRequested,
}

/// The editor core — buffer + cursor + mode.
pub struct Editor {
    pub buffer: PieceTable,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub mode: EditorMode,
    pub keymap: VimKeymap,
    pub register: String,
    pub viewport_scroll: usize,
    pub viewport_height: usize,
}

impl Editor {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let buffer = PieceTable::from_file(path.to_str().unwrap_or(""))?;
        Ok(Self {
            buffer,
            cursor_line: 0,
            cursor_col: 0,
            mode: EditorMode::Normal,
            keymap: VimKeymap::new(),
            register: String::new(),
            viewport_scroll: 0,
            viewport_height: 24,
        })
    }

    pub fn from_text(content: &str) -> Self {
        Self {
            buffer: PieceTable::from_text(content),
            cursor_line: 0,
            cursor_col: 0,
            mode: EditorMode::Normal,
            keymap: VimKeymap::new(),
            register: String::new(),
            viewport_scroll: 0,
            viewport_height: 24,
        }
    }

    pub fn execute(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::Noop => EditorAction::None,

            // Movement
            Command::MoveLeft => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
                EditorAction::CursorMoved
            }
            Command::MoveRight => {
                let line_len = self.buffer.line_len(self.cursor_line);
                let max = if self.mode == EditorMode::Insert { line_len } else { line_len.saturating_sub(1) };
                if self.cursor_col < max {
                    self.cursor_col += 1;
                }
                EditorAction::CursorMoved
            }
            Command::MoveUp => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.clamp_col();
                }
                EditorAction::CursorMoved
            }
            Command::MoveDown => {
                if self.cursor_line + 1 < self.buffer.line_count() {
                    self.cursor_line += 1;
                    self.clamp_col();
                }
                EditorAction::CursorMoved
            }
            Command::MoveWordForward => {
                self.move_word_forward();
                EditorAction::CursorMoved
            }
            Command::MoveWordBackward => {
                self.move_word_backward();
                EditorAction::CursorMoved
            }
            Command::MoveLineStart => {
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::MoveLineEnd => {
                let len = self.buffer.line_len(self.cursor_line);
                self.cursor_col = len.saturating_sub(1);
                EditorAction::CursorMoved
            }
            Command::MoveFileStart => {
                self.cursor_line = 0;
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::MoveFileEnd => {
                self.cursor_line = self.buffer.line_count().saturating_sub(1);
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::HalfPageDown => {
                let half = self.viewport_height / 2;
                let max_line = self.buffer.line_count().saturating_sub(1);
                self.cursor_line = (self.cursor_line + half).min(max_line);
                self.clamp_col();
                EditorAction::CursorMoved
            }
            Command::HalfPageUp => {
                let half = self.viewport_height / 2;
                self.cursor_line = self.cursor_line.saturating_sub(half);
                self.clamp_col();
                EditorAction::CursorMoved
            }

            // Insert mode entry
            Command::EnterInsertMode => {
                self.mode = EditorMode::Insert;
                EditorAction::None
            }
            Command::EnterInsertAfter => {
                self.mode = EditorMode::Insert;
                let len = self.buffer.line_len(self.cursor_line);
                if self.cursor_col < len {
                    self.cursor_col += 1;
                }
                EditorAction::None
            }
            Command::EnterInsertLineEnd => {
                self.mode = EditorMode::Insert;
                self.cursor_col = self.buffer.line_len(self.cursor_line);
                EditorAction::None
            }
            Command::EnterInsertBelow | Command::NewlineBelow => {
                self.mode = EditorMode::Insert;
                let line_len = self.buffer.line_len(self.cursor_line);
                let offset = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
                self.buffer.insert(offset, "\n");
                self.cursor_line += 1;
                self.cursor_col = 0;
                EditorAction::ContentChanged
            }
            Command::EnterInsertAbove | Command::NewlineAbove => {
                self.mode = EditorMode::Insert;
                let offset = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
                self.buffer.insert(offset, "\n");
                self.cursor_col = 0;
                EditorAction::ContentChanged
            }
            Command::ExitInsertMode => {
                self.mode = EditorMode::Normal;
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
                EditorAction::None
            }

            // Editing
            Command::InsertChar(ch) => {
                if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                    self.buffer.insert(offset, &ch.to_string());
                    self.cursor_col += 1;
                }
                EditorAction::ContentChanged
            }
            Command::InsertNewline => {
                if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                    self.buffer.insert(offset, "\n");
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                EditorAction::ContentChanged
            }
            Command::DeleteCharForward => {
                let line_len = self.buffer.line_len(self.cursor_line);
                if self.cursor_col < line_len {
                    if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                        let content = self.buffer.content();
                        let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                        self.buffer.delete(offset, offset + ch_len);
                    }
                }
                EditorAction::ContentChanged
            }
            Command::DeleteCharBackward => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                        let content = self.buffer.content();
                        let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                        self.buffer.delete(offset, offset + ch_len);
                    }
                } else if self.cursor_line > 0 {
                    // Join with previous line
                    let prev_len = self.buffer.line_len(self.cursor_line - 1);
                    if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
                        self.buffer.delete(offset - 1, offset); // delete the \n
                        self.cursor_line -= 1;
                        self.cursor_col = prev_len;
                    }
                }
                EditorAction::ContentChanged
            }
            Command::DeleteLine => {
                let line = self.buffer.line(self.cursor_line).unwrap_or_default();
                self.register = line;
                let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
                let end = if self.cursor_line + 1 < self.buffer.line_count() {
                    self.buffer.line_col_to_offset(self.cursor_line + 1, 0).unwrap_or(start)
                } else if self.cursor_line > 0 {
                    // Last line — also delete preceding newline
                    let content = self.buffer.content();
                    content.len()
                } else {
                    let content = self.buffer.content();
                    content.len()
                };
                if start < end {
                    self.buffer.delete(start, end);
                }
                if self.cursor_line >= self.buffer.line_count() && self.cursor_line > 0 {
                    self.cursor_line -= 1;
                }
                self.clamp_col();
                EditorAction::ContentChanged
            }
            Command::DeleteWord => {
                let start_offset = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(0);
                self.move_word_forward();
                let end_offset = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(start_offset);
                if end_offset > start_offset {
                    self.buffer.delete(start_offset, end_offset);
                    let (line, col) = self.buffer.offset_to_line_col(start_offset);
                    self.cursor_line = line;
                    self.cursor_col = col;
                }
                EditorAction::ContentChanged
            }

            // Undo/redo
            Command::Undo => {
                self.buffer.undo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }
            Command::Redo => {
                self.buffer.redo();
                self.clamp_cursor();
                EditorAction::ContentChanged
            }

            // Clipboard
            Command::YankLine => {
                self.register = self.buffer.line(self.cursor_line).unwrap_or_default();
                EditorAction::None
            }
            Command::Paste => {
                if !self.register.is_empty() {
                    // Paste after current line
                    let line_len = self.buffer.line_len(self.cursor_line);
                    let offset = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
                    let text = format!("\n{}", self.register);
                    self.buffer.insert(offset, &text);
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
                EditorAction::ContentChanged
            }

            // Ex commands
            Command::ExCommand(ref input) => {
                if input.is_empty() {
                    // Entering command mode — handled by view
                    EditorAction::None
                } else {
                    let parsed = ex::parse_ex(input);
                    if parsed == Command::Save {
                        return EditorAction::SaveRequested;
                    }
                    if parsed == Command::CloseBuffer {
                        return EditorAction::CloseRequested;
                    }
                    EditorAction::None
                }
            }

            // File
            Command::Save => EditorAction::SaveRequested,
            Command::CloseBuffer => EditorAction::CloseRequested,
        }
    }

    fn clamp_col(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_line);
        let max = if self.mode == EditorMode::Insert { line_len } else { line_len.saturating_sub(1) };
        if self.cursor_col > max {
            self.cursor_col = max;
        }
    }

    fn clamp_cursor(&mut self) {
        let max_line = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line > max_line {
            self.cursor_line = max_line;
        }
        self.clamp_col();
    }

    fn move_word_forward(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        // Skip current word chars
        while col < chars.len() && !chars[col].is_whitespace() {
            col += 1;
        }
        // Skip whitespace
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }
        if col >= chars.len() && self.cursor_line + 1 < self.buffer.line_count() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col = col.min(chars.len().saturating_sub(1));
        }
    }

    fn move_word_backward(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                let len = self.buffer.line_len(self.cursor_line);
                self.cursor_col = len.saturating_sub(1);
            }
            return;
        }
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col.saturating_sub(1);
        // Skip whitespace backward
        while col > 0 && chars[col].is_whitespace() {
            col -= 1;
        }
        // Skip word chars backward
        while col > 0 && !chars[col - 1].is_whitespace() {
            col -= 1;
        }
        self.cursor_col = col;
    }
}
