//! Editor — cursor, mode, command execution over a PieceTable buffer.

mod clipboard;
pub mod command;
mod dispatch_edit;
mod editing;
pub mod ex;
mod ex_execute;
mod execute;
mod indent;
pub mod keymap;
pub mod keymap_vim;
mod keymap_vim_modes;
pub mod motions;
pub mod save;
mod search;
mod visual;

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
    ModeChanged,
    OpenFile(String),
    ShellOutput(String),
    SetGlobal(String),
}

/// The editor core — buffer + cursor + mode + registers + search.
pub struct Editor {
    pub buffer: PieceTable,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub mode: EditorMode,
    pub keymap: VimKeymap,
    pub register: String,
    pub viewport_scroll: usize,
    pub viewport_height: usize,
    pub visual_anchor: Option<(usize, usize)>,
    pub search_pattern: String,
    pub search_direction_forward: bool,
    pub command_buf: String,
    pub last_find: Option<(char, char)>,
    pub last_command: Option<Command>,
    pub status: String,
    pub options: EditorOptions,
}

/// Editor display options controlled by :set.
#[derive(Debug, Clone)]
pub struct EditorOptions {
    pub list: bool,
    pub number: bool,
    pub wrap: bool,
    pub tab_width: usize,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            list: false,
            number: true,
            wrap: true,
            tab_width: 4,
        }
    }
}

impl Editor {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let buffer = PieceTable::from_file(path.to_str().unwrap_or(""))?;
        Ok(Self::with_buffer(buffer))
    }

    pub fn from_text(content: &str) -> Self {
        Self::with_buffer(PieceTable::from_text(content))
    }

    fn with_buffer(buffer: PieceTable) -> Self {
        Self {
            buffer,
            cursor_line: 0,
            cursor_col: 0,
            mode: EditorMode::Normal,
            keymap: VimKeymap::new(),
            register: String::new(),
            viewport_scroll: 0,
            viewport_height: 24,
            visual_anchor: None,
            search_pattern: String::new(),
            search_direction_forward: true,
            command_buf: String::new(),
            last_find: None,
            last_command: None,
            status: String::new(),
            options: EditorOptions::default(),
        }
    }
}

// --- Movement methods ---
impl Editor {
    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn move_right(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_line);
        let max = if self.mode == EditorMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        if self.cursor_col < max {
            self.cursor_col += 1;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_col();
        }
    }

    fn move_down(&mut self) {
        if self.cursor_line + 1 < self.buffer.line_count() {
            self.cursor_line += 1;
            self.clamp_col();
        }
    }

    fn move_word_forward(&mut self) {
        let (l, c) = motions::word_forward(&self.buffer, self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    fn move_word_backward(&mut self) {
        let (l, c) = motions::word_backward(&self.buffer, self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    fn move_word_end(&mut self) {
        let (l, c) = motions::word_end(&self.buffer, self.cursor_line, self.cursor_col);
        self.cursor_line = l;
        self.cursor_col = c;
    }

    fn move_line_end(&mut self) {
        let len = self.buffer.line_len(self.cursor_line);
        self.cursor_col = len.saturating_sub(1);
    }

    fn move_first_non_blank(&mut self) {
        self.cursor_col = motions::first_non_blank(&self.buffer, self.cursor_line);
    }

    pub(super) fn goto_line(&mut self, n: usize) {
        let target = n.saturating_sub(1).min(self.buffer.line_count().saturating_sub(1));
        self.cursor_line = target;
        self.cursor_col = 0;
    }

    fn half_page_down(&mut self) {
        let half = self.viewport_height / 2;
        let max_line = self.buffer.line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + half).min(max_line);
        self.clamp_col();
    }

    fn half_page_up(&mut self) {
        let half = self.viewport_height / 2;
        self.cursor_line = self.cursor_line.saturating_sub(half);
        self.clamp_col();
    }

    fn page_down(&mut self) {
        let page = self.viewport_height.saturating_sub(2);
        let max_line = self.buffer.line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + page).min(max_line);
        self.clamp_col();
    }

    fn page_up(&mut self) {
        let page = self.viewport_height.saturating_sub(2);
        self.cursor_line = self.cursor_line.saturating_sub(page);
        self.clamp_col();
    }

    fn match_bracket(&mut self) {
        if let Some((l, c)) = motions::match_bracket(&self.buffer, self.cursor_line, self.cursor_col) {
            self.cursor_line = l;
            self.cursor_col = c;
        }
    }

    fn find_char(&mut self, cmd: char, target: char) {
        self.last_find = Some((cmd, target));
        self.execute_find(cmd, target);
    }

    fn execute_find(&mut self, cmd: char, target: char) {
        let result = match cmd {
            'f' => motions::find_char(&self.buffer, self.cursor_line, self.cursor_col, target),
            'F' => motions::find_char_back(&self.buffer, self.cursor_line, self.cursor_col, target),
            't' => motions::find_char(&self.buffer, self.cursor_line, self.cursor_col, target)
                .map(|c| c.saturating_sub(1).max(self.cursor_col + 1)),
            'T' => motions::find_char_back(&self.buffer, self.cursor_line, self.cursor_col, target)
                .map(|c| (c + 1).min(self.cursor_col.saturating_sub(1))),
            _ => None,
        };
        if let Some(col) = result {
            self.cursor_col = col;
        }
    }

    fn repeat_find(&mut self, reverse: bool) {
        if let Some((cmd, ch)) = self.last_find {
            let actual_cmd = if reverse {
                match cmd {
                    'f' => 'F',
                    'F' => 'f',
                    't' => 'T',
                    'T' => 't',
                    _ => cmd,
                }
            } else {
                cmd
            };
            self.execute_find(actual_cmd, ch);
        }
    }
}

// --- Utility methods ---
impl Editor {
    pub fn clamp_col(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_line);
        let max = if self.mode == EditorMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        if self.cursor_col > max {
            self.cursor_col = max;
        }
    }

    pub fn clamp_cursor(&mut self) {
        let max_line = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line > max_line {
            self.cursor_line = max_line;
        }
        self.clamp_col();
    }
}
