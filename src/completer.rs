//! Completers — FilePathCompleter and CommandCompleter for kairn.

use std::path::{Path, PathBuf};

use txv_core::complete::{Completer, Completion};

/// Known commands for the M-x prompt and :command mode.
const COMMANDS: &[&str] = &[
    "close", "help", "open", "quit", "save", "shell",
];

/// Completer for kairn application commands.
pub struct CommandCompleter;

impl Completer for CommandCompleter {
    fn complete(&self, input: &str, _cursor: usize) -> Vec<Completion> {
        let trimmed = input.trim();
        COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                kind: "command",
            })
            .collect()
    }
}

/// Combined completer: command names + file paths for open/e commands.
/// Resolves file paths relative to a root directory.
pub struct AppCompleter {
    root: PathBuf,
}

impl AppCompleter {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl Completer for AppCompleter {
    fn complete(&self, input: &str, _cursor: usize) -> Vec<Completion> {
        let trimmed = input.trim();
        // If input starts with a file-opening command, complete paths
        if let Some(path_part) = trimmed.strip_prefix("open ") {
            return complete_path(path_part, &self.root);
        }
        // Otherwise complete command names
        COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .map(|cmd| Completion {
                text: cmd.to_string(),
                display: cmd.to_string(),
                kind: "command",
            })
            .collect()
    }
}

/// Complete filesystem paths relative to root dir.
fn complete_path(partial: &str, root: &Path) -> Vec<Completion> {
    let search_dir = root.join(
        if partial.is_empty() { "." }
        else if partial.contains('/') || partial.contains(std::path::MAIN_SEPARATOR) {
            Path::new(partial).parent().map(|p| p.to_str().unwrap_or(".")).unwrap_or(".")
        } else { "." }
    );

    let prefix = if partial.contains('/') || partial.contains(std::path::MAIN_SEPARATOR) {
        Path::new(partial).file_name().and_then(|n| n.to_str()).unwrap_or("")
    } else {
        partial
    };

    let Ok(entries) = std::fs::read_dir(&search_dir) else {
        return Vec::new();
    };

    let dir_prefix = if partial.contains('/') {
        let p = Path::new(partial);
        p.parent().map(|d| format!("{}/", d.display())).unwrap_or_default()
    } else {
        String::new()
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        // Build the completion text as "open <relative_path>"
        let rel_path = format!("{dir_prefix}{name_str}");
        let text = format!("open {rel_path}");
        let display = if entry.path().is_dir() {
            format!("{name_str}/")
        } else {
            name_str.to_string()
        };
        results.push(Completion { text, display, kind: if entry.path().is_dir() { "dir" } else { "file" } });
    }
    results.sort_by(|a, b| a.display.cmp(&b.display));
    results
}
