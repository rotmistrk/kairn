//! Editor — cursor, mode, command execution over a PieceTable buffer.

pub mod command;
pub mod ex;
pub mod keymap;
pub mod keymap_vim;
pub mod motions;
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
    ModeChanged,
    OpenFile(String),
    ShellOutput(String),
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
    // Visual mode anchor
    pub visual_anchor: Option<(usize, usize)>,
    // Search state
    pub search_pattern: String,
    pub search_direction_forward: bool,
    // Command/search input buffer
    pub command_buf: String,
    // Find char repeat
    pub last_find: Option<(char, char)>,
    // Dot repeat
    pub last_command: Option<Command>,
    // Status message
    pub status: String,
    // Editor options (:set)
    pub options: EditorOptions,
}

/// Editor display options controlled by :set.
#[derive(Debug, Clone)]
pub struct EditorOptions {
    /// Show invisible characters (spaces, tabs, EOL).
    pub list: bool,
    /// Show line numbers in gutter.
    pub number: bool,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            list: false,
            number: true,
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

    pub fn execute(&mut self, cmd: Command) -> EditorAction {
        // Store for dot repeat (only editing commands)
        if should_record(&cmd) {
            self.last_command = Some(cmd.clone());
        }
        self.dispatch(cmd)
    }

    fn dispatch(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::Noop => EditorAction::None,

            // Movement
            Command::MoveLeft => {
                self.move_left();
                EditorAction::CursorMoved
            }
            Command::MoveRight => {
                self.move_right();
                EditorAction::CursorMoved
            }
            Command::MoveUp => {
                self.move_up();
                EditorAction::CursorMoved
            }
            Command::MoveDown => {
                self.move_down();
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
            Command::MoveWordEnd => {
                self.move_word_end();
                EditorAction::CursorMoved
            }
            Command::MoveLineStart => {
                self.cursor_col = 0;
                EditorAction::CursorMoved
            }
            Command::MoveLineEnd => {
                self.move_line_end();
                EditorAction::CursorMoved
            }
            Command::MoveFirstNonBlank => {
                self.move_first_non_blank();
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
            Command::GotoLine(n) => {
                self.goto_line(n);
                EditorAction::CursorMoved
            }
            Command::HalfPageDown => {
                self.half_page_down();
                EditorAction::CursorMoved
            }
            Command::HalfPageUp => {
                self.half_page_up();
                EditorAction::CursorMoved
            }
            Command::PageDown => {
                self.page_down();
                EditorAction::CursorMoved
            }
            Command::PageUp => {
                self.page_up();
                EditorAction::CursorMoved
            }
            Command::MatchBracket => {
                self.match_bracket();
                EditorAction::CursorMoved
            }

            // Find char
            Command::FindChar(ch) => {
                self.find_char('f', ch);
                EditorAction::CursorMoved
            }
            Command::FindCharBack(ch) => {
                self.find_char('F', ch);
                EditorAction::CursorMoved
            }
            Command::TillChar(ch) => {
                self.find_char('t', ch);
                EditorAction::CursorMoved
            }
            Command::TillCharBack(ch) => {
                self.find_char('T', ch);
                EditorAction::CursorMoved
            }
            Command::RepeatFind => {
                self.repeat_find(false);
                EditorAction::CursorMoved
            }
            Command::RepeatFindReverse => {
                self.repeat_find(true);
                EditorAction::CursorMoved
            }

            // Insert mode entry
            Command::EnterInsertMode => {
                self.mode = EditorMode::Insert;
                EditorAction::ModeChanged
            }
            Command::EnterInsertAfter => {
                self.enter_insert_after();
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineEnd => {
                self.mode = EditorMode::Insert;
                self.cursor_col = self.buffer.line_len(self.cursor_line);
                EditorAction::ModeChanged
            }
            Command::EnterInsertLineStart => {
                self.mode = EditorMode::Insert;
                self.cursor_col = motions::first_non_blank(&self.buffer, self.cursor_line);
                EditorAction::ModeChanged
            }
            Command::EnterInsertBelow | Command::NewlineBelow => {
                self.open_line_below();
                EditorAction::ContentChanged
            }
            Command::EnterInsertAbove | Command::NewlineAbove => {
                self.open_line_above();
                EditorAction::ContentChanged
            }
            Command::ExitInsertMode => {
                self.exit_insert();
                EditorAction::ModeChanged
            }

            // Editing
            Command::InsertChar(ch) => {
                self.insert_char(ch);
                EditorAction::ContentChanged
            }
            Command::InsertNewline => {
                self.insert_newline();
                EditorAction::ContentChanged
            }
            Command::DeleteCharForward => {
                self.delete_char_forward();
                EditorAction::ContentChanged
            }
            Command::DeleteCharBackward => {
                self.delete_char_backward();
                EditorAction::ContentChanged
            }
            Command::DeleteLine => {
                self.delete_line();
                EditorAction::ContentChanged
            }
            Command::DeleteWord => {
                self.delete_word();
                EditorAction::ContentChanged
            }
            Command::DeleteWordBackward => {
                self.delete_word_backward();
                EditorAction::ContentChanged
            }
            Command::DeleteToEnd => {
                self.delete_to_end();
                EditorAction::ContentChanged
            }
            Command::DeleteToStart => {
                self.delete_to_start();
                EditorAction::ContentChanged
            }
            Command::ChangeWord => {
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::ChangeLine => {
                self.change_line();
                EditorAction::ContentChanged
            }
            Command::ChangeToEnd => {
                self.delete_to_end();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::Substitute => {
                self.delete_char_forward();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::SubstituteLine => {
                self.change_line();
                EditorAction::ContentChanged
            }
            Command::JoinLines => {
                self.join_lines();
                EditorAction::ContentChanged
            }
            Command::ToggleCase => {
                self.toggle_case();
                EditorAction::ContentChanged
            }
            Command::ReplaceChar(ch) => {
                self.replace_char(ch);
                EditorAction::ContentChanged
            }
            Command::Indent => {
                self.indent_line();
                EditorAction::ContentChanged
            }
            Command::Unindent => {
                self.unindent_line();
                EditorAction::ContentChanged
            }

            // Operators (simplified — act on current word/line)
            Command::OperatorDelete => {
                self.delete_word();
                EditorAction::ContentChanged
            }
            Command::OperatorChange => {
                self.delete_word();
                self.mode = EditorMode::Insert;
                EditorAction::ContentChanged
            }
            Command::OperatorYank => {
                self.register = self.buffer.line(self.cursor_line).unwrap_or_default();
                EditorAction::None
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
            Command::YankWord => {
                self.yank_word();
                EditorAction::None
            }
            Command::YankToEnd => {
                self.yank_to_end();
                EditorAction::None
            }
            Command::Paste => {
                self.paste_after();
                EditorAction::ContentChanged
            }
            Command::PasteBefore => {
                self.paste_before();
                EditorAction::ContentChanged
            }

            // Visual mode
            Command::EnterVisual => {
                self.enter_visual();
                EditorAction::ModeChanged
            }
            Command::EnterVisualLine => {
                self.enter_visual_line();
                EditorAction::ModeChanged
            }
            Command::ExitVisual => {
                self.exit_visual();
                EditorAction::ModeChanged
            }
            Command::VisualDelete => {
                self.visual_delete();
                EditorAction::ContentChanged
            }
            Command::VisualYank => {
                self.visual_yank();
                EditorAction::None
            }
            Command::VisualChange => {
                self.visual_change();
                EditorAction::ContentChanged
            }
            Command::VisualIndent => {
                self.visual_indent();
                EditorAction::ContentChanged
            }
            Command::VisualUnindent => {
                self.visual_unindent();
                EditorAction::ContentChanged
            }
            Command::VisualExCommand => {
                self.visual_ex_command();
                EditorAction::ModeChanged
            }

            // Search
            Command::EnterSearchMode => {
                self.mode = EditorMode::Search;
                self.command_buf.clear();
                EditorAction::ModeChanged
            }
            Command::SearchForward(ref pat) => {
                self.search_forward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchBackward(ref pat) => {
                self.search_backward(pat);
                EditorAction::CursorMoved
            }
            Command::SearchNext => {
                self.search_next();
                EditorAction::CursorMoved
            }
            Command::SearchPrev => {
                self.search_prev();
                EditorAction::CursorMoved
            }
            Command::SearchWordForward => {
                self.search_word(true);
                EditorAction::CursorMoved
            }
            Command::SearchWordBackward => {
                self.search_word(false);
                EditorAction::CursorMoved
            }

            // Command mode
            Command::EnterCommandMode => {
                self.mode = EditorMode::Command;
                self.command_buf.clear();
                EditorAction::ModeChanged
            }
            Command::ExCommand(ref input) => self.execute_ex(input.clone()),

            // File
            Command::Save => EditorAction::SaveRequested,
            Command::CloseBuffer => EditorAction::CloseRequested,

            // Dot repeat
            Command::DotRepeat => {
                if let Some(last) = self.last_command.clone() {
                    self.dispatch(last)
                } else {
                    EditorAction::None
                }
            }

            // Count repeat
            Command::Repeat(n, cmd) => {
                match *cmd {
                    // Line-oriented commands: apply to N lines from cursor
                    Command::YankLine => {
                        self.yank_lines(n);
                        EditorAction::None
                    }
                    Command::DeleteLine => {
                        self.delete_lines(n);
                        EditorAction::ContentChanged
                    }
                    Command::ChangeLine => {
                        self.change_lines(n);
                        EditorAction::ContentChanged
                    }
                    Command::Indent => {
                        self.indent_lines(n);
                        EditorAction::ContentChanged
                    }
                    Command::Unindent => {
                        self.unindent_lines(n);
                        EditorAction::ContentChanged
                    }
                    Command::JoinLines => {
                        for _ in 0..n {
                            self.join_lines();
                        }
                        EditorAction::ContentChanged
                    }
                    // All other commands: just repeat N times
                    other => {
                        let mut last_action = EditorAction::None;
                        for _ in 0..n {
                            last_action = self.dispatch(other.clone());
                        }
                        last_action
                    }
                }
            }
        }
    }
}

fn should_record(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::InsertChar(_)
            | Command::InsertNewline
            | Command::DeleteCharForward
            | Command::DeleteCharBackward
            | Command::DeleteLine
            | Command::DeleteWord
            | Command::DeleteToEnd
            | Command::ChangeWord
            | Command::ChangeLine
            | Command::ChangeToEnd
            | Command::Substitute
            | Command::SubstituteLine
            | Command::JoinLines
            | Command::ToggleCase
            | Command::ReplaceChar(_)
            | Command::Indent
            | Command::Unindent
            | Command::Paste
            | Command::PasteBefore
    )
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

    fn goto_line(&mut self, n: usize) {
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

// --- Editing methods ---
impl Editor {
    fn enter_insert_after(&mut self) {
        self.mode = EditorMode::Insert;
        let len = self.buffer.line_len(self.cursor_line);
        if self.cursor_col < len {
            self.cursor_col += 1;
        }
    }

    fn exit_insert(&mut self) {
        self.mode = EditorMode::Normal;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn open_line_below(&mut self) {
        self.mode = EditorMode::Insert;
        let line_len = self.buffer.line_len(self.cursor_line);
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
            self.buffer.insert(offset, "\n");
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    fn open_line_above(&mut self) {
        self.mode = EditorMode::Insert;
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
            self.buffer.insert(offset, "\n");
            self.cursor_col = 0;
        }
    }

    fn insert_char(&mut self, ch: char) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            if ch == '\t' {
                self.buffer.insert(offset, "    ");
                self.cursor_col += 4;
            } else {
                self.buffer.insert(offset, &ch.to_string());
                self.cursor_col += 1;
            }
        }
    }

    fn insert_newline(&mut self) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            self.buffer.insert(offset, "\n");
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    fn delete_char_forward(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_line);
        if self.cursor_col < line_len {
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                let content = self.buffer.content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buffer.delete(offset, offset + ch_len);
            }
        }
    }

    fn delete_char_backward(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                let content = self.buffer.content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buffer.delete(offset, offset + ch_len);
            }
        } else if self.cursor_line > 0 {
            let prev_len = self.buffer.line_len(self.cursor_line - 1);
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
                self.buffer.delete(offset - 1, offset);
                self.cursor_line -= 1;
                self.cursor_col = prev_len;
            }
        }
    }

    fn delete_line(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        self.register = line;
        let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let end = if self.cursor_line + 1 < self.buffer.line_count() {
            self.buffer.line_col_to_offset(self.cursor_line + 1, 0).unwrap_or(start)
        } else {
            self.buffer.content().len()
        };
        if start < end {
            self.buffer.delete(start, end);
        }
        if self.cursor_line >= self.buffer.line_count() && self.cursor_line > 0 {
            self.cursor_line -= 1;
        }
        self.clamp_col();
    }

    fn delete_word(&mut self) {
        let start_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (new_line, new_col) = motions::word_forward(&self.buffer, self.cursor_line, self.cursor_col);
        let end_offset = self
            .buffer
            .line_col_to_offset(new_line, new_col)
            .unwrap_or(start_offset);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            let (l, c) = self.buffer.offset_to_line_col(start_offset);
            self.cursor_line = l;
            self.cursor_col = c;
        }
    }

    fn delete_word_backward(&mut self) {
        let end_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (new_line, new_col) = motions::word_backward(&self.buffer, self.cursor_line, self.cursor_col);
        let start_offset = self.buffer.line_col_to_offset(new_line, new_col).unwrap_or(end_offset);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            self.cursor_line = new_line;
            self.cursor_col = new_col;
        }
    }

    fn delete_to_start(&mut self) {
        let end_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let start_offset = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            self.cursor_col = 0;
        }
    }

    fn delete_to_end(&mut self) {
        let start = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self
            .buffer
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
            self.buffer.delete(start, end);
        }
        self.clamp_col();
    }

    fn change_line(&mut self) {
        let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self
            .buffer
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            self.buffer.delete(start, end);
        }
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }

    fn join_lines(&mut self) {
        if self.cursor_line + 1 >= self.buffer.line_count() {
            return;
        }
        let line_len = self.buffer.line_len(self.cursor_line);
        let end_offset = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
        // Delete the newline
        self.buffer.delete(end_offset, end_offset + 1);
        // Remove leading whitespace from joined line and add a space
        let next_line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let after_join = &next_line[line_len..];
        let ws_count = after_join.chars().take_while(|c| c.is_whitespace()).count();
        if ws_count > 0 {
            let ws_start = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
            let ws_end = self
                .buffer
                .line_col_to_offset(self.cursor_line, line_len + ws_count)
                .unwrap_or(ws_start);
            self.buffer.delete(ws_start, ws_end);
        }
        // Insert a space
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
            self.buffer.insert(offset, " ");
        }
    }

    fn toggle_case(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        if let Some(ch) = line.chars().nth(self.cursor_col) {
            let toggled = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap_or(ch)
            } else {
                ch.to_uppercase().next().unwrap_or(ch)
            };
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                self.buffer.delete(offset, offset + ch.len_utf8());
                self.buffer.insert(offset, &toggled.to_string());
            }
            self.cursor_col += 1;
            self.clamp_col();
        }
    }

    fn replace_char(&mut self, ch: char) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            let content = self.buffer.content();
            let old_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            self.buffer.delete(offset, offset + old_len);
            self.buffer.insert(offset, &ch.to_string());
        }
    }

    fn indent_line(&mut self) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
            self.buffer.insert(offset, "    ");
            self.cursor_col += 4;
        }
    }

    fn unindent_line(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let remove = if line.starts_with("    ") {
            4
        } else if line.starts_with('\t') {
            1
        } else {
            line.chars().take_while(|c| c.is_whitespace()).count().min(4)
        };
        if remove > 0 {
            let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
            let end = self
                .buffer
                .line_col_to_offset(self.cursor_line, remove)
                .unwrap_or(start);
            self.buffer.delete(start, end);
            self.cursor_col = self.cursor_col.saturating_sub(remove);
        }
    }
}

// --- Clipboard methods ---
impl Editor {
    fn paste_after(&mut self) {
        if !self.register.is_empty() {
            let line_len = self.buffer.line_len(self.cursor_line);
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
                let text = format!("\n{}", self.register);
                self.buffer.insert(offset, &text);
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    fn paste_before(&mut self) {
        if !self.register.is_empty() {
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
                let text = format!("{}\n", self.register);
                self.buffer.insert(offset, &text);
                self.cursor_col = 0;
            }
        }
    }

    fn yank_word(&mut self) {
        let start = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (nl, nc) = motions::word_forward(&self.buffer, self.cursor_line, self.cursor_col);
        let end = self.buffer.line_col_to_offset(nl, nc).unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
        }
    }

    fn yank_to_end(&mut self) {
        let start = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self
            .buffer
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
        }
    }

    fn apply_set_option(&mut self, opt: &str) {
        match opt {
            "list" | "li" => self.options.list = true,
            "nolist" | "noli" => self.options.list = false,
            "number" | "nu" => self.options.number = true,
            "nonumber" | "nonu" => self.options.number = false,
            _ => {
                self.status = format!("Unknown option: {opt}");
            }
        }
    }

    fn yank_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        let mut result = String::new();
        for i in self.cursor_line..end_line {
            result.push_str(&self.buffer.line(i).unwrap_or_default());
            result.push('\n');
        }
        self.register = result;
    }

    fn delete_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.delete_line();
        }
    }

    fn change_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.delete_line();
        }
        // Insert empty line at cursor and enter insert mode
        let offset = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        self.buffer.insert(offset, "\n");
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }

    fn indent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        for line in self.cursor_line..end_line {
            if let Some(offset) = self.buffer.line_col_to_offset(line, 0) {
                self.buffer.insert(offset, "    ");
            }
        }
    }

    fn unindent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        for line in self.cursor_line..end_line {
            let text = self.buffer.line(line).unwrap_or_default();
            let spaces = text.chars().take(4).take_while(|c| *c == ' ').count();
            if spaces > 0 {
                if let Some(offset) = self.buffer.line_col_to_offset(line, 0) {
                    self.buffer.delete(offset, offset + spaces);
                }
            }
        }
    }
}

// --- Visual mode methods ---
impl Editor {
    fn enter_visual(&mut self) {
        self.mode = EditorMode::Visual;
        self.visual_anchor = Some((self.cursor_line, self.cursor_col));
    }

    fn enter_visual_line(&mut self) {
        self.mode = EditorMode::VisualLine;
        self.visual_anchor = Some((self.cursor_line, 0));
    }

    fn exit_visual(&mut self) {
        self.mode = EditorMode::Normal;
        self.visual_anchor = None;
    }

    /// Get the visual selection range as (start_offset, end_offset).
    pub fn visual_range(&self) -> Option<(usize, usize)> {
        let (al, ac) = self.visual_anchor?;
        let (cl, cc) = (self.cursor_line, self.cursor_col);
        let anchor_off = self.buffer.line_col_to_offset(al, ac)?;
        let cursor_off = self.buffer.line_col_to_offset(cl, cc)?;
        // Include the char under cursor
        let content = self.buffer.content();
        let end_extra = content[cursor_off..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        if anchor_off <= cursor_off {
            Some((anchor_off, cursor_off + end_extra))
        } else {
            let anchor_extra = content[anchor_off..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            Some((cursor_off, anchor_off + anchor_extra))
        }
    }

    /// Get visual line range as (start_line, end_line).
    pub fn visual_line_range(&self) -> Option<(usize, usize)> {
        let (al, _) = self.visual_anchor?;
        let cl = self.cursor_line;
        Some((al.min(cl), al.max(cl)))
    }

    fn visual_delete(&mut self) {
        if self.mode == EditorMode::VisualLine {
            if let Some((start_line, end_line)) = self.visual_line_range() {
                let start = self.buffer.line_col_to_offset(start_line, 0).unwrap_or(0);
                let end = if end_line + 1 < self.buffer.line_count() {
                    self.buffer.line_col_to_offset(end_line + 1, 0).unwrap_or(start)
                } else {
                    self.buffer.content().len()
                };
                if end > start {
                    let content = self.buffer.content();
                    self.register = content[start..end].to_string();
                    self.buffer.delete(start, end);
                }
                self.cursor_line = start_line.min(self.buffer.line_count().saturating_sub(1));
                self.cursor_col = 0;
            }
        } else if let Some((start, end)) = self.visual_range() {
            if end > start {
                let content = self.buffer.content();
                self.register = content[start..end].to_string();
                self.buffer.delete(start, end);
                let (l, c) = self.buffer.offset_to_line_col(start);
                self.cursor_line = l;
                self.cursor_col = c;
            }
        }
        self.exit_visual();
    }

    fn visual_yank(&mut self) {
        if self.mode == EditorMode::VisualLine {
            if let Some((start_line, end_line)) = self.visual_line_range() {
                let start = self.buffer.line_col_to_offset(start_line, 0).unwrap_or(0);
                let end = if end_line + 1 < self.buffer.line_count() {
                    self.buffer.line_col_to_offset(end_line + 1, 0).unwrap_or(start)
                } else {
                    self.buffer.content().len()
                };
                let content = self.buffer.content();
                self.register = content[start..end].to_string();
            }
        } else if let Some((start, end)) = self.visual_range() {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
        }
        self.exit_visual();
        self.status = "yanked".to_string();
    }

    fn visual_change(&mut self) {
        self.visual_delete();
        self.mode = EditorMode::Insert;
    }

    fn visual_ex_command(&mut self) {
        let range = if self.mode == EditorMode::VisualLine {
            "'<,'>".to_string()
        } else {
            "'<,'>".to_string()
        };
        self.exit_visual();
        self.mode = EditorMode::Command;
        self.command_buf = range;
    }

    fn visual_indent(&mut self) {
        let (start_line, end_line) = match self.visual_line_range() {
            Some(r) => r,
            None => {
                let (al, _) = self.visual_anchor.unwrap_or((self.cursor_line, 0));
                (al.min(self.cursor_line), al.max(self.cursor_line))
            }
        };
        for line in (start_line..=end_line).rev() {
            if let Some(offset) = self.buffer.line_col_to_offset(line, 0) {
                self.buffer.insert(offset, "    ");
            }
        }
        self.exit_visual();
    }

    fn visual_unindent(&mut self) {
        let (start_line, end_line) = match self.visual_line_range() {
            Some(r) => r,
            None => {
                let (al, _) = self.visual_anchor.unwrap_or((self.cursor_line, 0));
                (al.min(self.cursor_line), al.max(self.cursor_line))
            }
        };
        for line in (start_line..=end_line).rev() {
            let text = self.buffer.line(line).unwrap_or_default();
            let remove = if text.starts_with("    ") {
                4
            } else if text.starts_with('\t') {
                1
            } else {
                text.chars().take_while(|c| c.is_whitespace()).count().min(4)
            };
            if remove > 0 {
                let start = self.buffer.line_col_to_offset(line, 0).unwrap_or(0);
                let end = self.buffer.line_col_to_offset(line, remove).unwrap_or(start);
                self.buffer.delete(start, end);
            }
        }
        self.exit_visual();
    }
}

// --- Search methods ---
impl Editor {
    fn search_forward(&mut self, pattern: &str) {
        if pattern.is_empty() {
            return;
        }
        self.search_pattern = pattern.to_string();
        self.search_direction_forward = true;
        self.search_next();
    }

    fn search_backward(&mut self, pattern: &str) {
        if pattern.is_empty() {
            return;
        }
        self.search_pattern = pattern.to_string();
        self.search_direction_forward = false;
        self.search_prev();
    }

    fn search_next(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        let content = self.buffer.content();
        let start_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        // Search from after cursor
        let search_from = start_offset + 1;
        if let Some(pos) = content[search_from..].find(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(search_from + pos);
            self.cursor_line = l;
            self.cursor_col = c;
        } else if let Some(pos) = content[..start_offset].find(&self.search_pattern) {
            // Wrap around
            let (l, c) = self.buffer.offset_to_line_col(pos);
            self.cursor_line = l;
            self.cursor_col = c;
            self.status = "search wrapped".to_string();
        }
    }

    fn search_prev(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        let content = self.buffer.content();
        let start_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        // Search backward from before cursor
        if let Some(pos) = content[..start_offset].rfind(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(pos);
            self.cursor_line = l;
            self.cursor_col = c;
        } else if let Some(pos) = content[start_offset + 1..].rfind(&self.search_pattern) {
            // Wrap around
            let (l, c) = self.buffer.offset_to_line_col(start_offset + 1 + pos);
            self.cursor_line = l;
            self.cursor_col = c;
            self.status = "search wrapped".to_string();
        }
    }

    fn search_word(&mut self, forward: bool) {
        if let Some(word) = motions::word_at(&self.buffer, self.cursor_line, self.cursor_col) {
            self.search_pattern = word;
            self.search_direction_forward = forward;
            if forward {
                self.search_next();
            } else {
                self.search_prev();
            }
        }
    }
}

// --- Ex command execution ---
impl Editor {
    fn execute_ex(&mut self, input: String) -> EditorAction {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return EditorAction::None;
        }

        // Simple commands
        match trimmed {
            "w" => return EditorAction::SaveRequested,
            "q" => {
                if self.buffer.is_dirty() {
                    self.status = "No write since last change (use :q! to override)".to_string();
                    return EditorAction::None;
                }
                return EditorAction::CloseRequested;
            }
            "q!" => return EditorAction::CloseRequested,
            "wq" | "x" => return EditorAction::SaveRequested,
            _ => {}
        }

        // :e filename — open file
        if let Some(filename) = trimmed.strip_prefix("e ") {
            let filename = filename.trim();
            if !filename.is_empty() {
                return EditorAction::OpenFile(filename.to_string());
            }
        }

        // :!command (bare, no range) — run and show output
        if let Some(cmd) = trimmed.strip_prefix('!') {
            let cmd = cmd.trim();
            if !cmd.is_empty() {
                let output = match std::process::Command::new("sh").arg("-c").arg(cmd).output() {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => {
                        self.status = format!("Shell error: {e}");
                        return EditorAction::None;
                    }
                };
                return EditorAction::ShellOutput(output);
            }
        }

        // Goto line
        if let Ok(n) = trimmed.parse::<usize>() {
            self.goto_line(n);
            return EditorAction::CursorMoved;
        }

        // Full ex command parsing
        let total = self.buffer.line_count();
        if let Some(ex_cmd) = ex::parse_ex_full(trimmed, self.cursor_line, total) {
            match ex_cmd {
                ex::ExCommand::Save => return EditorAction::SaveRequested,
                ex::ExCommand::Quit => return EditorAction::CloseRequested,
                ex::ExCommand::SaveQuit => return EditorAction::SaveRequested,
                ex::ExCommand::GotoLine(n) => {
                    self.goto_line(n);
                    return EditorAction::CursorMoved;
                }
                ex::ExCommand::Delete { start, end } => {
                    self.ex_delete(start, end);
                    return EditorAction::ContentChanged;
                }
                ex::ExCommand::Yank { start, end } => {
                    self.ex_yank(start, end);
                    return EditorAction::None;
                }
                ex::ExCommand::Substitute {
                    start,
                    end,
                    pattern,
                    replacement,
                    global,
                } => {
                    self.ex_substitute(start, end, &pattern, &replacement, global);
                    return EditorAction::ContentChanged;
                }
                ex::ExCommand::Shell { start, end, command } => {
                    self.ex_shell(start, end, &command);
                    return EditorAction::ContentChanged;
                }
                ex::ExCommand::Set(opt) => {
                    self.apply_set_option(&opt);
                    return EditorAction::None;
                }
            }
        }

        self.status = format!("Unknown: {trimmed}");
        EditorAction::None
    }

    fn ex_delete(&mut self, start: usize, end: usize) {
        let total = self.buffer.line_count();
        let end = end.min(total.saturating_sub(1));
        let start_off = self.buffer.line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buffer.line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buffer.content().len()
        };
        if end_off > start_off {
            let content = self.buffer.content();
            self.register = content[start_off..end_off].to_string();
            self.buffer.delete(start_off, end_off);
        }
        self.cursor_line = start.min(self.buffer.line_count().saturating_sub(1));
        self.cursor_col = 0;
        let count = end - start + 1;
        self.status = format!("{count} line(s) deleted");
    }

    fn ex_yank(&mut self, start: usize, end: usize) {
        let total = self.buffer.line_count();
        let end = end.min(total.saturating_sub(1));
        let start_off = self.buffer.line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buffer.line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buffer.content().len()
        };
        let content = self.buffer.content();
        self.register = content[start_off..end_off].to_string();
        let count = end - start + 1;
        self.status = format!("{count} line(s) yanked");
    }

    fn ex_substitute(&mut self, start: usize, end: usize, pattern: &str, replacement: &str, global: bool) {
        let total = self.buffer.line_count();
        let end = end.min(total.saturating_sub(1));
        let mut count = 0usize;
        // Process lines from end to start to preserve offsets
        for line_idx in (start..=end).rev() {
            let line = self.buffer.line(line_idx).unwrap_or_default();
            let new_line = if global {
                line.replace(pattern, replacement)
            } else {
                line.replacen(pattern, replacement, 1)
            };
            if new_line != line {
                count += 1;
                let line_start = self.buffer.line_col_to_offset(line_idx, 0).unwrap_or(0);
                let line_end = self
                    .buffer
                    .line_col_to_offset(line_idx, line.chars().count())
                    .unwrap_or(line_start);
                self.buffer.delete(line_start, line_end);
                self.buffer.insert(line_start, &new_line);
            }
        }
        self.status = format!("{count} substitution(s)");
    }

    fn ex_shell(&mut self, start: usize, end: usize, command: &str) {
        let total = self.buffer.line_count();
        let end = end.min(total.saturating_sub(1));
        let mut input_lines = Vec::new();
        for i in start..=end {
            input_lines.push(self.buffer.line(i).unwrap_or_default());
        }
        let input = input_lines.join("\n");

        let output = match std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(input.as_bytes()).ok();
                }
                drop(child.stdin.take());
                match child.wait_with_output() {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => {
                        self.status = format!("Shell error: {e}");
                        return;
                    }
                }
            }
            Err(e) => {
                self.status = format!("Shell error: {e}");
                return;
            }
        };

        // Delete original range and insert output
        let start_off = self.buffer.line_col_to_offset(start, 0).unwrap_or(0);
        let end_off = if end + 1 < total {
            self.buffer.line_col_to_offset(end + 1, 0).unwrap_or(start_off)
        } else {
            self.buffer.content().len()
        };
        if end_off > start_off {
            self.buffer.delete(start_off, end_off);
        }
        let trimmed_output = output.trim_end_matches('\n');
        if !trimmed_output.is_empty() {
            let insert_text = if start_off < self.buffer.content().len() || start_off == 0 {
                format!("{trimmed_output}\n")
            } else {
                format!("\n{trimmed_output}")
            };
            self.buffer.insert(start_off, &insert_text);
        }
        self.cursor_line = start;
        self.cursor_col = 0;
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
