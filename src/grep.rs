//! Grep — async project search with wake pipe integration.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use txv_core::run::Waker;

use crate::views::results::ResultEntry;

/// Shared grep state between background thread and UI.
pub struct GrepState {
    pub entries: Mutex<Vec<ResultEntry>>,
    pub done: Mutex<bool>,
}

impl GrepState {
    pub fn take_entries(&self) -> Vec<ResultEntry> {
        self.entries.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or_default()
    }

    pub fn is_done(&self) -> bool {
        self.done.lock().map(|d| *d).unwrap_or(false)
    }
}

/// Spawn async grep. Returns shared state. Waker pokes the event loop when results arrive.
pub fn grep_async(pattern: &str, root: &Path, waker: Waker) -> Arc<GrepState> {
    let state = Arc::new(GrepState {
        entries: Mutex::new(Vec::new()),
        done: Mutex::new(false),
    });
    let state_clone = state.clone();
    let pattern = pattern.to_string();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        let child = Command::new("rg")
            .args([
                "--line-number", "--no-heading", "--color=never",
                "--max-count=10", "--max-columns=200", &pattern,
            ])
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
            if let Ok(mut d) = state_clone.done.lock() { *d = true; }
            waker.wake();
            return;
        };

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut batch = Vec::with_capacity(16);
        let mut count = 0;

        for line in reader.lines().map_while(Result::ok) {
            if let Some(entry) = parse_grep_line(&line, &root) {
                batch.push(entry);
                count += 1;
                if batch.len() >= 16 {
                    if let Ok(mut v) = state_clone.entries.lock() {
                        v.append(&mut batch);
                    }
                    waker.wake();
                }
            }
            if count >= 1000 {
                break;
            }
        }
        if !batch.is_empty() {
            if let Ok(mut v) = state_clone.entries.lock() {
                v.append(&mut batch);
            }
        }
        let _ = child.wait();
        if let Ok(mut d) = state_clone.done.lock() { *d = true; }
        waker.wake();
    });

    state
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

    #[test]
    fn parse_rg_output() {
        let root = PathBuf::from("/project");
        let e = parse_grep_line("src/main.rs:42:fn main() {", &root).unwrap();
        assert_eq!(e.path, PathBuf::from("/project/src/main.rs"));
        assert_eq!(e.line, 41);
    }

    #[test]
    fn parse_invalid_skipped() {
        let root = PathBuf::from("/p");
        assert!(parse_grep_line("no-colon", &root).is_none());
        assert!(parse_grep_line("f.rs:bad:x", &root).is_none());
    }
}
