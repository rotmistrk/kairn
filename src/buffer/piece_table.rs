/// Piece table buffer — the core text data structure for kairn.
///
/// Original text is never modified. Inserts append to an add-only buffer.
/// The document is described by an ordered list of pieces referencing
/// either the original or add buffer.
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::buffer::line_index::LineIndex;
use crate::buffer::undo::{EditRecord, PieceSnapshot, UndoHistory};

/// Which buffer a piece references.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    Original,
    Add,
}

/// A contiguous run of text from one source buffer.
#[derive(Clone, Debug)]
struct Piece {
    source: Source,
    start: usize,
    len: usize,
}

/// A text change for LSP incremental sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub new_text: String,
}

/// The piece table buffer.
pub struct PieceTable {
    original: String,
    add_buf: String,
    pieces: Vec<Piece>,
    total_len: usize,
    line_index: LineIndex,
    history: UndoHistory,
    modified: bool,
    file_path: Option<String>,
    pending_changes: Vec<TextChange>,
}

// ── Construction ──

impl PieceTable {
    /// Create an empty buffer.
    pub fn new() -> Self {
        Self {
            original: String::new(),
            add_buf: String::new(),
            pieces: Vec::new(),
            total_len: 0,
            line_index: LineIndex::build(""),
            history: UndoHistory::new(),
            modified: false,
            file_path: None,
            pending_changes: Vec::new(),
        }
    }

    /// Create a buffer from string content.
    pub fn from_str(content: &str) -> Self {
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
            total_len: content.len(),
            line_index: LineIndex::build(content),
            history: UndoHistory::new(),
            modified: false,
            file_path: None,
            pending_changes: Vec::new(),
        }
    }

    /// Load from a file path.
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(Path::new(path))?;
        let mut pt = Self::from_str(&content);
        pt.file_path = Some(path.to_string());
        Ok(pt)
    }
}

impl Default for PieceTable {
    fn default() -> Self {
        Self::new()
    }
}

// ── Read ──

impl PieceTable {
    /// Total byte length of the document.
    pub fn len(&self) -> usize {
        self.total_len
    }

    /// Whether the document is empty.
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    /// Total number of lines (always >= 1).
    pub fn line_count(&self) -> usize {
        self.line_index.line_count()
    }

    /// Get the content of a single line (0-indexed). No trailing newline.
    pub fn line(&self, line_num: usize) -> Option<String> {
        let start = self.line_index.line_start(line_num)?;
        let end = self
            .line_index
            .line_start(line_num + 1)
            .map(|e| e - 1) // exclude the \n
            .unwrap_or(self.total_len);
        Some(self.slice(start, end))
    }

    /// Get a range of lines as a single string.
    pub fn lines_range(&self, start: usize, end: usize) -> String {
        let byte_start = match self.line_index.line_start(start) {
            Some(s) => s,
            None => return String::new(),
        };
        let byte_end = if end >= self.line_count() {
            self.total_len
        } else {
            self.line_index.line_start(end).unwrap_or(self.total_len)
        };
        self.slice(byte_start, byte_end)
    }

    /// Get the full document content as a string.
    pub fn content(&self) -> String {
        let mut out = String::with_capacity(self.total_len);
        for p in &self.pieces {
            let buf = self.source_buf(p.source);
            out.push_str(&buf[p.start..p.start + p.len]);
        }
        out
    }

    /// Get a byte range of the document.
    pub fn slice(&self, byte_start: usize, byte_end: usize) -> String {
        let end = byte_end.min(self.total_len);
        let start = byte_start.min(end);
        if start == end {
            return String::new();
        }
        let mut out = String::with_capacity(end - start);
        let mut offset = 0usize;
        for p in &self.pieces {
            let p_end = offset + p.len;
            if p_end <= start {
                offset = p_end;
                continue;
            }
            if offset >= end {
                break;
            }
            let local_start = start.saturating_sub(offset);
            let local_end = (end - offset).min(p.len);
            let buf = self.source_buf(p.source);
            let abs_start = p.start + local_start;
            let abs_end = p.start + local_end;
            out.push_str(&buf[abs_start..abs_end]);
            offset = p_end;
        }
        out
    }

    /// Convert (line, col) to byte offset. Col is in characters.
    pub fn line_col_to_offset(&self, line: usize, col: usize) -> Option<usize> {
        let line_start = self.line_index.line_start(line)?;
        let line_text = self.line(line)?;
        let byte_col = line_text
            .char_indices()
            .nth(col)
            .map(|(i, _)| i)
            .unwrap_or(line_text.len());
        Some(line_start + byte_col)
    }

    /// Convert byte offset to (line, col). Col is in characters.
    pub fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.line_index.offset_to_line(offset);
        let line_start = self.line_index.line_start(line).unwrap_or(0);
        let byte_col = offset.saturating_sub(line_start);
        let line_text = self.line(line).unwrap_or_default();
        let char_col = line_text
            .get(..byte_col)
            .map(|s| s.chars().count())
            .unwrap_or(0);
        (line, char_col)
    }

    fn source_buf(&self, source: Source) -> &str {
        match source {
            Source::Original => &self.original,
            Source::Add => &self.add_buf,
        }
    }
}

// ── Write ──

impl PieceTable {
    /// Insert text at a byte offset.
    pub fn insert(&mut self, offset: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        self.save_undo_snapshot();
        self.record_insert_change(offset, text);
        self.do_insert(offset, text);
    }

    /// Delete a byte range `[start, end)`.
    pub fn delete(&mut self, start: usize, end: usize) {
        if start >= end || start >= self.total_len {
            return;
        }
        self.save_undo_snapshot();
        self.record_delete_change(start, end);
        self.do_delete(start, end);
    }

    /// Replace a byte range with new text (single undo unit).
    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        self.save_undo_snapshot();
        self.record_delete_change(start, end);
        self.record_insert_change(start, text);
        self.do_delete_raw(start, end);
        self.do_insert(start, text);
    }

    /// Insert text at (line, col).
    pub fn insert_at(&mut self, line: usize, col: usize, text: &str) {
        if let Some(offset) = self.line_col_to_offset(line, col) {
            self.insert(offset, text);
        }
    }

    /// Delete from (line1, col1) to (line2, col2).
    pub fn delete_range(&mut self, line1: usize, col1: usize, line2: usize, col2: usize) {
        let start = self.line_col_to_offset(line1, col1);
        let end = self.line_col_to_offset(line2, col2);
        if let (Some(s), Some(e)) = (start, end) {
            self.delete(s, e);
        }
    }

    fn do_insert(&mut self, offset: usize, text: &str) {
        let add_start = self.add_buf.len();
        self.add_buf.push_str(text);
        let new_piece = Piece {
            source: Source::Add,
            start: add_start,
            len: text.len(),
        };
        let idx = self.find_piece_at(offset);
        self.split_and_insert(idx, offset, new_piece);
        self.total_len += text.len();
        self.line_index.update_insert(offset, text);
        self.modified = true;
    }

    fn do_delete(&mut self, start: usize, end: usize) {
        self.do_delete_raw(start, end);
    }

    fn do_delete_raw(&mut self, start: usize, end: usize) {
        let end = end.min(self.total_len);
        if start >= end {
            return;
        }
        let removed = end - start;
        self.remove_range(start, end);
        self.total_len -= removed;
        self.line_index.update_delete(start, end);
        self.modified = true;
    }

    /// Find which piece index contains `offset`.
    /// Returns `(piece_index, cumulative_offset_at_piece_start)`.
    fn find_piece_at(&self, offset: usize) -> (usize, usize) {
        let mut cum = 0usize;
        for (i, p) in self.pieces.iter().enumerate() {
            if cum + p.len > offset || (cum + p.len == offset && i + 1 == self.pieces.len()) {
                return (i, cum);
            }
            cum += p.len;
        }
        (self.pieces.len(), cum)
    }

    fn split_and_insert(&mut self, (idx, cum): (usize, usize), offset: usize, new_piece: Piece) {
        if idx >= self.pieces.len() {
            self.pieces.push(new_piece);
            return;
        }
        let local = offset - cum;
        let p = &self.pieces[idx];
        if local == 0 {
            self.pieces.insert(idx, new_piece);
        } else if local == p.len {
            self.pieces.insert(idx + 1, new_piece);
        } else {
            let left = Piece {
                source: p.source,
                start: p.start,
                len: local,
            };
            let right = Piece {
                source: p.source,
                start: p.start + local,
                len: p.len - local,
            };
            self.pieces.splice(idx..=idx, [left, new_piece, right]);
        }
    }

    fn remove_range(&mut self, start: usize, end: usize) {
        let mut cum = 0usize;
        let mut first_idx = None;
        let mut last_idx = None;
        let mut first_local = 0;
        let mut last_local = 0;

        for (i, p) in self.pieces.iter().enumerate() {
            let p_end = cum + p.len;
            if first_idx.is_none() && p_end > start {
                first_idx = Some(i);
                first_local = start - cum;
            }
            if p_end >= end {
                last_idx = Some(i);
                last_local = end - cum;
                break;
            }
            cum += p.len;
        }

        let (fi, li) = match (first_idx, last_idx) {
            (Some(f), Some(l)) => (f, l),
            _ => return,
        };

        let mut replacement = Vec::new();
        // Left remainder of first piece
        if first_local > 0 {
            let p = &self.pieces[fi];
            replacement.push(Piece {
                source: p.source,
                start: p.start,
                len: first_local,
            });
        }
        // Right remainder of last piece
        let lp = &self.pieces[li];
        if last_local < lp.len {
            replacement.push(Piece {
                source: lp.source,
                start: lp.start + last_local,
                len: lp.len - last_local,
            });
        }
        self.pieces.splice(fi..=li, replacement);
    }
}

// ── Undo/Redo ──

impl PieceTable {
    /// Undo the last edit. Returns false if nothing to undo.
    pub fn undo(&mut self) -> bool {
        let current = self.make_snapshot();
        match self.history.pop_undo() {
            Some(record) => {
                self.history.push_redo(current);
                self.restore_snapshot(record);
                true
            }
            None => false,
        }
    }

    /// Redo the last undone edit. Returns false if nothing to redo.
    pub fn redo(&mut self) -> bool {
        let current = self.make_snapshot();
        match self.history.pop_redo() {
            Some(record) => {
                self.history.push_undo(current);
                self.restore_snapshot(record);
                true
            }
            None => false,
        }
    }

    /// Start a group — all edits until `end_group` undo as one unit.
    pub fn begin_group(&mut self) {
        let snap = self.make_snapshot();
        self.history.begin_group(snap);
    }

    /// End the current group.
    pub fn end_group(&mut self) {
        self.history.end_group();
    }

    /// Number of undoable operations.
    pub fn undo_count(&self) -> usize {
        self.history.undo_count()
    }

    /// Number of redoable operations.
    pub fn redo_count(&self) -> usize {
        self.history.redo_count()
    }

    fn save_undo_snapshot(&mut self) {
        let snap = self.make_snapshot();
        self.history.push(snap);
    }

    fn make_snapshot(&self) -> EditRecord {
        EditRecord {
            pieces: self
                .pieces
                .iter()
                .map(|p| PieceSnapshot {
                    source_is_add: p.source == Source::Add,
                    start: p.start,
                    len: p.len,
                })
                .collect(),
            line_starts: self.line_index.snapshot(),
            total_len: self.total_len,
        }
    }

    fn restore_snapshot(&mut self, record: EditRecord) {
        self.pieces = record
            .pieces
            .into_iter()
            .map(|ps| Piece {
                source: if ps.source_is_add {
                    Source::Add
                } else {
                    Source::Original
                },
                start: ps.start,
                len: ps.len,
            })
            .collect();
        self.line_index.restore(record.line_starts);
        self.total_len = record.total_len;
        self.modified = true;
    }
}

// ── State ──

impl PieceTable {
    /// Whether the buffer has been modified since last save/load.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the buffer as unmodified (called after save).
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// The file path, if any.
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Set the file path.
    pub fn set_file_path(&mut self, path: &str) {
        self.file_path = Some(path.to_string());
    }
}

// ── LSP sync ──

impl PieceTable {
    /// Get incremental changes since last call. For LSP `didChange`.
    pub fn take_changes(&mut self) -> Vec<TextChange> {
        std::mem::take(&mut self.pending_changes)
    }

    fn record_insert_change(&mut self, offset: usize, text: &str) {
        let (line, col) = self.offset_to_line_col(offset);
        self.pending_changes.push(TextChange {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col,
            new_text: text.to_string(),
        });
    }

    fn record_delete_change(&mut self, start: usize, end: usize) {
        let (sl, sc) = self.offset_to_line_col(start);
        let (el, ec) = self.offset_to_line_col(end);
        self.pending_changes.push(TextChange {
            start_line: sl,
            start_col: sc,
            end_line: el,
            end_col: ec,
            new_text: String::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_at_start() {
        let mut pt = PieceTable::from_str("hello");
        pt.insert(0, "say ");
        assert_eq!(pt.content(), "say hello");
    }

    #[test]
    fn insert_at_end() {
        let mut pt = PieceTable::from_str("hello");
        pt.insert(5, " world");
        assert_eq!(pt.content(), "hello world");
    }

    #[test]
    fn insert_in_middle() {
        let mut pt = PieceTable::from_str("helo");
        pt.insert(3, "l");
        assert_eq!(pt.content(), "hello");
    }

    #[test]
    fn delete_range() {
        let mut pt = PieceTable::from_str("hello world");
        pt.delete(5, 11);
        assert_eq!(pt.content(), "hello");
    }

    #[test]
    fn replace_range() {
        let mut pt = PieceTable::from_str("hello world");
        pt.replace(6, 11, "rust");
        assert_eq!(pt.content(), "hello rust");
    }

    #[test]
    fn undo_redo() {
        let mut pt = PieceTable::from_str("hello");
        pt.insert(5, " world");
        assert_eq!(pt.content(), "hello world");
        pt.undo();
        assert_eq!(pt.content(), "hello");
        pt.redo();
        assert_eq!(pt.content(), "hello world");
    }

    #[test]
    fn line_count() {
        let pt = PieceTable::from_str("line1\nline2\nline3");
        assert_eq!(pt.line_count(), 3);
    }

    #[test]
    fn get_line() {
        let pt = PieceTable::from_str("line1\nline2\nline3");
        assert_eq!(pt.line(0), Some("line1".to_string()));
        assert_eq!(pt.line(1), Some("line2".to_string()));
        assert_eq!(pt.line(2), Some("line3".to_string()));
    }

    #[test]
    fn line_col_to_offset_test() {
        let pt = PieceTable::from_str("abc\ndef\nghi");
        assert_eq!(pt.line_col_to_offset(0, 0), Some(0));
        assert_eq!(pt.line_col_to_offset(1, 0), Some(4));
        assert_eq!(pt.line_col_to_offset(1, 2), Some(6));
        assert_eq!(pt.line_col_to_offset(2, 0), Some(8));
    }

    #[test]
    fn offset_to_line_col_test() {
        let pt = PieceTable::from_str("abc\ndef\nghi");
        assert_eq!(pt.offset_to_line_col(0), (0, 0));
        assert_eq!(pt.offset_to_line_col(4), (1, 0));
        assert_eq!(pt.offset_to_line_col(6), (1, 2));
    }

    #[test]
    fn grouped_undo() {
        let mut pt = PieceTable::from_str("hello world");
        pt.begin_group();
        pt.delete(5, 11);
        pt.insert(5, " rust");
        pt.end_group();
        assert_eq!(pt.content(), "hello rust");
        pt.undo();
        assert_eq!(pt.content(), "hello world");
    }

    #[test]
    fn modified_flag() {
        let mut pt = PieceTable::from_str("hello");
        assert!(!pt.is_modified());
        pt.insert(5, "!");
        assert!(pt.is_modified());
        pt.mark_saved();
        assert!(!pt.is_modified());
    }

    #[test]
    fn lsp_changes() {
        let mut pt = PieceTable::from_str("hello\nworld");
        pt.insert(5, " beautiful");
        let changes = pt.take_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].start_line, 0);
        assert_eq!(changes[0].start_col, 5);
        assert_eq!(changes[0].new_text, " beautiful");
    }

    #[test]
    fn empty_buffer() {
        let pt = PieceTable::new();
        assert_eq!(pt.len(), 0);
        assert_eq!(pt.line_count(), 1);
        assert_eq!(pt.line(0), Some(String::new()));
    }

    #[test]
    fn large_file_line_access() {
        let content: String = (0..10000).map(|i| format!("line {i}\n")).collect();
        let pt = PieceTable::from_str(&content);
        assert_eq!(pt.line_count(), 10001);
        assert_eq!(pt.line(5000), Some("line 5000".to_string()));
    }

    #[test]
    fn slice_across_pieces() {
        let mut pt = PieceTable::from_str("abcdef");
        pt.insert(3, "XY");
        // content is now "abcXYdef"
        assert_eq!(pt.slice(2, 6), "cXYd");
    }

    #[test]
    fn delete_at_and_insert_at() {
        let mut pt = PieceTable::from_str("abc\ndef\nghi");
        pt.delete_range(1, 0, 1, 3);
        // deleted "def" → "abc\n\nghi"
        assert_eq!(pt.content(), "abc\n\nghi");
        pt.insert_at(1, 0, "xyz");
        assert_eq!(pt.content(), "abc\nxyz\nghi");
    }

    #[test]
    fn file_path_operations() {
        let mut pt = PieceTable::new();
        assert!(pt.file_path().is_none());
        pt.set_file_path("/tmp/test.txt");
        assert_eq!(pt.file_path(), Some("/tmp/test.txt"));
    }

    #[test]
    fn lines_range_test() {
        let pt = PieceTable::from_str("aaa\nbbb\nccc\nddd");
        let r = pt.lines_range(1, 3);
        assert_eq!(r, "bbb\nccc\n");
    }

    #[test]
    fn multiple_undo_redo() {
        let mut pt = PieceTable::from_str("a");
        pt.insert(1, "b");
        pt.insert(2, "c");
        assert_eq!(pt.content(), "abc");
        pt.undo();
        assert_eq!(pt.content(), "ab");
        pt.undo();
        assert_eq!(pt.content(), "a");
        pt.redo();
        assert_eq!(pt.content(), "ab");
        pt.redo();
        assert_eq!(pt.content(), "abc");
    }
}
