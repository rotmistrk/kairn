//! Git blame — async per-line blame annotations via git2.

use std::path::Path;
use std::sync::{Arc, Mutex};

/// A single line's blame annotation.
#[derive(Debug, Clone)]
pub struct BlameLine {
    pub(crate) hash: String,
    pub(crate) author: String,
    pub(crate) date: String,
    pub(crate) line: usize,
}

/// Blame state for a file.
#[derive(Debug, Clone)]
pub enum BlameState {
    Loading,
    Ready(Vec<BlameLine>),
    Error(String),
}

/// Shared blame result, written by background thread.
pub type SharedBlame = Arc<Mutex<BlameState>>;

/// Spawn a background thread to compute blame for a file.
pub fn blame_async(root: &Path, rel_path: &Path) -> SharedBlame {
    let state: SharedBlame = Arc::new(Mutex::new(BlameState::Loading));
    let state_clone = Arc::clone(&state);
    let root = root.to_path_buf();
    let rel = rel_path.to_path_buf();

    std::thread::spawn(move || {
        let result = compute_blame(&root, &rel);
        if let Ok(mut guard) = state_clone.lock() {
            *guard = result;
        }
    });

    state
}

fn compute_blame(root: &Path, rel_path: &Path) -> BlameState {
    let repo = match git2::Repository::discover(root) {
        Ok(r) => r,
        Err(e) => return BlameState::Error(format!("Not a git repo: {e}")),
    };
    let blame = match repo.blame_file(rel_path, None) {
        Ok(b) => b,
        Err(e) => return BlameState::Error(format!("Blame failed: {e}")),
    };
    let mut lines = Vec::new();
    for hunk in blame.iter() {
        let sig = hunk.final_signature();
        let author = sig.name().unwrap_or("?").to_string();
        let hash = format!("{}", hunk.final_commit_id());
        let time = sig.when();
        let date = format_epoch(time.seconds());
        let start = hunk.final_start_line();
        for i in 0..hunk.lines_in_hunk() {
            lines.push(BlameLine {
                hash: hash[..7.min(hash.len())].to_string(),
                author: truncate(&author, 10),
                date: date.clone(),
                line: start + i - 1,
            });
        }
    }
    lines.sort_by_key(|l| l.line);
    BlameState::Ready(lines)
}

fn format_epoch(secs: i64) -> String {
    // Simple date formatting: YYYY-MM-DD
    let days = secs / 86400;
    let mut y = 1970i32;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let months = [
        31,
        if is_leap(y) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1u32;
    for &dm in &months {
        if remaining < dm {
            break;
        }
        remaining -= dm;
        m += 1;
    }
    let d = remaining + 1;
    format!("{y:04}-{m:02}-{d:02}")
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s[..max].to_string()
    }
}
