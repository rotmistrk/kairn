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
mod movement;
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
    ForceCloseRequested,
    ModeChanged,
    OpenFile(String),
    ShellOutput(String),
    SetGlobal(String),
    Diff(String),
    NoDiff,
    LspGotoDefinition,
    LspFindReferences,
    LspHover,
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
