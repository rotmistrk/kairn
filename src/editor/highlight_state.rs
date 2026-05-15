//! Search highlight state for the editor.

/// Tracks search matches and the current (primary) match index.
pub struct HighlightState {
    pub pattern: String,
    /// (start_offset, end_offset) pairs in the buffer.
    pub matches: Vec<(usize, usize)>,
    /// Index into `matches` for the primary highlight.
    pub current: usize,
}

impl HighlightState {
    /// Build highlight state by finding all occurrences of `pattern` in `content`.
    pub fn build(pattern: &str, content: &str, cursor_offset: usize) -> Option<Self> {
        if pattern.is_empty() {
            return None;
        }
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = content[start..].find(pattern) {
            let abs = start + pos;
            matches.push((abs, abs + pattern.len()));
            start = abs + 1;
        }
        if matches.is_empty() {
            return None;
        }
        // Find the match closest to (at or after) cursor_offset
        let current = matches.iter().position(|(s, _)| *s >= cursor_offset).unwrap_or(0);
        Some(Self {
            pattern: pattern.to_string(),
            matches,
            current,
        })
    }

    /// Check if a byte offset falls within any match. Returns Some(true) for current, Some(false) for other.
    pub fn match_at(&self, offset: usize) -> Option<bool> {
        for (i, (s, e)) in self.matches.iter().enumerate() {
            if offset >= *s && offset < *e {
                return Some(i == self.current);
            }
        }
        None
    }
}
