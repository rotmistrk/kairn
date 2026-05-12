//! Grep — search project files for a pattern, stream results incrementally.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;

use crate::views::results::ResultEntry;

/// Spawn grep in background, return receiver that yields batches of results.
/// Uses `rg` (respects .gitignore) or falls back to `grep -rn`.
pub fn grep_stream(
    pattern: &str,
    root: &Path,
) -> mpsc::Receiver<Vec<ResultEntry>> {
    let (tx, rx) = mpsc::channel();
    let pattern = pattern.to_string();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        let child = Command::new("rg")
            .args(["--line-number", "--no-heading", "--color=never", &pattern])
            .current_dir(&root)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .or_else(|_| {
                Command::new("grep")
                    .args(["-rn", "--include=*", &pattern, "."])
                    .current_dir(&root)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
            });

        let Ok(mut child) = child else {
            return;
        };

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut batch = Vec::with_capacity(32);
        let mut count = 0;

        for line in reader.lines().map_while(Result::ok) {
            if let Some(entry) = parse_grep_line(&line, &root) {
                batch.push(entry);
                count += 1;
                if (batch.len() >= 32 || count <= 32)
                    && tx.send(std::mem::take(&mut batch)).is_err()
                {
                    break;
                }
            }
            if count >= 1000 {
                break;
            }
        }
        if !batch.is_empty() {
            let _ = tx.send(batch);
        }
        let _ = child.wait();
    });

    rx
}

fn parse_grep_line(line: &str, root: &Path) -> Option<ResultEntry> {
    let (path_str, rest) = line.split_once(':')?;
    let (line_str, text) = rest.split_once(':')?;
    let line_num: u32 = line_str.parse().ok()?;
    let path = root.join(path_str.strip_prefix("./").unwrap_or(path_str));
    Some(ResultEntry {
        path,
        line: line_num.saturating_sub(1),
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
        let root = PathBuf::from("/project");
        let e = parse_grep_line("src/main.rs:42:fn main() {", &root).unwrap();
        assert_eq!(e.path, PathBuf::from("/project/src/main.rs"));
        assert_eq!(e.line, 41);
        assert_eq!(e.text, "fn main() {");
    }

    #[test]
    fn parse_grep_with_dot_prefix() {
        let root = PathBuf::from("/proj");
        let e = parse_grep_line("./src/foo.rs:5:let x = 1;", &root).unwrap();
        assert_eq!(e.path, PathBuf::from("/proj/src/foo.rs"));
    }

    #[test]
    fn parse_invalid_line_skipped() {
        let root = PathBuf::from("/p");
        assert!(parse_grep_line("no-colon-here", &root).is_none());
        assert!(parse_grep_line("src/a.rs:bad:text", &root).is_none());
    }
}
