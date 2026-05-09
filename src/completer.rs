//! Completers — FilePathCompleter and CommandCompleter for kairn.

use std::path::{Path, PathBuf};

use txv_core::complete::{Completer, Completion};

/// Known commands for the M-x prompt and :command mode.
const COMMANDS: &[&str] = &[
    "close", "e", "edit", "help", "quit", "save", "shell",
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

/// Combined completer: command names + file paths for edit/e commands.
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
        // If input starts with a file-editing command, complete paths
        if let Some(path_part) = trimmed.strip_prefix("edit ") {
            return complete_path(path_part, &self.root);
        }
        if let Some(path_part) = trimmed.strip_prefix("e ") {
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
    let (search_dir, prefix, dir_prefix) = if partial.is_empty() {
        (root.to_path_buf(), "", String::new())
    } else if partial.ends_with('/') || partial.ends_with(std::path::MAIN_SEPARATOR) {
        // "src/" → list contents of src/
        (root.join(partial), "", partial.to_string())
    } else if partial.contains('/') || partial.contains(std::path::MAIN_SEPARATOR) {
        // "src/ma" → list src/ filtered by "ma"
        let p = Path::new(partial);
        let parent = p.parent().map(|d| d.to_str().unwrap_or(".")).unwrap_or(".");
        let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let dir_prefix = format!("{}/", parent);
        (root.join(parent), prefix, dir_prefix)
    } else {
        // "READ" → list root filtered by "READ"
        (root.to_path_buf(), partial, String::new())
    };

    let Ok(entries) = std::fs::read_dir(&search_dir) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let rel_path = format!("{dir_prefix}{name_str}");
        let text = format!("edit {rel_path}");
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
