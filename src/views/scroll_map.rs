//! ScrollMap — hunk-aligned line mapping between two diff panes.
//!
//! Built from the diff output, maps lines in pane 0 (base) to pane 1 (current)
//! and vice versa, accounting for insertions and deletions.

/// Bidirectional line mapping for synchronized scrolling.
pub struct ScrollMap {
    /// For each line in pane 0, the corresponding line in pane 1.
    pub left_to_right: Vec<usize>,
    /// For each line in pane 1, the corresponding line in pane 0.
    pub right_to_left: Vec<usize>,
}

impl ScrollMap {
    /// Translate a scroll position from one pane to the other.
    /// `from_pane`: 0 = left (base), 1 = right (current).
    pub fn translate(&self, from_pane: usize, line: usize) -> usize {
        let map = if from_pane == 0 {
            &self.left_to_right
        } else {
            &self.right_to_left
        };
        if line >= map.len() {
            map.last().copied().unwrap_or(0)
        } else {
            map[line]
        }
    }

    /// Build a scroll map from diff changes.
    /// Uses `similar` to diff base vs current and build alignment.
    pub fn from_diff(base: &str, current: &str) -> Self {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(base, current);
        let mut left_to_right = Vec::new();
        let mut right_to_left = Vec::new();
        let mut base_line: usize = 0;
        let mut curr_line: usize = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Equal => {
                    left_to_right.push(curr_line);
                    right_to_left.push(base_line);
                    base_line += 1;
                    curr_line += 1;
                }
                ChangeTag::Delete => {
                    // Line exists in base but not current — map to current position
                    left_to_right.push(curr_line);
                    base_line += 1;
                }
                ChangeTag::Insert => {
                    // Line exists in current but not base — map to base position
                    right_to_left.push(base_line);
                    curr_line += 1;
                }
            }
        }

        Self {
            left_to_right,
            right_to_left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_files() {
        let text = "a\nb\nc\n";
        let map = ScrollMap::from_diff(text, text);
        assert_eq!(map.translate(0, 0), 0);
        assert_eq!(map.translate(0, 2), 2);
        assert_eq!(map.translate(1, 1), 1);
    }

    #[test]
    fn insertion_shifts_lines() {
        let base = "a\nb\nc\n";
        let current = "a\nNEW\nb\nc\n";
        let map = ScrollMap::from_diff(base, current);
        // base line 0 (a) → current line 0 (a)
        assert_eq!(map.translate(0, 0), 0);
        // base line 1 (b) → current line 2 (b)
        assert_eq!(map.translate(0, 1), 2);
        // current line 0 (a) → base line 0 (a)
        assert_eq!(map.translate(1, 0), 0);
        // current line 1 (NEW) → base line 1 (stays at insertion point)
        assert_eq!(map.translate(1, 1), 1);
        // current line 2 (b) → base line 1 (b)
        assert_eq!(map.translate(1, 2), 1);
    }

    #[test]
    fn deletion_compresses_lines() {
        let base = "a\nDEL\nb\nc\n";
        let current = "a\nb\nc\n";
        let map = ScrollMap::from_diff(base, current);
        // base line 0 (a) → current line 0 (a)
        assert_eq!(map.translate(0, 0), 0);
        // base line 1 (DEL) → current line 1 (maps to where it would be)
        assert_eq!(map.translate(0, 1), 1);
        // base line 2 (b) → current line 1 (b)
        assert_eq!(map.translate(0, 2), 1);
    }
}
