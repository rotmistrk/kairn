//! PieceTable — core text buffer with O(n) insert/delete and undo.

mod ops;

use super::line_index::LineIndex;
use super::undo::{EditRecord, UndoHistory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Source {
    Original,
    Add,
}

#[derive(Clone, Debug)]
pub(super) struct Piece {
    pub source: Source,
    pub start: usize,
    pub len: usize,
}

/// Piece table text buffer.
pub struct PieceTable {
    original: String,
    add_buf: String,
    pub(super) pieces: Vec<Piece>,
    line_index: LineIndex,
    history: UndoHistory,
    modified: bool,
    grouping: bool,
    pub file_path: Option<String>,
}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceTable {
    pub fn new() -> Self {
        Self {
            original: String::new(),
            add_buf: String::new(),
            pieces: Vec::new(),
            line_index: LineIndex::build(""),
            history: UndoHistory::new(),
            modified: false,
            grouping: false,
            file_path: None,
        }
    }

    pub fn from_text(content: &str) -> Self {
        let pieces = if content.is_empty() {
            Vec::new()
        } else {
            vec![Piece {
                source: Source::Original,
                start: 0,
                len: content.len(),
            }]
        };
        Self {
            original: content.to_string(),
            add_buf: String::new(),
            pieces,
            line_index: LineIndex::build(content),
            history: UndoHistory::new(),
            modified: false,
            grouping: false,
            file_path: None,
        }
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut pt = Self::from_text(&content);
        pt.file_path = Some(path.to_string());
        Ok(pt)
    }

    pub fn content(&self) -> String {
        let mut s = String::with_capacity(self.len());
        for p in &self.pieces {
            let buf = match p.source {
                Source::Original => &self.original,
                Source::Add => &self.add_buf,
            };
            s.push_str(&buf[p.start..p.start + p.len]);
        }
        s
    }

    pub fn len(&self) -> usize {
        self.pieces.iter().map(|p| p.len).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn line_count(&self) -> usize {
        self.line_index.line_count()
    }

    pub fn line(&self, line_num: usize) -> Option<String> {
        let content = self.content();
        let start = self.line_index.line_start(line_num)?;
        let end = self
            .line_index
            .line_start(line_num + 1)
            .map(|e| e.saturating_sub(1))
            .unwrap_or(content.len());
        Some(content[start..end].to_string())
    }

    pub fn line_len(&self, line_num: usize) -> usize {
        self.line(line_num).map(|l| l.chars().count()).unwrap_or(0)
    }

    pub fn insert(&mut self, offset: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.save_undo();
        let add_start = self.add_buf.len();
        self.add_buf.push_str(text);
        let new_piece = Piece {
            source: Source::Add,
            start: add_start,
            len: text.len(),
        };
        self.insert_piece_at(offset, new_piece);
        self.rebuild_line_index();
        self.modified = true;
    }

    pub fn delete(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        self.save_undo();
        self.delete_range_internal(start, end);
        self.rebuild_line_index();
        self.modified = true;
    }

    pub fn insert_at(&mut self, line: usize, col: usize, text: &str) {
        if let Some(offset) = self.line_col_to_offset(line, col) {
            self.insert(offset, text);
        }
    }

    pub fn delete_at(&mut self, line1: usize, col1: usize, line2: usize, col2: usize) {
        let start = self.line_col_to_offset(line1, col1);
        let end = self.line_col_to_offset(line2, col2);
        if let (Some(s), Some(e)) = (start, end) {
            self.delete(s, e);
        }
    }

    pub fn line_col_to_offset(&self, line: usize, col: usize) -> Option<usize> {
        let line_start = self.line_index.line_start(line)?;
        let content = self.content();
        let line_content = &content[line_start..];
        let byte_offset: usize = line_content.chars().take(col).map(|c| c.len_utf8()).sum();
        Some(line_start + byte_offset)
    }

    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_index.offset_to_line(offset);
        let line_start = self.line_index.line_start(line).unwrap_or(0);
        let content = self.content();
        let col = content[line_start..offset.min(content.len())].chars().count();
        (line, col)
    }

    pub fn undo(&mut self) -> bool {
        let current = EditRecord {
            pieces: self.pieces.clone(),
            line_starts: self.line_index.line_starts.clone(),
        };
        if let Some(prev) = self.history.undo(current) {
            self.pieces = prev.pieces;
            self.line_index.line_starts = prev.line_starts;
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        let current = EditRecord {
            pieces: self.pieces.clone(),
            line_starts: self.line_index.line_starts.clone(),
        };
        if let Some(next) = self.history.redo(current) {
            self.pieces = next.pieces;
            self.line_index.line_starts = next.line_starts;
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
    pub fn is_dirty(&self) -> bool {
        self.modified
    }

    /// Begin an undo group. Saves one snapshot now; subsequent edits won't push.
    pub fn begin_group(&mut self) {
        if !self.grouping {
            let record = EditRecord {
                pieces: self.pieces.clone(),
                line_starts: self.line_index.line_starts.clone(),
            };
            self.history.push(record);
            self.grouping = true;
        }
    }

    /// End the current undo group.
    pub fn end_group(&mut self) {
        self.grouping = false;
    }

    fn save_undo(&mut self) {
        if self.grouping {
            return;
        }
        let record = EditRecord {
            pieces: self.pieces.clone(),
            line_starts: self.line_index.line_starts.clone(),
        };
        self.history.push(record);
    }

    fn rebuild_line_index(&mut self) {
        let content = self.content();
        self.line_index.rebuild(&content);
    }
}

#[cfg(test)]
mod tests;
