//! Grep — pure Rust async project search. No external tools.
//! Uses `ignore` crate (respects .gitignore) + `regex` for matching.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

use ignore::WalkBuilder;
use regex::Regex;
use txv_core::run::Waker;

use crate::views::results::ResultEntry;

/// Shared grep state between background thread and UI.
pub struct GrepState {
    pub entries: Mutex<Vec<ResultEntry>>,
    pub done: Mutex<bool>,
    pub error: Mutex<Option<String>>,
}

impl GrepState {
    pub fn take_entries(&self) -> Vec<ResultEntry> {
        self.entries.lock().map(|mut v| std::mem::take(&mut *v)).unwrap_or_default()
    }

    pub fn is_done(&self) -> bool {
        self.done.lock().map(|d| *d).unwrap_or(false)
    }

    pub fn take_error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|mut e| e.take())
    }
}

/// Spawn async grep. Pure Rust — no external tools.
pub fn grep_async(pattern: &str, root: &Path, waker: Waker) -> Arc<GrepState> {
    let state = Arc::new(GrepState {
        entries: Mutex::new(Vec::new()),
        done: Mutex::new(false),
        error: Mutex::new(None),
    });
    let state_clone = state.clone();
    let pattern = pattern.to_string();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        let re = match Regex::new(&pattern) {
            Ok(r) => r,
            Err(e) => {
                // Fall back to literal search if regex is invalid
                match Regex::new(&regex::escape(&pattern)) {
                    Ok(r) => r,
                    Err(e2) => {
                        if let Ok(mut err) = state_clone.error.lock() {
                            *err = Some(format!("Invalid pattern: {e2}"));
                        }
                        if let Ok(mut d) = state_clone.done.lock() { *d = true; }
                        waker.wake();
                        return;
                    }
                }
            }
        };

        let walker = WalkBuilder::new(&root)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .build();

        let mut count = 0;
        let mut batch = Vec::with_capacity(16);

        for entry in walker.flatten() {
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let file = match File::open(path) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let reader = BufReader::new(file);
            let mut file_matches = 0;

            for (line_idx, line) in reader.lines().enumerate() {
                let Ok(line) = line else { break };
                if re.is_match(&line) {
                    batch.push(ResultEntry {
                        path: path.to_path_buf(),
                        line: line_idx as u32,
                        col: 0,
                        text: line.chars().take(200).collect(),
                    });
                    count += 1;
                    file_matches += 1;
                    if file_matches >= 10 {
                        break;
                    }
                    if batch.len() >= 16 {
                        if let Ok(mut v) = state_clone.entries.lock() {
                            v.append(&mut batch);
                        }
                        waker.wake();
                    }
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
        if let Ok(mut d) = state_clone.done.lock() { *d = true; }
        waker.wake();
    });

    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn grep_finds_matches() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
        let state = grep_async("main", dir.path(), Waker::noop());
        std::thread::sleep(std::time::Duration::from_millis(100));
        let entries = state.take_entries();
        assert!(!entries.is_empty());
        assert!(state.is_done());
    }

    #[test]
    fn grep_respects_gitignore() {
        let dir = TempDir::new().unwrap();
        // ignore crate needs .git dir to respect .gitignore
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
        fs::create_dir(dir.path().join("ignored")).unwrap();
        fs::write(dir.path().join("ignored/file.rs"), "findme\n").unwrap();
        fs::write(dir.path().join("visible.rs"), "findme\n").unwrap();
        let state = grep_async("findme", dir.path(), Waker::noop());
        std::thread::sleep(std::time::Duration::from_millis(100));
        let entries = state.take_entries();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].path.ends_with("visible.rs"));
    }
}
