//! Text utilities: display width, truncation, wrapping, byte↔column mapping.

use unicode_width::UnicodeWidthChar;

/// Display width of a string in terminal columns.
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// Truncate a string to fit within `max_width` terminal columns.
/// Returns a new string that is at most `max_width` columns wide.
pub fn truncate(s: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut col = 0;
    for c in s.chars() {
        let w = char_width(c);
        if col + w > max_width {
            break;
        }
        out.push(c);
        col += w;
    }
    out
}

/// Wrap text to lines of at most `max_width` columns.
/// Breaks at word boundaries when possible, otherwise mid-character.
pub fn wrap(s: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![s.to_string()];
    }
    let mut lines = Vec::new();
    for line in s.split('\n') {
        wrap_line(line, max_width, &mut lines);
    }
    lines
}

fn wrap_line(line: &str, max_width: usize, out: &mut Vec<String>) {
    if display_width(line) <= max_width {
        out.push(line.to_string());
        return;
    }

    let mut current = String::new();
    let mut col = 0;

    for word in WordIter::new(line) {
        let ww = display_width(word);
        if col == 0 {
            // First word on line — must take it even if too long
            if ww <= max_width {
                current.push_str(word);
                col = ww;
            } else {
                // Force-break long word
                for c in word.chars() {
                    let cw = char_width(c);
                    if col + cw > max_width {
                        out.push(std::mem::take(&mut current));
                        col = 0;
                    }
                    current.push(c);
                    col += cw;
                }
            }
        } else if col + ww <= max_width {
            current.push_str(word);
            col += ww;
        } else {
            out.push(std::mem::take(&mut current));
            col = 0;
            // Skip leading space on new line
            let trimmed = word.trim_start();
            let tw = display_width(trimmed);
            if tw <= max_width {
                current.push_str(trimmed);
                col = tw;
            } else {
                for c in trimmed.chars() {
                    let cw = char_width(c);
                    if col + cw > max_width {
                        out.push(std::mem::take(&mut current));
                        col = 0;
                    }
                    current.push(c);
                    col += cw;
                }
            }
        }
    }
    if !current.is_empty() || col == 0 {
        out.push(current);
    }
}

/// Iterator that yields words with their trailing/leading whitespace attached.
struct WordIter<'a> {
    rest: &'a str,
}

impl<'a> WordIter<'a> {
    fn new(s: &'a str) -> Self {
        Self { rest: s }
    }
}

impl<'a> Iterator for WordIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        // Find end of whitespace + word chunk
        let mut end = 0;
        let mut in_space = self.rest.starts_with(char::is_whitespace);
        for (i, c) in self.rest.char_indices() {
            if i > 0 && c.is_whitespace() != in_space {
                if !in_space {
                    // We were in a word, hit space — emit word
                    end = i;
                    break;
                }
                in_space = false;
            }
            end = i + c.len_utf8();
        }
        let chunk = &self.rest[..end];
        self.rest = &self.rest[end..];
        Some(chunk)
    }
}

/// Convert a byte offset in `s` to a column (display) position.
pub fn byte_to_col(s: &str, byte_offset: usize) -> usize {
    s[..byte_offset.min(s.len())].chars().map(char_width).sum()
}

/// Convert a column (display) position to the byte offset in `s`.
/// Returns the byte offset of the character at or past `col`.
pub fn col_to_byte(s: &str, col: usize) -> usize {
    let mut current_col = 0;
    for (i, c) in s.char_indices() {
        if current_col >= col {
            return i;
        }
        current_col += char_width(c);
    }
    s.len()
}

/// Width of a single character in terminal columns.
fn char_width(c: char) -> usize {
    if c == '\t' {
        return 1; // Treat tab as 1 for raw width; caller handles tab stops
    }
    c.width().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(display_width("hello"), 5);
    }

    #[test]
    fn cjk_width() {
        // CJK characters are 2 columns wide
        assert_eq!(display_width("日本"), 4);
    }

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate("hello world", 5), "hello");
    }

    #[test]
    fn truncate_cjk_boundary() {
        // "日本語" = 6 cols; truncate to 5 should drop last char
        assert_eq!(truncate("日本語", 5), "日本");
    }

    #[test]
    fn truncate_no_op() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn wrap_short_line() {
        assert_eq!(wrap("hello", 10), vec!["hello"]);
    }

    #[test]
    fn wrap_breaks_at_space() {
        let result = wrap("hello world", 7);
        assert_eq!(result, vec!["hello", "world"]);
    }

    #[test]
    fn wrap_long_word() {
        let result = wrap("abcdefgh", 4);
        assert_eq!(result, vec!["abcd", "efgh"]);
    }

    #[test]
    fn byte_to_col_ascii() {
        assert_eq!(byte_to_col("hello", 3), 3);
    }

    #[test]
    fn byte_to_col_cjk() {
        // "日本" — '日' is 3 bytes, 2 cols
        assert_eq!(byte_to_col("日本", 3), 2);
    }

    #[test]
    fn col_to_byte_ascii() {
        assert_eq!(col_to_byte("hello", 3), 3);
    }

    #[test]
    fn col_to_byte_cjk() {
        // col 2 → byte 3 (start of second char)
        assert_eq!(col_to_byte("日本", 2), 3);
    }

    #[test]
    fn col_to_byte_past_end() {
        assert_eq!(col_to_byte("hi", 10), 2);
    }
}
