//! PieceTable — core text buffer with O(n) insert/delete and undo.

use super::line_index::LineIndex;
use super::undo::{EditRecord, UndoHistory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    Original,
    Add,
}

#[derive(Clone, Debug)]
pub struct Piece {
    source: Source,
    start: usize,
    len: usize,
}

/// Piece table text buffer.
pub struct PieceTable {
    original: String,
    add_buf: String,
    pieces: Vec<Piece>,
    line_index: LineIndex,
    history: UndoHistory,
    modified: bool,
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
            file_path: None,
        }
    }

    pub fn from_text(content: &str) -> Self {
        let pieces = if content.is_empty() {
            Vec::new()
        } else {
            vec![Piece { source: Source::Original, start: 0, len: content.len() }]
        };
        Self {
            original: content.to_string(),
            add_buf: String::new(),
            pieces,
            line_index: LineIndex::build(content),
            history: UndoHistory::new(),
            modified: false,
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
        let end = self.line_index.line_start(line_num + 1)
            .map(|e| e.saturating_sub(1)) // strip \n
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
        let new_piece = Piece { source: Source::Add, start: add_start, len: text.len() };
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

    pub fn delete_at(
        &mut self,
        line1: usize,
        col1: usize,
        line2: usize,
        col2: usize,
    ) {
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

    // --- Internal ---

    fn save_undo(&mut self) {
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

    fn insert_piece_at(&mut self, offset: usize, new_piece: Piece) {
        if self.pieces.is_empty() {
            self.pieces.push(new_piece);
            return;
        }
        let mut cumulative = 0;
        for i in 0..self.pieces.len() {
            let p = &self.pieces[i];
            if cumulative + p.len > offset || (cumulative + p.len == offset && i == self.pieces.len() - 1) {
                let local = offset - cumulative;
                if local == 0 {
                    self.pieces.insert(i, new_piece);
                } else if local == p.len {
                    self.pieces.insert(i + 1, new_piece);
                } else {
                    // Split piece
                    let left = Piece { source: p.source, start: p.start, len: local };
                    let right = Piece { source: p.source, start: p.start + local, len: p.len - local };
                    self.pieces.splice(i..=i, [left, new_piece, right]);
                }
                return;
            }
            cumulative += p.len;
        }
        // Append at end
        self.pieces.push(new_piece);
    }

    fn delete_range_internal(&mut self, start: usize, end: usize) {
        let mut new_pieces = Vec::new();
        let mut cumulative = 0;
        for p in &self.pieces {
            let p_start = cumulative;
            let p_end = cumulative + p.len;
            cumulative = p_end;

            if p_end <= start || p_start >= end {
                // Entirely outside delete range — keep
                new_pieces.push(p.clone());
            } else {
                // Partially or fully inside delete range
                if p_start < start {
                    // Keep left portion
                    let keep = start - p_start;
                    new_pieces.push(Piece { source: p.source, start: p.start, len: keep });
                }
                if p_end > end {
                    // Keep right portion
                    let skip = end - p_start;
                    new_pieces.push(Piece {
                        source: p.source,
                        start: p.start + skip,
                        len: p.len - skip,
                    });
                }
            }
        }
        self.pieces = new_pieces;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_at_start() {
        let mut pt = PieceTable::from_text("hello");
        pt.insert(0, "say ");
        assert_eq!(pt.content(), "say hello");
    }

    #[test]
    fn insert_at_end() {
        let mut pt = PieceTable::from_text("hello");
        pt.insert(5, " world");
        assert_eq!(pt.content(), "hello world");
    }

    #[test]
    fn insert_in_middle() {
        let mut pt = PieceTable::from_text("helo");
        pt.insert(3, "l");
        assert_eq!(pt.content(), "hello");
    }

    #[test]
    fn delete_range() {
        let mut pt = PieceTable::from_text("hello world");
        pt.delete(5, 11);
        assert_eq!(pt.content(), "hello");
    }

    #[test]
    fn undo_redo() {
        let mut pt = PieceTable::from_text("hello");
        pt.insert(5, " world");
        assert_eq!(pt.content(), "hello world");
        pt.undo();
        assert_eq!(pt.content(), "hello");
        pt.redo();
        assert_eq!(pt.content(), "hello world");
    }

    #[test]
    fn line_count() {
        let pt = PieceTable::from_text("line1\nline2\nline3");
        assert_eq!(pt.line_count(), 3);
    }

    #[test]
    fn get_line() {
        let pt = PieceTable::from_text("line1\nline2\nline3");
        assert_eq!(pt.line(0), Some("line1".to_string()));
        assert_eq!(pt.line(1), Some("line2".to_string()));
        assert_eq!(pt.line(2), Some("line3".to_string()));
    }

    #[test]
    fn empty_buffer() {
        let pt = PieceTable::new();
        assert_eq!(pt.len(), 0);
        assert_eq!(pt.line_count(), 1);
    }
}
