//! Build output error parser — extracts file:line:col from compiler output.
//! Supports: Rust (multi-line), GCC/Clang, Go, and generic file:line:col patterns.

use std::path::Path;

use crate::views::results::ResultEntry;

/// Parse a single line of build output into a ResultEntry.
/// Returns None for non-error lines.
pub fn parse_line(line: &str, root: &Path) -> Option<ResultEntry> {
    parse_rust_arrow(line, root).or_else(|| parse_gcc_style(line, root))
}

/// Parse Rust "  --> file:line:col" format.
fn parse_rust_arrow(line: &str, root: &Path) -> Option<ResultEntry> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("-->")?;
    let rest = rest.trim();
    let (file, rest) = rest.rsplit_once(':')?;
    let col: u32 = rest.parse().ok()?;
    let (file, rest) = file.rsplit_once(':')?;
    let line_num: u32 = rest.parse().ok()?;
    let path = root.join(file);
    Some(ResultEntry {
        path,
        line: line_num.saturating_sub(1),
        col: col.saturating_sub(1),
        text: String::new(),
    })
}

/// Parse "file:line:col: message" (gcc, clang, go, typescript).
fn parse_gcc_style(line: &str, root: &Path) -> Option<ResultEntry> {
    // Skip lines that don't look like paths
    let first_char = line.chars().next()?;
    if first_char == ' ' || first_char == '\t' {
        return None;
    }
    // Find file:line:col: pattern
    let mut parts = line.splitn(4, ':');
    let file = parts.next()?.trim();
    if file.is_empty() {
        return None;
    }
    let line_str = parts.next()?;
    let line_num: u32 = line_str.trim().parse().ok()?;
    let col_or_msg = parts.next()?;
    let (col, message) = if let Ok(c) = col_or_msg.trim().parse::<u32>() {
        let msg = parts.next().unwrap_or("").trim().to_string();
        (c, msg)
    } else {
        (1, col_or_msg.trim().to_string())
    };
    let path = root.join(file);
    Some(ResultEntry {
        path,
        line: line_num.saturating_sub(1),
        col: col.saturating_sub(1),
        text: message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_rust_arrow_line() {
        let root = PathBuf::from("/project");
        let entry = parse_line("  --> src/main.rs:42:5", &root).unwrap();
        assert_eq!(entry.path, PathBuf::from("/project/src/main.rs"));
        assert_eq!(entry.line, 41);
        assert_eq!(entry.col, 4);
    }

    #[test]
    fn parse_gcc_error() {
        let root = PathBuf::from("/project");
        let entry = parse_line("src/lib.c:10:3: error: undeclared", &root).unwrap();
        assert_eq!(entry.path, PathBuf::from("/project/src/lib.c"));
        assert_eq!(entry.line, 9);
        assert_eq!(entry.col, 2);
        assert!(entry.text.contains("error"));
    }

    #[test]
    fn parse_go_error() {
        let root = PathBuf::from("/project");
        let entry = parse_line("./main.go:15:2: undefined: foo", &root).unwrap();
        assert_eq!(entry.path, PathBuf::from("/project/./main.go"));
        assert_eq!(entry.line, 14);
        assert_eq!(entry.col, 1);
    }

    #[test]
    fn parse_no_match() {
        let root = PathBuf::from("/project");
        assert!(parse_line("Compiling kairn v0.1.0", &root).is_none());
        assert!(parse_line("   Finished dev", &root).is_none());
    }

    #[test]
    fn parse_file_line_only() {
        let root = PathBuf::from("/project");
        let entry = parse_line("test.py:42: SyntaxError", &root).unwrap();
        assert_eq!(entry.line, 41);
        assert_eq!(entry.col, 0);
    }
}
