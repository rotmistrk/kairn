//! Ephemeral highlight range — a highlighted region in buffer coordinates.

/// A highlighted range in buffer coordinates.
#[derive(Clone, Copy)]
pub struct EphemeralRange {
    pub(crate) start_line: usize,
    pub(crate) start_col: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
}

impl EphemeralRange {
    /// Create a range spanning an entire line.
    pub fn full_line(line: usize) -> Self {
        Self {
            start_line: line,
            start_col: 0,
            end_line: line,
            end_col: usize::MAX,
        }
    }

    /// Create a range at a specific position (single char width for cursor indicator).
    pub fn point(line: usize, col: usize) -> Self {
        Self {
            start_line: line,
            start_col: col,
            end_line: line,
            end_col: col + 1,
        }
    }

    /// Does this range cover the given line?
    pub fn covers_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Column range on a given line (None if line not covered).
    pub fn col_range_on_line(&self, line: usize) -> Option<(usize, usize)> {
        if !self.covers_line(line) {
            return None;
        }
        let start = if line == self.start_line {
            self.start_col
        } else {
            0
        };
        let end = if line == self.end_line {
            self.end_col
        } else {
            usize::MAX
        };
        Some((start, end))
    }
}
