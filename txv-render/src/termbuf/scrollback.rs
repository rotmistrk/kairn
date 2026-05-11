//! Scrollback ring buffer — stores lines pushed off the top of the terminal.

use std::collections::VecDeque;

use super::TCell;

/// A capped ring buffer of terminal lines for scrollback history.
pub(super) struct Scrollback {
    lines: VecDeque<Vec<TCell>>,
    limit: usize,
}

impl Scrollback {
    /// Create a new scrollback buffer with the given line capacity.
    pub fn new(limit: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            limit,
        }
    }

    /// Push a line into the scrollback. Drops oldest if at capacity.
    pub fn push(&mut self, line: Vec<TCell>) {
        if self.limit == 0 {
            return;
        }
        if self.lines.len() >= self.limit {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Number of lines currently stored.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Get a line by offset from the bottom (0 = most recent).
    pub fn line_from_bottom(&self, offset: usize) -> Option<&Vec<TCell>> {
        if offset >= self.lines.len() {
            return None;
        }
        let idx = self.lines.len() - 1 - offset;
        self.lines.get(idx)
    }

    /// Get a line by index from the top (0 = oldest).
    #[cfg(test)]
    pub fn line_from_top(&self, idx: usize) -> Option<&Vec<TCell>> {
        self.lines.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_line(ch: char, width: usize) -> Vec<TCell> {
        vec![TCell { ch, ..TCell::default() }; width]
    }

    #[test]
    fn push_and_retrieve() {
        let mut sb = Scrollback::new(10);
        sb.push(make_line('A', 5));
        sb.push(make_line('B', 5));
        assert_eq!(sb.len(), 2);
        assert_eq!(sb.line_from_bottom(0).map(|l| l[0].ch), Some('B'));
        assert_eq!(sb.line_from_bottom(1).map(|l| l[0].ch), Some('A'));
    }

    #[test]
    fn respects_limit() {
        let mut sb = Scrollback::new(3);
        for ch in ['A', 'B', 'C', 'D', 'E'] {
            sb.push(make_line(ch, 5));
        }
        assert_eq!(sb.len(), 3);
        assert_eq!(sb.line_from_top(0).map(|l| l[0].ch), Some('C'));
        assert_eq!(sb.line_from_top(2).map(|l| l[0].ch), Some('E'));
    }

    #[test]
    fn zero_limit_stores_nothing() {
        let mut sb = Scrollback::new(0);
        sb.push(make_line('X', 5));
        assert_eq!(sb.len(), 0);
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let mut sb = Scrollback::new(10);
        sb.push(make_line('A', 5));
        assert!(sb.line_from_bottom(1).is_none());
        assert!(sb.line_from_top(1).is_none());
    }
}
