//! Editor core — buffer + cursor + mode + command execution.

pub mod command;
pub mod ex;
pub mod keymap;
pub mod keymap_vim;
pub mod launcher;
pub mod save;

// Re-export launcher functions for backward compatibility with app.rs.
pub use launcher::{launch_editor, launch_shell};

use crate::buffer::PieceTable;
use command::{Command, EditorAction, EditorMode, FocusTarget, VisualKind};

/// Visual selection anchor.
struct Selection {
    kind: VisualKind,
    anchor_line: usize,
    anchor_col: usize,
}

/// The editor: buffer + cursor + mode + selection state.
pub struct Editor {
    buffer: PieceTable,
    cursor_line: usize,
    cursor_col: usize,
    mode: EditorMode,
    keymap: Box<dyn keymap::Keymap>,
    selection: Option<Selection>,
    register: String,
    register_linewise: bool,
    viewport_scroll: usize,
    viewport_height: usize,
    search_pattern: Option<String>,
}

impl Editor {
    /// Create a new editor with the given buffer and keymap.
    pub fn new(buffer: PieceTable, keymap: Box<dyn keymap::Keymap>) -> Self {
        Self {
            buffer,
            cursor_line: 0,
            cursor_col: 0,
            mode: EditorMode::Normal,
            keymap,
            selection: None,
            register: String::new(),
            register_linewise: false,
            viewport_scroll: 0,
            viewport_height: 24,
            search_pattern: None,
        }
    }

    /// Current editor mode.
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Current cursor position.
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_line, self.cursor_col)
    }

    /// Mutable access to the buffer.
    pub fn buffer_mut(&mut self) -> &mut PieceTable {
        &mut self.buffer
    }

    /// Read access to the buffer.
    pub fn buffer(&self) -> &PieceTable {
        &self.buffer
    }

    /// Set viewport height (for page up/down calculations).
    pub fn set_viewport_height(&mut self, h: usize) {
        self.viewport_height = h;
    }
}

// ── Command execution ──

impl Editor {
    /// Execute a command. Returns an [`EditorAction`] for the panel.
    pub fn execute(&mut self, cmd: Command) -> EditorAction {
        match cmd {
            Command::Noop => EditorAction::None,
            // Movement
            Command::MoveLeft => self.move_left(),
            Command::MoveRight => self.move_right(),
            Command::MoveUp => self.move_up(),
            Command::MoveDown => self.move_down(),
            Command::MoveWordForward => self.move_word_forward(),
            Command::MoveWordBackward => self.move_word_backward(),
            Command::MoveLineStart => self.move_line_start(),
            Command::MoveLineEnd => self.move_line_end(),
            Command::MoveFileStart => self.move_file_start(),
            Command::MoveFileEnd => self.move_file_end(),
            Command::PageUp => self.page_up(),
            Command::PageDown => self.page_down(),
            Command::HalfPageUp => self.half_page_up(),
            Command::HalfPageDown => self.half_page_down(),
            Command::GotoLine(n) => self.goto_line(n),
            // Editing
            Command::InsertChar(ch) => self.insert_char(ch),
            Command::InsertNewline => self.insert_newline(),
            Command::DeleteCharForward => self.delete_char_forward(),
            Command::DeleteCharBackward => self.delete_char_backward(),
            Command::DeleteWord => self.delete_word(),
            Command::DeleteWordBackward => self.delete_word_backward(),
            Command::DeleteLine => self.delete_line(),
            Command::DeleteToLineEnd => self.delete_to_line_end(),
            Command::DeleteToLineStart => self.delete_to_line_start(),
            Command::NewlineBelow => self.newline_below(),
            Command::NewlineAbove => self.newline_above(),
            Command::Indent => self.indent(),
            Command::Dedent => self.dedent(),
            Command::JoinLines => self.join_lines(),
            // Undo/redo
            Command::Undo => self.do_undo(),
            Command::Redo => self.do_redo(),
            // Selection
            Command::SelectionStart => self.start_selection(VisualKind::Stream),
            Command::SelectionLineStart => self.start_selection(VisualKind::Line),
            Command::SelectionBlockStart => self.start_selection(VisualKind::Block),
            Command::SelectionCancel => self.cancel_selection(),
            // Clipboard
            Command::Yank => self.yank(),
            Command::YankLine => self.yank_line(),
            Command::Paste => self.paste(),
            Command::PasteBefore => self.paste_before(),
            // Search
            Command::SearchForward(p) => self.search_forward(p),
            Command::SearchBackward(p) => self.search_backward(p),
            Command::SearchNext => self.search_next(),
            Command::SearchPrev => self.search_prev(),
            Command::ClearSearchHighlight => self.clear_search(),
            // Mode
            Command::EnterInsertMode => self.enter_insert(),
            Command::EnterInsertAfter => self.enter_insert_after(),
            Command::EnterInsertLineStart => self.enter_insert_line_start(),
            Command::EnterInsertLineEnd => self.enter_insert_line_end(),
            Command::EnterInsertBelow => self.enter_insert_below(),
            Command::EnterInsertAbove => self.enter_insert_above(),
            Command::ExitInsertMode => self.exit_insert(),
            Command::EnterCommandMode => {
                self.mode = EditorMode::CommandLine;
                EditorAction::CursorMoved
            }
            // Ex commands
            Command::ExCommand(s) => self.run_ex(&s),
            // File
            Command::Save => EditorAction::SaveRequested,
            Command::SaveAs(p) => {
                self.buffer.set_file_path(&p);
                EditorAction::SaveRequested
            }
            Command::SaveAll => EditorAction::SaveRequested,
            Command::CloseBuffer => self.close_buffer(),
            Command::ForceCloseBuffer => EditorAction::ForceCloseRequested,
            Command::OpenFile(p) => EditorAction::OpenFile(p),
            // Focus
            Command::FocusTree => EditorAction::FocusChange(FocusTarget::Tree),
            Command::FocusEditor => EditorAction::FocusChange(FocusTarget::Editor),
            Command::FocusControl => EditorAction::FocusChange(FocusTarget::Control),
            Command::FocusBottom => EditorAction::FocusChange(FocusTarget::Bottom),
            Command::FocusNext => EditorAction::FocusChange(FocusTarget::Next),
            Command::FocusPrev => EditorAction::FocusChange(FocusTarget::Prev),
            Command::Quit => EditorAction::CloseRequested,
            Command::ForceQuit => EditorAction::ForceCloseRequested,
            // Unhandled — return None
            _ => EditorAction::None,
        }
    }
}

// ── Movement ──

impl Editor {
    fn move_left(&mut self) -> EditorAction {
        self.cursor_col = self.cursor_col.saturating_sub(1);
        EditorAction::CursorMoved
    }

    fn move_right(&mut self) -> EditorAction {
        let line_len = self.current_line_len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        }
        EditorAction::CursorMoved
    }

    fn move_up(&mut self) -> EditorAction {
        self.cursor_line = self.cursor_line.saturating_sub(1);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn move_down(&mut self) -> EditorAction {
        let max = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line < max {
            self.cursor_line += 1;
        }
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn move_word_forward(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        // Skip current word
        while col < chars.len() && !chars[col].is_whitespace() {
            col += 1;
        }
        // Skip whitespace
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }
        self.cursor_col = col;
        EditorAction::CursorMoved
    }

    fn move_word_backward(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        col = col.saturating_sub(1);
        // Skip whitespace
        while col > 0 && chars.get(col).is_some_and(|c| c.is_whitespace()) {
            col -= 1;
        }
        // Skip word
        while col > 0 && chars.get(col - 1).is_some_and(|c| !c.is_whitespace()) {
            col -= 1;
        }
        self.cursor_col = col;
        EditorAction::CursorMoved
    }

    fn move_line_start(&mut self) -> EditorAction {
        self.cursor_col = 0;
        EditorAction::CursorMoved
    }

    fn move_line_end(&mut self) -> EditorAction {
        self.cursor_col = self.current_line_len();
        EditorAction::CursorMoved
    }

    fn move_file_start(&mut self) -> EditorAction {
        self.cursor_line = 0;
        self.cursor_col = 0;
        EditorAction::CursorMoved
    }

    fn move_file_end(&mut self) -> EditorAction {
        self.cursor_line = self.buffer.line_count().saturating_sub(1);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn page_up(&mut self) -> EditorAction {
        let h = self.viewport_height.max(1);
        self.cursor_line = self.cursor_line.saturating_sub(h);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn page_down(&mut self) -> EditorAction {
        let h = self.viewport_height.max(1);
        let max = self.buffer.line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + h).min(max);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn half_page_up(&mut self) -> EditorAction {
        let h = self.viewport_height.max(2) / 2;
        self.cursor_line = self.cursor_line.saturating_sub(h);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn half_page_down(&mut self) -> EditorAction {
        let h = self.viewport_height.max(2) / 2;
        let max = self.buffer.line_count().saturating_sub(1);
        self.cursor_line = (self.cursor_line + h).min(max);
        self.clamp_col();
        EditorAction::CursorMoved
    }

    fn goto_line(&mut self, n: usize) -> EditorAction {
        let max = self.buffer.line_count().saturating_sub(1);
        self.cursor_line = n.min(max);
        self.cursor_col = 0;
        EditorAction::CursorMoved
    }
}

// ── Editing ──

impl Editor {
    fn insert_char(&mut self, ch: char) -> EditorAction {
        let offset = self.cursor_offset();
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        self.buffer.insert(offset, s);
        self.cursor_col += 1;
        EditorAction::ContentChanged
    }

    fn insert_newline(&mut self) -> EditorAction {
        let offset = self.cursor_offset();
        self.buffer.insert(offset, "\n");
        self.cursor_line += 1;
        self.cursor_col = 0;
        EditorAction::ContentChanged
    }

    fn delete_char_forward(&mut self) -> EditorAction {
        let offset = self.cursor_offset();
        if offset < self.buffer.len() {
            let line = self.current_line_text();
            let byte_len = line
                .chars()
                .nth(self.cursor_col)
                .map(|c| c.len_utf8())
                .unwrap_or(1);
            self.buffer.delete(offset, offset + byte_len);
        }
        EditorAction::ContentChanged
    }

    fn delete_char_backward(&mut self) -> EditorAction {
        if self.cursor_col > 0 {
            let line = self.current_line_text();
            let byte_col = char_to_byte_col(&line, self.cursor_col);
            let prev_byte = char_to_byte_col(&line, self.cursor_col - 1);
            let line_start = self.line_start_offset();
            self.buffer
                .delete(line_start + prev_byte, line_start + byte_col);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Join with previous line
            let offset = self.line_start_offset();
            let prev_len = self.line_len_at(self.cursor_line - 1);
            self.buffer.delete(offset - 1, offset);
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
        }
        EditorAction::ContentChanged
    }

    fn delete_word(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        while col < chars.len() && !chars[col].is_whitespace() {
            col += 1;
        }
        while col < chars.len() && chars[col].is_whitespace() {
            col += 1;
        }
        let end_byte = char_to_byte_col(&line, col);
        let start_byte = char_to_byte_col(&line, self.cursor_col);
        let ls = self.line_start_offset();
        self.buffer.delete(ls + start_byte, ls + end_byte);
        EditorAction::ContentChanged
    }

    fn delete_word_backward(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor_col;
        col = col.saturating_sub(1);
        while col > 0 && chars.get(col).is_some_and(|c| c.is_whitespace()) {
            col -= 1;
        }
        while col > 0 && chars.get(col - 1).is_some_and(|c| !c.is_whitespace()) {
            col -= 1;
        }
        let ls = self.line_start_offset();
        let start_byte = char_to_byte_col(&line, col);
        let end_byte = char_to_byte_col(&line, self.cursor_col);
        self.buffer.delete(ls + start_byte, ls + end_byte);
        self.cursor_col = col;
        EditorAction::ContentChanged
    }

    fn delete_line(&mut self) -> EditorAction {
        let ls = self.line_start_offset();
        let next_ls = self
            .buffer
            .line_col_to_offset(self.cursor_line + 1, 0)
            .unwrap_or(self.buffer.len());
        if ls < next_ls {
            // Save to register
            self.register = self.buffer.slice(ls, next_ls);
            self.register_linewise = true;
            self.buffer.delete(ls, next_ls);
        }
        let max = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line > max {
            self.cursor_line = max;
        }
        self.clamp_col();
        EditorAction::ContentChanged
    }

    fn delete_to_line_end(&mut self) -> EditorAction {
        let offset = self.cursor_offset();
        let line = self.current_line_text();
        let line_end_byte = self.line_start_offset() + line.len();
        if offset < line_end_byte {
            self.register = self.buffer.slice(offset, line_end_byte);
            self.register_linewise = false;
            self.buffer.delete(offset, line_end_byte);
        }
        EditorAction::ContentChanged
    }

    fn delete_to_line_start(&mut self) -> EditorAction {
        let ls = self.line_start_offset();
        let offset = self.cursor_offset();
        if ls < offset {
            self.buffer.delete(ls, offset);
            self.cursor_col = 0;
        }
        EditorAction::ContentChanged
    }

    fn newline_below(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let ls = self.line_start_offset();
        let end = ls + line.len();
        self.buffer.insert(end, "\n");
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
        EditorAction::ContentChanged
    }

    fn newline_above(&mut self) -> EditorAction {
        let ls = self.line_start_offset();
        self.buffer.insert(ls, "\n");
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
        EditorAction::ContentChanged
    }

    fn indent(&mut self) -> EditorAction {
        let ls = self.line_start_offset();
        self.buffer.insert(ls, "    ");
        self.cursor_col += 4;
        EditorAction::ContentChanged
    }

    fn dedent(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let spaces = line.len() - line.trim_start().len();
        let remove = spaces.min(4);
        if remove > 0 {
            let ls = self.line_start_offset();
            self.buffer.delete(ls, ls + remove);
            self.cursor_col = self.cursor_col.saturating_sub(remove);
        }
        EditorAction::ContentChanged
    }

    fn join_lines(&mut self) -> EditorAction {
        let max = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line >= max {
            return EditorAction::None;
        }
        let line = self.current_line_text();
        let line_end = self.line_start_offset() + line.len();
        // Delete the newline and leading whitespace of next line
        let next = self.buffer.line(self.cursor_line + 1).unwrap_or_default();
        let trimmed = next.trim_start().len();
        let ws = next.len() - trimmed;
        self.buffer.delete(line_end, line_end + 1 + ws);
        // Insert a space if needed
        if !line.is_empty() && !next.trim_start().is_empty() {
            self.buffer.insert(line_end, " ");
        }
        EditorAction::ContentChanged
    }
}

// ── Undo/Redo ──

impl Editor {
    fn do_undo(&mut self) -> EditorAction {
        if self.buffer.undo() {
            self.clamp_cursor();
            EditorAction::ContentChanged
        } else {
            EditorAction::None
        }
    }

    fn do_redo(&mut self) -> EditorAction {
        if self.buffer.redo() {
            self.clamp_cursor();
            EditorAction::ContentChanged
        } else {
            EditorAction::None
        }
    }
}

// ── Selection ──

impl Editor {
    fn start_selection(&mut self, kind: VisualKind) -> EditorAction {
        self.selection = Some(Selection {
            kind,
            anchor_line: self.cursor_line,
            anchor_col: self.cursor_col,
        });
        self.mode = EditorMode::Visual(kind);
        EditorAction::CursorMoved
    }

    fn cancel_selection(&mut self) -> EditorAction {
        self.selection = None;
        self.mode = EditorMode::Normal;
        EditorAction::CursorMoved
    }
}

// ── Clipboard ──

impl Editor {
    fn yank(&mut self) -> EditorAction {
        if let Some(sel) = &self.selection {
            let (sl, sc) = (sel.anchor_line, sel.anchor_col);
            let (el, ec) = (self.cursor_line, self.cursor_col);
            let (s, e) = self.ordered_offsets(sl, sc, el, ec);
            self.register = self.buffer.slice(s, e);
            self.register_linewise = false;
        }
        self.selection = None;
        self.mode = EditorMode::Normal;
        EditorAction::CursorMoved
    }

    fn yank_line(&mut self) -> EditorAction {
        let ls = self.line_start_offset();
        let next = self
            .buffer
            .line_col_to_offset(self.cursor_line + 1, 0)
            .unwrap_or(self.buffer.len());
        self.register = self.buffer.slice(ls, next);
        self.register_linewise = true;
        EditorAction::None
    }

    fn paste(&mut self) -> EditorAction {
        if self.register.is_empty() {
            return EditorAction::None;
        }
        if self.register_linewise {
            let next = self
                .buffer
                .line_col_to_offset(self.cursor_line + 1, 0)
                .unwrap_or(self.buffer.len());
            let text = self.register.clone();
            // If pasting after the last line with no trailing newline,
            // prepend a newline so the paste starts on its own line.
            let at_end = next == self.buffer.len();
            let needs_nl =
                at_end && !self.buffer.is_empty() && !self.buffer.content().ends_with('\n');
            if needs_nl {
                self.buffer.insert(next, "\n");
                let new_next = next + 1;
                self.buffer.insert(new_next, &text);
            } else {
                self.buffer.insert(next, &text);
            }
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            let offset = self.cursor_offset() + 1;
            let offset = offset.min(self.buffer.len());
            let text = self.register.clone();
            self.buffer.insert(offset, &text);
        }
        EditorAction::ContentChanged
    }

    fn paste_before(&mut self) -> EditorAction {
        if self.register.is_empty() {
            return EditorAction::None;
        }
        if self.register_linewise {
            let ls = self.line_start_offset();
            let text = self.register.clone();
            self.buffer.insert(ls, &text);
            self.cursor_col = 0;
        } else {
            let offset = self.cursor_offset();
            let text = self.register.clone();
            self.buffer.insert(offset, &text);
        }
        EditorAction::ContentChanged
    }

    fn ordered_offsets(&self, l1: usize, c1: usize, l2: usize, c2: usize) -> (usize, usize) {
        let a = self.buffer.line_col_to_offset(l1, c1).unwrap_or(0);
        let b = self.buffer.line_col_to_offset(l2, c2).unwrap_or(0);
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }
}

// ── Search ──

impl Editor {
    fn search_forward(&mut self, pattern: String) -> EditorAction {
        self.search_pattern = Some(pattern);
        self.mode = EditorMode::Normal;
        self.search_next()
    }

    fn search_backward(&mut self, pattern: String) -> EditorAction {
        self.search_pattern = Some(pattern);
        self.mode = EditorMode::Normal;
        self.search_prev()
    }

    fn search_next(&mut self) -> EditorAction {
        let pat = match &self.search_pattern {
            Some(p) => p.clone(),
            None => return EditorAction::None,
        };
        let start_line = self.cursor_line;
        let start_col = self.cursor_col + 1;
        let lc = self.buffer.line_count();
        for i in 0..lc {
            let line_idx = (start_line + i) % lc;
            let text = self.buffer.line(line_idx).unwrap_or_default();
            let from = if i == 0 { start_col } else { 0 };
            if let Some(pos) = text.get(from..).and_then(|s| s.find(&pat)) {
                self.cursor_line = line_idx;
                self.cursor_col = from + pos;
                return EditorAction::CursorMoved;
            }
        }
        EditorAction::None
    }

    fn search_prev(&mut self) -> EditorAction {
        let pat = match &self.search_pattern {
            Some(p) => p.clone(),
            None => return EditorAction::None,
        };
        let start_line = self.cursor_line;
        let lc = self.buffer.line_count();
        for i in 0..lc {
            let line_idx = (start_line + lc - i) % lc;
            let text = self.buffer.line(line_idx).unwrap_or_default();
            let limit = if i == 0 { self.cursor_col } else { text.len() };
            if let Some(pos) = text.get(..limit).and_then(|s| s.rfind(&pat)) {
                self.cursor_line = line_idx;
                self.cursor_col = pos;
                return EditorAction::CursorMoved;
            }
        }
        EditorAction::None
    }

    fn clear_search(&mut self) -> EditorAction {
        self.search_pattern = None;
        EditorAction::None
    }
}

// ── Mode switching ──

impl Editor {
    fn enter_insert(&mut self) -> EditorAction {
        self.mode = EditorMode::Insert;
        EditorAction::CursorMoved
    }

    fn enter_insert_after(&mut self) -> EditorAction {
        self.cursor_col = (self.cursor_col + 1).min(self.current_line_len());
        self.mode = EditorMode::Insert;
        EditorAction::CursorMoved
    }

    fn enter_insert_line_start(&mut self) -> EditorAction {
        let line = self.current_line_text();
        let first_non_ws = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
        self.cursor_col = first_non_ws;
        self.mode = EditorMode::Insert;
        EditorAction::CursorMoved
    }

    fn enter_insert_line_end(&mut self) -> EditorAction {
        self.cursor_col = self.current_line_len();
        self.mode = EditorMode::Insert;
        EditorAction::CursorMoved
    }

    fn enter_insert_below(&mut self) -> EditorAction {
        self.newline_below()
    }

    fn enter_insert_above(&mut self) -> EditorAction {
        self.newline_above()
    }

    fn exit_insert(&mut self) -> EditorAction {
        self.mode = EditorMode::Normal;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
        EditorAction::CursorMoved
    }
}

// ── Ex commands ──

impl Editor {
    fn run_ex(&mut self, input: &str) -> EditorAction {
        self.mode = EditorMode::Normal;
        let cmd = ex::parse_ex(input);
        self.execute(cmd)
    }

    fn close_buffer(&mut self) -> EditorAction {
        if self.buffer.is_modified() {
            EditorAction::CloseBlocked
        } else {
            EditorAction::CloseRequested
        }
    }
}

// ── Helpers ──

impl Editor {
    fn current_line_text(&self) -> String {
        self.buffer.line(self.cursor_line).unwrap_or_default()
    }

    fn current_line_len(&self) -> usize {
        self.current_line_text().chars().count()
    }

    fn line_len_at(&self, line: usize) -> usize {
        self.buffer
            .line(line)
            .map(|l| l.chars().count())
            .unwrap_or(0)
    }

    fn line_start_offset(&self) -> usize {
        self.buffer
            .line_col_to_offset(self.cursor_line, 0)
            .unwrap_or(0)
    }

    fn cursor_offset(&self) -> usize {
        self.buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0)
    }

    fn clamp_col(&mut self) {
        let len = self.current_line_len();
        if self.cursor_col > len {
            self.cursor_col = len;
        }
    }

    fn clamp_cursor(&mut self) {
        let max_line = self.buffer.line_count().saturating_sub(1);
        if self.cursor_line > max_line {
            self.cursor_line = max_line;
        }
        self.clamp_col();
    }
}

/// Convert a character column to a byte offset within a line.
fn char_to_byte_col(line: &str, char_col: usize) -> usize {
    line.char_indices()
        .nth(char_col)
        .map(|(i, _)| i)
        .unwrap_or(line.len())
}

#[cfg(test)]
mod tests {
    use super::keymap_vim::VimKeymap;
    use super::*;

    fn editor(text: &str) -> Editor {
        Editor::new(PieceTable::from_str(text), Box::new(VimKeymap::new()))
    }

    #[test]
    fn cursor_movement() {
        let mut e = editor("abc\ndef\nghi");
        e.execute(Command::MoveDown);
        assert_eq!(e.cursor(), (1, 0));
        e.execute(Command::MoveRight);
        assert_eq!(e.cursor(), (1, 1));
        e.execute(Command::MoveUp);
        assert_eq!(e.cursor(), (0, 1));
        e.execute(Command::MoveLeft);
        assert_eq!(e.cursor(), (0, 0));
    }

    #[test]
    fn insert_and_content() {
        let mut e = editor("hello");
        e.mode = EditorMode::Insert;
        e.execute(Command::InsertChar('!'));
        assert_eq!(e.buffer().content(), "!hello");
    }

    #[test]
    fn delete_char_backward() {
        let mut e = editor("hello");
        e.mode = EditorMode::Insert;
        e.cursor_col = 5;
        e.execute(Command::DeleteCharBackward);
        assert_eq!(e.buffer().content(), "hell");
    }

    #[test]
    fn delete_line() {
        let mut e = editor("aaa\nbbb\nccc");
        e.cursor_line = 1;
        e.execute(Command::DeleteLine);
        assert_eq!(e.buffer().content(), "aaa\nccc");
    }

    #[test]
    fn undo_redo_via_execute() {
        let mut e = editor("hello");
        e.mode = EditorMode::Insert;
        e.execute(Command::InsertChar('!'));
        assert_eq!(e.buffer().content(), "!hello");
        e.execute(Command::Undo);
        assert_eq!(e.buffer().content(), "hello");
        e.execute(Command::Redo);
        assert_eq!(e.buffer().content(), "!hello");
    }

    #[test]
    fn goto_line() {
        let mut e = editor("a\nb\nc\nd\ne");
        e.execute(Command::GotoLine(3));
        assert_eq!(e.cursor(), (3, 0));
    }

    #[test]
    fn page_movement() {
        let mut e = editor(
            &(0..100)
                .map(|i| format!("line{i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        e.set_viewport_height(10);
        e.execute(Command::PageDown);
        assert_eq!(e.cursor().0, 10);
        e.execute(Command::HalfPageDown);
        assert_eq!(e.cursor().0, 15);
    }

    #[test]
    fn search_forward_and_next() {
        let mut e = editor("foo bar foo baz foo");
        e.execute(Command::SearchForward("foo".into()));
        // First match after cursor (col 0) → col 8
        assert_eq!(e.cursor().1, 8);
        e.execute(Command::SearchNext);
        assert_eq!(e.cursor().1, 16);
    }

    #[test]
    fn yank_line_and_paste() {
        let mut e = editor("aaa\nbbb\nccc");
        e.cursor_line = 1;
        e.execute(Command::YankLine);
        e.cursor_line = 2;
        e.execute(Command::Paste);
        assert_eq!(e.buffer().content(), "aaa\nbbb\nccc\nbbb\n");
    }

    #[test]
    fn close_blocked_when_modified() {
        let mut e = editor("hello");
        e.mode = EditorMode::Insert;
        e.execute(Command::InsertChar('!'));
        let action = e.execute(Command::CloseBuffer);
        assert_eq!(action, EditorAction::CloseBlocked);
    }

    #[test]
    fn ex_command_write() {
        let mut e = editor("hello");
        let action = e.execute(Command::ExCommand("w".into()));
        assert_eq!(action, EditorAction::SaveRequested);
    }

    #[test]
    fn indent_dedent() {
        let mut e = editor("hello");
        e.execute(Command::Indent);
        assert_eq!(e.buffer().content(), "    hello");
        e.execute(Command::Dedent);
        assert_eq!(e.buffer().content(), "hello");
    }

    #[test]
    fn join_lines() {
        let mut e = editor("hello\n  world");
        e.execute(Command::JoinLines);
        assert_eq!(e.buffer().content(), "hello world");
    }

    #[test]
    fn mode_switching() {
        let mut e = editor("hello");
        e.execute(Command::EnterInsertMode);
        assert_eq!(e.mode(), EditorMode::Insert);
        e.execute(Command::ExitInsertMode);
        assert_eq!(e.mode(), EditorMode::Normal);
    }

    #[test]
    fn enter_insert_after() {
        let mut e = editor("abc");
        e.cursor_col = 1;
        e.execute(Command::EnterInsertAfter);
        assert_eq!(e.cursor().1, 2);
        assert_eq!(e.mode(), EditorMode::Insert);
    }

    #[test]
    fn focus_actions() {
        let mut e = editor("hello");
        let a = e.execute(Command::FocusTree);
        assert_eq!(a, EditorAction::FocusChange(FocusTarget::Tree));
    }
}
