//! Motions — word, line, paragraph movement over PieceTable buffer.

use crate::buffer::PieceTable;

/// Move cursor to next word start. Returns (line, col).
pub fn word_forward(buf: &PieceTable, line: usize, col: usize) -> (usize, usize) {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    let mut c = col;
    // Skip current word chars
    while c < chars.len() && is_word_char(chars[c]) {
        c += 1;
    }
    // Skip punctuation
    while c < chars.len() && !chars[c].is_whitespace() && !is_word_char(chars[c]) {
        c += 1;
    }
    // Skip whitespace
    while c < chars.len() && chars[c].is_whitespace() {
        c += 1;
    }
    if c >= chars.len() && line + 1 < buf.line_count() {
        // Move to start of next line
        let next = buf.line(line + 1).unwrap_or_default();
        let indent = next.chars().take_while(|ch| ch.is_whitespace()).count();
        return (line + 1, indent.min(next.chars().count().saturating_sub(1)));
    }
    (line, c.min(chars.len().saturating_sub(1)))
}

/// Move cursor to previous word start. Returns (line, col).
pub fn word_backward(buf: &PieceTable, line: usize, col: usize) -> (usize, usize) {
    if col == 0 {
        if line > 0 {
            let prev_len = buf.line_len(line - 1);
            return (line - 1, prev_len.saturating_sub(1));
        }
        return (0, 0);
    }
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return (line, 0);
    }
    let mut c = col.saturating_sub(1).min(chars.len() - 1);
    // Skip whitespace backward
    while c > 0 && chars[c].is_whitespace() {
        c -= 1;
    }
    // Skip word chars backward
    if is_word_char(chars[c]) {
        while c > 0 && is_word_char(chars[c - 1]) {
            c -= 1;
        }
    } else {
        while c > 0 && !chars[c - 1].is_whitespace() && !is_word_char(chars[c - 1]) {
            c -= 1;
        }
    }
    (line, c)
}

/// Move cursor to end of current/next word. Returns (line, col).
pub fn word_end(buf: &PieceTable, line: usize, col: usize) -> (usize, usize) {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    let mut c = col + 1;
    if c >= chars.len() {
        if line + 1 < buf.line_count() {
            let next = buf.line(line + 1).unwrap_or_default();
            let nchars: Vec<char> = next.chars().collect();
            let mut nc = 0;
            while nc < nchars.len() && nchars[nc].is_whitespace() {
                nc += 1;
            }
            while nc + 1 < nchars.len() && is_word_char(nchars[nc + 1]) {
                nc += 1;
            }
            return (line + 1, nc);
        }
        return (line, chars.len().saturating_sub(1));
    }
    // Skip whitespace
    while c < chars.len() && chars[c].is_whitespace() {
        c += 1;
    }
    // Skip to end of word
    while c + 1 < chars.len() && is_word_char(chars[c + 1]) {
        c += 1;
    }
    (line, c.min(chars.len().saturating_sub(1)))
}

/// Find first non-blank column on a line.
pub fn first_non_blank(buf: &PieceTable, line: usize) -> usize {
    let text = buf.line(line).unwrap_or_default();
    text.chars().take_while(|c| c.is_whitespace()).count()
}

/// Find char forward on line. Returns column or None.
pub fn find_char(buf: &PieceTable, line: usize, col: usize, target: char) -> Option<usize> {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .skip(col + 1)
        .find(|(_, c)| **c == target)
        .map(|(i, _)| i)
}

/// Find char backward on line. Returns column or None.
pub fn find_char_back(buf: &PieceTable, line: usize, col: usize, target: char) -> Option<usize> {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .take(col)
        .rev()
        .find(|(_, c)| **c == target)
        .map(|(i, _)| i)
}

/// Find matching bracket. Returns (line, col) or None.
pub fn match_bracket(buf: &PieceTable, line: usize, col: usize) -> Option<(usize, usize)> {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    let ch = *chars.get(col)?;
    let (open, close, forward) = match ch {
        '(' => ('(', ')', true),
        ')' => ('(', ')', false),
        '[' => ('[', ']', true),
        ']' => ('[', ']', false),
        '{' => ('{', '}', true),
        '}' => ('{', '}', false),
        _ => return None,
    };
    let content = buf.content();
    let offset = buf.line_col_to_offset(line, col)?;
    let bytes: Vec<char> = content.chars().collect();
    if offset >= bytes.len() {
        return None;
    }
    if forward {
        scan_forward(&bytes, offset, open, close, buf)
    } else {
        scan_backward(&bytes, offset, open, close, buf)
    }
}

fn scan_forward(bytes: &[char], offset: usize, open: char, close: char, buf: &PieceTable) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    for i in offset..bytes.len() {
        if bytes[i] == open {
            depth += 1;
        } else if bytes[i] == close {
            depth -= 1;
            if depth == 0 {
                let byte_off: usize = bytes[..i].iter().map(|c| c.len_utf8()).sum();
                return Some(buf.offset_to_line_col(byte_off));
            }
        }
    }
    None
}

fn scan_backward(bytes: &[char], offset: usize, open: char, close: char, buf: &PieceTable) -> Option<(usize, usize)> {
    let mut depth = 0i32;
    for i in (0..=offset).rev() {
        if bytes[i] == close {
            depth += 1;
        } else if bytes[i] == open {
            depth -= 1;
            if depth == 0 {
                let byte_off: usize = bytes[..i].iter().map(|c| c.len_utf8()).sum();
                return Some(buf.offset_to_line_col(byte_off));
            }
        }
    }
    None
}

/// Extract word at cursor position.
pub fn word_at(buf: &PieceTable, line: usize, col: usize) -> Option<String> {
    let text = buf.line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if col >= chars.len() || !is_word_char(chars[col]) {
        return None;
    }
    let start = chars[..col]
        .iter()
        .rposition(|c| !is_word_char(*c))
        .map_or(0, |p| p + 1);
    let end = chars[col..]
        .iter()
        .position(|c| !is_word_char(*c))
        .map_or(chars.len(), |p| col + p);
    Some(chars[start..end].iter().collect())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_forward() {
        let buf = PieceTable::from_text("hello world foo");
        assert_eq!(word_forward(&buf, 0, 0), (0, 6));
        assert_eq!(word_forward(&buf, 0, 6), (0, 12));
    }

    #[test]
    fn test_word_backward() {
        let buf = PieceTable::from_text("hello world");
        assert_eq!(word_backward(&buf, 0, 6), (0, 0));
    }

    #[test]
    fn test_word_end() {
        let buf = PieceTable::from_text("hello world");
        assert_eq!(word_end(&buf, 0, 0), (0, 4));
    }

    #[test]
    fn test_first_non_blank() {
        let buf = PieceTable::from_text("    hello");
        assert_eq!(first_non_blank(&buf, 0), 4);
    }

    #[test]
    fn test_find_char() {
        let buf = PieceTable::from_text("hello world");
        assert_eq!(find_char(&buf, 0, 0, 'o'), Some(4));
        assert_eq!(find_char_back(&buf, 0, 7, 'o'), Some(4));
    }

    #[test]
    fn test_match_bracket() {
        let buf = PieceTable::from_text("(hello)");
        assert_eq!(match_bracket(&buf, 0, 0), Some((0, 6)));
        assert_eq!(match_bracket(&buf, 0, 6), Some((0, 0)));
    }

    #[test]
    fn test_match_bracket_offset_past_end() {
        // Cursor col beyond content length should return None, not panic
        let buf = PieceTable::from_text("(");
        assert_eq!(match_bracket(&buf, 0, 0), None); // no matching close
        assert_eq!(match_bracket(&buf, 0, 5), None); // col past end
        assert_eq!(match_bracket(&buf, 99, 0), None); // line past end
    }
}
