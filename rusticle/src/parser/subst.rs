//! Pipe operator preprocessing.
//!
//! Rewrites `a | b` to `b [a]` at the text level before parsing.

/// Preprocess pipe operators in a script.
///
/// Transforms `expr1 | cmd arg` into `cmd arg [expr1]`.
/// Pipes only apply at the top level of a line (not inside braces/brackets/quotes).
pub fn preprocess_pipes(input: &str) -> String {
    let mut result = String::new();
    for raw_line in input.lines() {
        if !result.is_empty() {
            result.push('\n');
        }
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            result.push_str(raw_line);
            continue;
        }
        let segments = split_pipe_segments(raw_line);
        if segments.len() <= 1 {
            result.push_str(raw_line);
        } else {
            result.push_str(&fold_pipe_segments(&segments));
        }
    }
    result
}

/// Split a line on top-level `|` operators.
fn split_pipe_segments(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    let mut brace_depth = 0;
    let mut bracket_depth = 0;
    let mut in_quotes = false;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() {
            current.push(ch);
            current.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if ch == '"' && brace_depth == 0 {
            in_quotes = !in_quotes;
        }
        if !in_quotes && brace_depth == 0 {
            if ch == '[' {
                bracket_depth += 1;
            } else if ch == ']' && bracket_depth > 0 {
                bracket_depth -= 1;
            }
        }
        if !in_quotes && bracket_depth == 0 {
            if ch == '{' {
                brace_depth += 1;
            } else if ch == '}' && brace_depth > 0 {
                brace_depth -= 1;
            }
        }
        let is_pipe = ch == '|'
            && brace_depth == 0
            && bracket_depth == 0
            && !in_quotes
            && !is_double_pipe(&chars, i);
        if is_pipe {
            segments.push(current.trim().to_string());
            current = String::new();
            i += 1;
            continue;
        }
        current.push(ch);
        i += 1;
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }
    segments
}

/// Check if `|` at position `i` is part of `||`.
fn is_double_pipe(chars: &[char], i: usize) -> bool {
    (i + 1 < chars.len() && chars[i + 1] == '|') || (i > 0 && chars[i - 1] == '|')
}

/// Fold pipe segments: `a | b | c` → `c [b [a]]`.
fn fold_pipe_segments(segments: &[String]) -> String {
    if segments.is_empty() {
        return String::new();
    }
    let mut result = segments[0].clone();
    for seg in &segments[1..] {
        result = format!("{seg} [{result}]");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_pipe() {
        assert_eq!(preprocess_pipes("puts hello"), "puts hello");
    }

    #[test]
    fn simple_pipe() {
        let result = preprocess_pipes("a | b");
        assert_eq!(result, "b [a]");
    }

    #[test]
    fn chained_pipes() {
        let result = preprocess_pipes("a | b | c");
        assert_eq!(result, "c [b [a]]");
    }

    #[test]
    fn pipe_in_braces_ignored() {
        let result = preprocess_pipes("set x {a | b}");
        assert_eq!(result, "set x {a | b}");
    }

    #[test]
    fn pipe_in_quotes_ignored() {
        let result = preprocess_pipes(r#"set x "a | b""#);
        assert_eq!(result, r#"set x "a | b""#);
    }

    #[test]
    fn double_pipe_not_split() {
        let result = preprocess_pipes("expr {$a || $b}");
        assert_eq!(result, "expr {$a || $b}");
    }
}
