// Text utilities: display_width, truncate, wrap, byte↔col.

use unicode_width::UnicodeWidthChar;

/// Compute display width of a string (handles wide chars).
/// Control characters have width 0. Tab is not expanded.
pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// Truncate a string to fit within `max_width` display columns.
/// Appends '…' if truncated (the ellipsis counts toward max_width).
pub fn truncate(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut col = 0;
    let mut last_byte = 0;
    let mut needs_truncation = false;
    for (i, c) in s.char_indices() {
        let w = c.width().unwrap_or(0);
        if col + w > max_width {
            needs_truncation = true;
            last_byte = i;
            break;
        }
        col += w;
        last_byte = i + c.len_utf8();
    }
    if !needs_truncation && col <= max_width {
        return s.to_string();
    }
    // Need to fit '…' (width 1) within max_width
    let mut col2 = 0;
    let mut trunc_byte = 0;
    for (i, c) in s.char_indices() {
        if i >= last_byte {
            break;
        }
        let w = c.width().unwrap_or(0);
        if col2 + w + 1 > max_width {
            trunc_byte = i;
            break;
        }
        col2 += w;
        trunc_byte = i + c.len_utf8();
    }
    let mut result = s[..trunc_byte].to_string();
    result.push('…');
    result
}

/// Wrap text to fit within `max_width` display columns.
/// Returns a Vec of lines. Does not break words unless a single
/// word exceeds max_width.
pub fn wrap(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_width = 0;

    for word in WordIter::new(s) {
        let w = display_width(word);
        if line_width > 0 && line_width + w > max_width {
            lines.push(line);
            line = String::new();
            line_width = 0;
        }
        // If a single word exceeds max_width, break it by chars
        if w > max_width && line_width == 0 {
            for c in word.chars() {
                let cw = c.width().unwrap_or(0);
                if line_width + cw > max_width {
                    lines.push(line);
                    line = String::new();
                    line_width = 0;
                }
                line.push(c);
                line_width += cw;
            }
        } else {
            line.push_str(word);
            line_width += w;
        }
    }
    lines.push(line);
    lines
}

/// Compute display column for a byte offset in a string.
pub fn byte_to_col(s: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(s.len());
    let prefix = &s[..clamped];
    display_width(prefix)
}

/// Compute byte offset for a display column in a string.
/// If col falls in the middle of a wide char, returns the byte
/// offset of that wide char.
pub fn col_to_byte(s: &str, col: usize) -> usize {
    let mut current_col = 0;
    for (i, c) in s.char_indices() {
        if current_col >= col {
            return i;
        }
        current_col += c.width().unwrap_or(0);
    }
    s.len()
}

/// Iterator that yields words with their trailing whitespace attached.
struct WordIter<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> WordIter<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, pos: 0 }
    }
}

impl<'a> Iterator for WordIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.s.len() {
            return None;
        }
        let start = self.pos;
        let rest = &self.s[start..];
        // Find end of non-space chars
        let word_end = rest.find(' ').unwrap_or(rest.len());
        // Include trailing space
        let chunk_end = if word_end < rest.len() {
            word_end + 1
        } else {
            word_end
        };
        self.pos = start + chunk_end;
        Some(&self.s[start..start + chunk_end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn display_width_wide() {
        assert_eq!(display_width("漢字"), 4);
    }

    #[test]
    fn display_width_mixed() {
        assert_eq!(display_width("a漢b"), 4);
    }

    #[test]
    fn display_width_empty() {
        assert_eq!(display_width(""), 0);
    }

    #[test]
    fn truncate_no_truncation() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_fit() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_with_ellipsis() {
        assert_eq!(truncate("hello world", 6), "hello…");
    }

    #[test]
    fn truncate_wide_char_boundary() {
        // "a漢b" = width 4. Truncate to 3: "a" + "…" = width 2
        // Can't fit 漢 (width 2) + … (width 1) = 3, so "a漢" fits in 3? No, a(1)+漢(2)=3, no room for …
        // Actually: a(1)+漢(2)=3 which equals max_width, but we need to check if full string fits
        // Full string "a漢b" = width 4 > 3, so truncate.
        let result = truncate("a漢b", 3);
        assert_eq!(display_width(&result) <= 3, true);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn truncate_zero_width() {
        assert_eq!(truncate("hello", 0), "");
    }

    #[test]
    fn wrap_simple() {
        let lines = wrap("hello world", 6);
        assert_eq!(lines, vec!["hello ", "world"]);
    }

    #[test]
    fn wrap_no_wrap_needed() {
        let lines = wrap("hello", 10);
        assert_eq!(lines, vec!["hello"]);
    }

    #[test]
    fn wrap_long_word() {
        let lines = wrap("abcdef", 3);
        assert_eq!(lines, vec!["abc", "def"]);
    }

    #[test]
    fn wrap_wide_chars() {
        // "漢字漢字" = width 8, wrap at 5
        let lines = wrap("漢字漢字", 5);
        assert_eq!(lines.len(), 2);
        assert!(display_width(&lines[0]) <= 5);
    }

    #[test]
    fn byte_to_col_ascii() {
        assert_eq!(byte_to_col("hello", 2), 2);
    }

    #[test]
    fn byte_to_col_wide() {
        // "a漢b": a=1byte, 漢=3bytes, b=1byte
        assert_eq!(byte_to_col("a漢b", 0), 0);
        assert_eq!(byte_to_col("a漢b", 1), 1); // after 'a'
        assert_eq!(byte_to_col("a漢b", 4), 3); // after '漢'
    }

    #[test]
    fn col_to_byte_ascii() {
        assert_eq!(col_to_byte("hello", 2), 2);
    }

    #[test]
    fn col_to_byte_wide() {
        // "a漢b": col 0=a(byte 0), col 1=漢(byte 1), col 3=b(byte 4)
        assert_eq!(col_to_byte("a漢b", 0), 0);
        assert_eq!(col_to_byte("a漢b", 1), 1);
        assert_eq!(col_to_byte("a漢b", 3), 4);
    }

    #[test]
    fn col_to_byte_past_end() {
        assert_eq!(col_to_byte("hi", 10), 2);
    }

    #[test]
    fn byte_to_col_past_end() {
        assert_eq!(byte_to_col("hi", 100), 2);
    }
}
