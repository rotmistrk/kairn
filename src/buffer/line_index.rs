//! LineIndex — maps byte offsets to line numbers and vice versa.

/// Tracks line start byte offsets for fast line↔offset conversion.
pub struct LineIndex {
    /// Byte offset of the start of each line. line_starts[0] is always 0.
    pub(crate) line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn build(content: &str) -> Self {
        let mut starts = vec![0];
        for (i, b) in content.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { line_starts: starts }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_start(&self, line: usize) -> Option<usize> {
        self.line_starts.get(line).copied()
    }

    pub fn offset_to_line(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line.saturating_sub(1),
        }
    }

    pub fn rebuild(&mut self, content: &str) {
        *self = Self::build(content);
    }
}
