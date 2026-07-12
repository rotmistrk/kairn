//! Shared JSON string escaping utility.

/// Escape a string for safe inclusion in a JSON string literal.
///
/// Handles: double-quote, backslash, newline, carriage return, tab,
/// and all control characters below U+0020.
pub(crate) fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_basic_chars() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("a\"b"), "a\\\"b");
        assert_eq!(escape_json_string("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_escape_control_chars() {
        assert_eq!(escape_json_string("a\nb"), "a\\nb");
        assert_eq!(escape_json_string("a\rb"), "a\\rb");
        assert_eq!(escape_json_string("a\tb"), "a\\tb");
        assert_eq!(escape_json_string("\x01"), "\\u0001");
    }
}
