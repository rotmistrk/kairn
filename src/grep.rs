//! Grep — search project files for a pattern, return ResultEntry list.

use std::path::Path;
use std::process::Command;

use crate::views::results::ResultEntry;

/// Run grep on the project root, return matching entries.
/// Uses `grep -rn` with .gitignore-aware exclusions.
pub fn grep_project(pattern: &str, root: &Path) -> Vec<ResultEntry> {
    // Try ripgrep first, fall back to grep
    let output = Command::new("rg")
        .args(["--line-number", "--no-heading", "--color=never", pattern])
        .current_dir(root)
        .output()
        .or_else(|_| {
            Command::new("grep")
                .args(["-rn", "--include=*", pattern, "."])
                .current_dir(root)
                .output()
        });

    let Ok(output) = output else {
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_grep_output(&stdout, root)
}

fn parse_grep_output(output: &str, root: &Path) -> Vec<ResultEntry> {
    let mut entries = Vec::new();
    for line in output.lines().take(500) {
        if let Some(entry) = parse_grep_line(line, root) {
            entries.push(entry);
        }
    }
    entries
}

fn parse_grep_line(line: &str, root: &Path) -> Option<ResultEntry> {
    // Format: path:line:text  or  ./path:line:text
    let (path_str, rest) = line.split_once(':')?;
    let (line_str, text) = rest.split_once(':')?;
    let line_num: u32 = line_str.parse().ok()?;
    let path = root.join(path_str.strip_prefix("./").unwrap_or(path_str));
    Some(ResultEntry {
        path,
        line: line_num.saturating_sub(1), // 0-indexed
        col: 0,
        text: text.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_rg_output() {
        let output = "src/main.rs:42:fn main() {\nsrc/lib.rs:10:pub mod foo;\n";
        let root = PathBuf::from("/project");
        let entries = parse_grep_output(output, &root);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, PathBuf::from("/project/src/main.rs"));
        assert_eq!(entries[0].line, 41); // 0-indexed
        assert_eq!(entries[0].text, "fn main() {");
        assert_eq!(entries[1].line, 9);
    }

    #[test]
    fn parse_grep_with_dot_prefix() {
        let output = "./src/foo.rs:5:let x = 1;\n";
        let root = PathBuf::from("/proj");
        let entries = parse_grep_output(output, &root);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, PathBuf::from("/proj/src/foo.rs"));
    }

    #[test]
    fn parse_invalid_line_skipped() {
        let output = "no-colon-here\nsrc/a.rs:bad:text\nsrc/b.rs:3:ok\n";
        let root = PathBuf::from("/p");
        let entries = parse_grep_output(output, &root);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "ok");
    }
}
