//! Completers — FilePathCompleter and CommandCompleter for kairn.

use txv_core::complete::{Completer, Completion};

/// Known commands for the M-x prompt and :command mode.
const COMMANDS: &[&str] = &[
    "close", "help", "open", "quit", "save", "shell",
    "w", "q", "wq", "x", "e", "set",
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

/// Completer for file paths (used in :open/:e commands).
pub struct FilePathCompleter;

impl Completer for FilePathCompleter {
    fn complete(&self, input: &str, _cursor: usize) -> Vec<Completion> {
        let trimmed = input.trim();
        // Extract the path portion (after command like "open " or "e ")
        let path_part = if let Some(rest) = trimmed.strip_prefix("open ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("e ") {
            rest
        } else {
            return Vec::new();
        };

        complete_path(path_part)
    }
}

/// Complete filesystem paths from a partial input.
fn complete_path(partial: &str) -> Vec<Completion> {
    use std::path::Path;

    let (dir, prefix) = if partial.is_empty() {
        (".", "")
    } else if partial.ends_with('/') || partial.ends_with(std::path::MAIN_SEPARATOR) {
        (partial, "")
    } else {
        let p = Path::new(partial);
        let dir = p.parent().map(|d| d.to_str().unwrap_or(".")).unwrap_or(".");
        let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        (if dir.is_empty() { "." } else { dir }, prefix)
    };

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let full = if dir == "." {
            name_str.to_string()
        } else {
            format!("{dir}/{name_str}")
        };
        let display = if entry.path().is_dir() {
            format!("{name_str}/")
        } else {
            name_str.to_string()
        };
        results.push(Completion {
            text: full,
            display,
            kind: if entry.path().is_dir() { "dir" } else { "file" },
        });
    }
    results.sort_by(|a, b| a.display.cmp(&b.display));
    results
}

/// Combined completer that dispatches to command or file path completion.
pub struct EditorCompleter;

impl Completer for EditorCompleter {
    fn complete(&self, input: &str, cursor: usize) -> Vec<Completion> {
        let trimmed = input.trim();
        // If input starts with a file-opening command, complete paths
        if trimmed.starts_with("open ") || trimmed.starts_with("e ") {
            return FilePathCompleter.complete(input, cursor);
        }
        // Otherwise complete command names
        CommandCompleter.complete(input, cursor)
    }
}
