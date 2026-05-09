/// Fast line ↔ byte-offset mapping for the piece table buffer.
///
/// Maintains a sorted `Vec<usize>` of line-start byte offsets.
/// `line_starts[0]` is always `0`. Lookups use binary search.
#[derive(Clone, Debug)]
pub struct LineIndex {
    /// Byte offset of the start of each line. `line_starts[0]` is always 0.
    line_starts: Vec<usize>,
}

impl LineIndex {
    /// Build a line index from full document content.
    pub fn build(content: &str) -> Self {
        let mut line_starts = vec![0usize];
        for (i, b) in content.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Self { line_starts }
    }

    /// Incrementally update after an insert at `offset` of `text`.
    pub fn update_insert(&mut self, offset: usize, text: &str) {
        let line = self.offset_to_line(offset);
        let mut new_starts = Vec::new();
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                new_starts.push(offset + i + 1);
            }
        }
        let text_len = text.len();
        // Shift all line starts after the insertion point
        let shift_from = line + 1;
        for ls in &mut self.line_starts[shift_from..] {
            *ls += text_len;
        }
        // Insert new line starts at the correct position
        if !new_starts.is_empty() {
            let insert_pos = shift_from;
            self.line_starts.splice(insert_pos..insert_pos, new_starts);
        }
    }

    /// Incrementally update after a delete of byte range `[start, end)`.
    pub fn update_delete(&mut self, start: usize, end: usize) {
        let removed = end - start;
        if removed == 0 {
            return;
        }
        // Remove line starts within the deleted range (start, end].
        // A line start at `end` was caused by a \n inside the range.
        self.line_starts
            .retain(|&ls| ls == 0 || ls <= start || ls > end);
        // Shift line starts after the deleted range
        for ls in &mut self.line_starts {
            if *ls > end {
                *ls -= removed;
            }
        }
    }

    /// Total number of lines (always >= 1).
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Byte offset of the start of `line` (0-indexed).
    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    /// Which line contains the given byte offset.
    pub fn offset_to_line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(pos) => pos.saturating_sub(1),
        }
    }

    /// Get a snapshot of line_starts for undo.
    pub fn snapshot(&self) -> Vec<usize> {
        self.line_starts.clone()
    }

    /// Restore from a snapshot.
    pub fn restore(&mut self, snapshot: Vec<usize>) {
        self.line_starts = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content() {
        let idx = LineIndex::build("");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.line_start(0), Some(0));
        assert_eq!(idx.offset_to_line(0), 0);
    }

    #[test]
    fn single_line() {
        let idx = LineIndex::build("hello");
        assert_eq!(idx.line_count(), 1);
        assert_eq!(idx.offset_to_line(3), 0);
    }

    #[test]
    fn multi_line() {
        let idx = LineIndex::build("abc\ndef\nghi");
        assert_eq!(idx.line_count(), 3);
        assert_eq!(idx.line_start(0), Some(0));
        assert_eq!(idx.line_start(1), Some(4));
        assert_eq!(idx.line_start(2), Some(8));
        assert_eq!(idx.offset_to_line(0), 0);
        assert_eq!(idx.offset_to_line(3), 0);
        assert_eq!(idx.offset_to_line(4), 1);
        assert_eq!(idx.offset_to_line(8), 2);
    }

    #[test]
    fn trailing_newline() {
        let idx = LineIndex::build("a\nb\n");
        assert_eq!(idx.line_count(), 3);
        assert_eq!(idx.line_start(2), Some(4));
    }

    #[test]
    fn insert_newline() {
        let mut idx = LineIndex::build("helloworld");
        idx.update_insert(5, "\n");
        assert_eq!(idx.line_count(), 2);
        assert_eq!(idx.line_start(1), Some(6));
    }

    #[test]
    fn delete_newline() {
        let mut idx = LineIndex::build("abc\ndef");
        idx.update_delete(3, 4);
        assert_eq!(idx.line_count(), 1);
    }
}
