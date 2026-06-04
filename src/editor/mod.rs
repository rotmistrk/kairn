//! Editor — cursor, mode, command execution over a PieceTable buffer.

mod clipboard;
pub mod command;
mod dispatch_edit;
mod dispatch_search;
mod dispatch_visual;
mod dispatch_yank;
mod editing;
pub mod ephemeral;
pub mod ephemeral_range;
pub mod ex;
pub mod ex_commands;
mod ex_execute;
mod ex_execute_range;
mod execute;
pub mod highlight_state;
mod indent;
pub mod keymap;
pub mod keymap_vim;
mod keymap_vim_modes;
pub mod motions;
mod movement;
pub mod options;
pub mod save;
mod search;
mod visual;
mod visual_block;

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::buffer::PieceTable;
use crate::shared_register::{new_register, RegisterHandle};

use self::command::Command;
use self::ephemeral::EphemeralHighlights;
use self::highlight_state::HighlightState;
use self::keymap::EditorMode;
use self::keymap_vim::VimKeymap;
pub use self::options::EditorOptions;

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
    NoBlame,
    Revert,
    LspGotoDefinition,
    LspGotoShow,
    LspFindReferences,
    LspHover,
    LspCompletion,
    /// Forward unrecognized : command to app-level M-x dispatch.
    AppCommand(String),
    /// :split [file] — horizontal split
    Split(String),
    /// :vsplit [file] — vertical split
    Vsplit(String),
    /// :only — close split
    Only,
}

/// The editor core — buffer + cursor + mode + registers + search.
pub struct Editor {
    pub(crate) buffer: Arc<Mutex<PieceTable>>,
    pub(crate) cursor_line: usize,
    pub(crate) cursor_col: usize,
    /// Sticky column: remembered column for vertical movement.
    pub(crate) desired_col: Option<usize>,
    pub(crate) mode: EditorMode,
    pub(crate) keymap: VimKeymap,
    pub(crate) shared_register: RegisterHandle,
    pub(crate) viewport_scroll: usize,
    pub(crate) viewport_height: usize,
    pub(crate) h_scroll: usize,
    pub(crate) visual_anchor: Option<(usize, usize)>,
    /// Last visual selection line range (for '< '> marks in ex commands).
    pub(crate) last_visual_lines: Option<(usize, usize)>,
    pub(crate) search_pattern: String,
    pub(crate) search_direction_forward: bool,
    /// Cursor position before incremental search started (for elastic backspace).
    pub(crate) incsearch_origin: Option<(usize, usize)>,
    pub(crate) command_buf: String,
    pub(crate) command_history: Vec<String>,
    pub(crate) history_index: Option<usize>,
    pub(crate) history_prefix: String,
    pub(crate) last_find: Option<(char, char)>,
    pub(crate) last_command: Option<Command>,
    pub(crate) status: String,
    pub(crate) options: EditorOptions,
    pub(crate) highlight: Option<HighlightState>,
    pub(crate) ephemeral: EphemeralHighlights,
}

impl Editor {
    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }
    pub fn set_cursor_line(&mut self, v: usize) {
        self.cursor_line = v;
    }
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }
    pub fn set_cursor_col(&mut self, v: usize) {
        self.cursor_col = v;
    }
    pub fn set_viewport_height(&mut self, v: usize) {
        self.viewport_height = v;
    }
    pub fn mode(&self) -> EditorMode {
        self.mode
    }
    pub fn set_mode(&mut self, v: EditorMode) {
        self.mode = v;
    }
    pub fn register(&self) -> String {
        self.shared_register
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .text
            .clone()
    }
    pub fn register_linewise(&self) -> bool {
        self.shared_register.lock().unwrap_or_else(|p| p.into_inner()).linewise
    }
    pub fn register_block(&self) -> bool {
        self.shared_register.lock().unwrap_or_else(|p| p.into_inner()).block
    }
    pub fn set_register(&mut self, text: String, linewise: bool, block: bool) {
        let mut reg = self.shared_register.lock().unwrap_or_else(|p| p.into_inner());
        reg.text = text;
        reg.linewise = linewise;
        reg.block = block;
    }
    pub(crate) fn set_shared_register(&mut self, handle: RegisterHandle) {
        self.shared_register = handle;
    }
    pub fn search_pattern(&self) -> &str {
        &self.search_pattern
    }
    pub fn set_search_pattern(&mut self, pat: impl Into<String>) {
        self.search_pattern = pat.into();
    }
    pub fn visual_anchor(&self) -> Option<(usize, usize)> {
        self.visual_anchor
    }
    pub fn options(&self) -> &EditorOptions {
        &self.options
    }
    pub fn options_mut(&mut self) -> &mut EditorOptions {
        &mut self.options
    }
    pub fn command_buf(&self) -> &str {
        &self.command_buf
    }
    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn open(path: &Path) -> std::io::Result<Self> {
        let buffer = PieceTable::from_file(path.to_str().unwrap_or(""))?;
        Ok(Self::with_buffer(buffer))
    }

    pub fn from_text(content: &str) -> Self {
        Self::with_buffer(PieceTable::from_text(content))
    }

    fn with_buffer(buffer: PieceTable) -> Self {
        Self::with_arc(Arc::new(Mutex::new(buffer)))
    }

    pub fn with_arc(buffer: Arc<Mutex<PieceTable>>) -> Self {
        Self {
            buffer,
            cursor_line: 0,
            cursor_col: 0,
            desired_col: None,
            mode: EditorMode::Normal,
            keymap: VimKeymap::new(),
            shared_register: new_register(),
            viewport_scroll: 0,
            viewport_height: 24,
            h_scroll: 0,
            visual_anchor: None,
            last_visual_lines: None,
            search_pattern: String::new(),
            search_direction_forward: true,
            incsearch_origin: None,
            command_buf: String::new(),
            command_history: Vec::new(),
            history_index: None,
            history_prefix: String::new(),
            last_find: None,
            last_command: None,
            status: String::new(),
            options: EditorOptions::default(),
            highlight: None,
            ephemeral: EphemeralHighlights::new(),
        }
    }
}

// --- Utility methods ---
impl Editor {
    pub fn buf(&self) -> MutexGuard<'_, PieceTable> {
        self.buffer.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Replace buffer content entirely (external reload). Resets cursor to top.
    pub fn replace_content(&mut self, content: &str) {
        *self.buf() = PieceTable::from_text(content);
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.viewport_scroll = 0;
        self.h_scroll = 0;
    }

    pub fn clamp_col(&mut self) {
        let line_len = self.buf().line_len(self.cursor_line);
        let max = if self.mode == EditorMode::Insert {
            line_len
        } else {
            line_len.saturating_sub(1)
        };
        let target = self.desired_col.unwrap_or(self.cursor_col);
        self.cursor_col = target.min(max);
    }

    pub fn clamp_cursor(&mut self) {
        let max_line = self.buf().line_count().saturating_sub(1);
        if self.cursor_line > max_line {
            self.cursor_line = max_line;
        }
        self.clamp_col();
    }

    /// Get the word under the cursor (alphanumeric + underscore).
    pub fn word_under_cursor(&self) -> Option<String> {
        let line = self.buf().line(self.cursor_line)?;
        let chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col;
        if col >= chars.len() || !is_word_char(chars[col]) {
            return None;
        }
        let start = (0..col).rev().take_while(|&i| is_word_char(chars[i])).count();
        let begin = col - start;
        let end = (col..chars.len()).take_while(|&i| is_word_char(chars[i])).count() + col;
        Some(chars[begin..end].iter().collect())
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
