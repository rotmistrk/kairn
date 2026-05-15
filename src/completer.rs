//! Completers — dynamic command completion + file path completion for kairn.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use txv_core::complete::{Completer, Completion};

/// Built-in commands (always available).
pub const BUILTIN_COMMANDS: &[&str] = &[
    "build",
    "close",
    "code-action",
    "diff",
    "e",
    "edit",
    "git-commit",
    "git-stage",
    "git-unstage",
    "git-untrack",
    "grep",
    "grow",
    "grow-v",
    "help",
    "kiro",
    "lsp-rename",
    "lsp-status",
    "messages",
    "next-error",
    "paste",
    "prev-error",
    "quit",
    "run",
    "save",
    "shell",
    "shrink",
    "shrink-v",
    "struct",
    "tab",
    "tab-rename",
    "test",
    "test-at-cursor",
    "test-file",
    "text",
    "theme",
    "tree",
    "welcome",
];

/// Shared command list that can be updated at runtime (e.g. from plugins).
pub type CommandList = Arc<Mutex<Vec<String>>>;

/// Create a new command list pre-populated with built-in commands.
pub fn new_command_list() -> CommandList {
    Arc::new(Mutex::new(BUILTIN_COMMANDS.iter().map(|s| s.to_string()).collect()))
}

/// Combined completer: dynamic command names + file paths for edit/e commands.
pub struct AppCompleter {
    root: PathBuf,
    commands: CommandList,
}

impl AppCompleter {
    pub fn new(root: PathBuf, commands: CommandList) -> Self {
        Self { root, commands }
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
        // Theme sub-commands
        if let Some(sub) = trimmed.strip_prefix("theme ") {
            return complete_theme(sub);
        }
        // Otherwise complete command names
        let cmds = self.commands.lock().unwrap_or_else(|e| e.into_inner());
        cmds.iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .map(|cmd| Completion {
                text: cmd.clone(),
                display: cmd.clone(),
                kind: "command",
            })
            .collect()
    }
}

/// Theme sub-argument completions.
fn complete_theme(sub: &str) -> Vec<Completion> {
    const THEME_SUBS: &[&str] = &["auto", "dark", "glyphs", "light", "syntax"];
    const GLYPH_OPTS: &[&str] = &["ascii", "nerd", "utf"];

    if let Some(partial) = sub.strip_prefix("syntax ") {
        // Complete syntax theme names
        let themes = crate::highlight::Highlighter::new();
        let mut names: Vec<&str> = themes.available_themes();
        names.sort();
        return names
            .into_iter()
            .filter(|t| t.starts_with(partial))
            .map(|t| Completion {
                text: format!("theme syntax {t}"),
                display: t.to_string(),
                kind: "theme",
            })
            .collect();
    }
    if let Some(partial) = sub.strip_prefix("glyphs ") {
        return GLYPH_OPTS
            .iter()
            .filter(|o| o.starts_with(partial))
            .map(|o| Completion {
                text: format!("theme glyphs {o}"),
                display: o.to_string(),
                kind: "option",
            })
            .collect();
    }
    // Complete first-level sub-commands
    THEME_SUBS
        .iter()
        .filter(|s| s.starts_with(sub))
        .map(|s| Completion {
            text: format!("theme {s}"),
            display: s.to_string(),
            kind: "command",
        })
        .collect()
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
        results.push(Completion {
            text,
            display,
            kind: if entry.path().is_dir() {
                "dir"
            } else {
                "file"
            },
        });
    }
    results.sort_by(|a, b| a.display.cmp(&b.display));
    results
}

/// Refresh the command list with Tcl commands from the script engine.
pub fn refresh_commands(list: &CommandList, script: &crate::scripting::ScriptEngine) {
    let mut cmds: Vec<String> = BUILTIN_COMMANDS.iter().map(|s| s.to_string()).collect();
    for name in script.command_names() {
        if !cmds.contains(&name) {
            cmds.push(name);
        }
    }
    cmds.sort();
    if let Ok(mut guard) = list.lock() {
        *guard = cmds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::ScriptEngine;

    #[test]
    fn refresh_commands_includes_builtins_and_tcl_bridge() {
        let list = new_command_list();
        let script = ScriptEngine::new();
        refresh_commands(&list, &script);
        let cmds = list.lock().unwrap();
        // Should contain built-in kairn commands
        assert!(cmds.contains(&"build".to_string()), "should contain builtin 'build'");
        assert!(cmds.contains(&"quit".to_string()), "should contain builtin 'quit'");
        // Should contain Tcl bridge namespace commands (registered by ScriptEngine)
        assert!(
            cmds.contains(&"editor".to_string()),
            "should contain Tcl bridge 'editor'"
        );
        // Verify sorted
        let sorted: Vec<String> = {
            let mut c = cmds.clone();
            c.sort();
            c
        };
        assert_eq!(*cmds, sorted, "commands should be sorted");
    }
}
