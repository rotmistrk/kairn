//! Structured literal parsing: `%{ key: value }` and `%[ item, item ]`.

use crate::error::TclError;

/// Parse a `%{...}` dict literal starting after `%{`.
/// Returns the raw content between the braces.
pub fn parse_dict_literal(
    chars: &[char],
    pos: &mut usize,
    line: &mut usize,
) -> Result<String, TclError> {
    parse_balanced(chars, pos, line, '{', '}')
}

/// Parse a `%[...]` list literal starting after `%[`.
/// Returns the raw content between the brackets.
pub fn parse_list_literal(
    chars: &[char],
    pos: &mut usize,
    line: &mut usize,
) -> Result<String, TclError> {
    parse_balanced(chars, pos, line, '[', ']')
}

/// Parse balanced delimiters, returning the content between them.
fn parse_balanced(
    chars: &[char],
    pos: &mut usize,
    line: &mut usize,
    open: char,
    close: char,
) -> Result<String, TclError> {
    let mut depth = 1;
    let mut content = String::new();
    while *pos < chars.len() && depth > 0 {
        let ch = chars[*pos];
        if ch == '\n' {
            *line += 1;
        }
        if ch == open {
            depth += 1;
            content.push(ch);
        } else if ch == close {
            depth -= 1;
            if depth > 0 {
                content.push(ch);
            }
        } else if ch == '"' {
            content.push(ch);
            *pos += 1;
            // Read until closing quote
            while *pos < chars.len() && chars[*pos] != '"' {
                if chars[*pos] == '\n' {
                    *line += 1;
                }
                if chars[*pos] == '\\' && *pos + 1 < chars.len() {
                    content.push(chars[*pos]);
                    *pos += 1;
                }
                content.push(chars[*pos]);
                *pos += 1;
            }
            if *pos < chars.len() {
                content.push(chars[*pos]); // closing "
            }
        } else {
            content.push(ch);
        }
        *pos += 1;
    }
    if depth != 0 {
        return Err(TclError::new(format!(
            "unmatched '{open}' in structured literal"
        )));
    }
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dict_literal() {
        let input: Vec<char> = r#"name: "kairn", ver: 1 }"#.chars().collect();
        let mut pos = 0;
        let mut line = 1;
        let result = parse_dict_literal(&input, &mut pos, &mut line);
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("name"));
        assert!(content.contains("kairn"));
    }

    #[test]
    fn nested_dict_literal() {
        let _input: Vec<char> = r#"a: %{ b: 1 } }"#.chars().collect();
        let mut pos = 0;
        let mut line = 1;
        // The outer } closes the literal; inner %{ b: 1 } is nested
        // But since we start after %{, the content has a nested { }
        // Actually this test needs the { } to be balanced
        let input2: Vec<char> = "a: { b: 1 } }".chars().collect();
        let result = parse_dict_literal(&input2, &mut pos, &mut line);
        assert!(result.is_ok());
    }

    #[test]
    fn simple_list_literal() {
        let input: Vec<char> = r#""a", "b", "c" ]"#.chars().collect();
        let mut pos = 0;
        let mut line = 1;
        let result = parse_list_literal(&input, &mut pos, &mut line);
        assert!(result.is_ok());
    }

    #[test]
    fn unmatched_brace_error() {
        let input: Vec<char> = "a: 1".chars().collect();
        let mut pos = 0;
        let mut line = 1;
        let result = parse_dict_literal(&input, &mut pos, &mut line);
        assert!(result.is_err());
    }
}
