//! Completers — dynamic command completion + file path completion for kairn.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use txv_core::complete::{Completer, Completion};

/// Built-in commands (always available).
/// Extra commands not in the dispatch table (ex-mode aliases handled elsewhere).
pub const BUILTIN_COMMANDS: &[&str] = &["dir", "file", "only"];

/// Shared command list that can be updated at runtime (e.g. from plugins).
pub type CommandList = Arc<Mutex<Vec<String>>>;

/// Shared list of known LSP language IDs for completions.
pub type LspLanguageList = Arc<Mutex<Vec<String>>>;

/// Create a new command list from the dispatch table + extras.
pub fn new_command_list() -> CommandList {
    let mut cmds: Vec<String> = crate::handler_exec::dispatch_table()
        .flat_map(|e| e.names.iter())
        .chain(BUILTIN_COMMANDS.iter())
        .map(|s| s.to_string())
        .collect();
    cmds.sort_unstable();
    cmds.dedup();
    Arc::new(Mutex::new(cmds))
}

/// Combined completer: dynamic command names + file paths for edit/e commands.
pub struct AppCompleter {
    root: PathBuf,
    commands: CommandList,
    lsp_languages: LspLanguageList,
}

impl AppCompleter {
    pub fn new(root: PathBuf, commands: CommandList) -> Self {
        Self {
            root,
            commands,
            lsp_languages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_lsp_languages(&mut self, langs: LspLanguageList) {
        self.lsp_languages = langs;
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
        // LSP sub-commands
        if let Some(sub) = trimmed.strip_prefix("lsp ") {
            return complete_lsp(sub, &self.lsp_languages);
        }
        // Otherwise complete command names
        let cmds = self.commands.lock().unwrap_or_else(|e| e.into_inner());
        cmds.iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .map(|cmd| Completion::new(cmd.clone(), cmd.clone(), "command"))
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
            .map(|t| Completion::new(format!("theme syntax {t}"), t.to_string(), "theme"))
            .collect();
    }
    if let Some(partial) = sub.strip_prefix("glyphs ") {
        return GLYPH_OPTS
            .iter()
            .filter(|o| o.starts_with(partial))
            .map(|o| Completion::new(format!("theme glyphs {o}"), o.to_string(), "option"))
            .collect();
    }
    // Complete first-level sub-commands
    THEME_SUBS
        .iter()
        .filter(|s| s.starts_with(sub))
        .map(|s| Completion::new(format!("theme {s}"), s.to_string(), "command"))
        .collect()
}

/// Complete filesystem paths relative to root dir.
/// LSP sub-argument completions.
fn complete_lsp(sub: &str, langs: &LspLanguageList) -> Vec<Completion> {
    const LSP_SUBS: &[&str] = &["args", "restart", "start", "status", "stop", "timeout"];

    // Check if we're past the subcommand (e.g. "start ru")
    if let Some((subcmd, partial)) = sub.split_once(' ') {
        if LSP_SUBS.contains(&subcmd) && subcmd != "status" {
            let languages = langs.lock().unwrap_or_else(|e| e.into_inner());
            return languages
                .iter()
                .filter(|l| l.starts_with(partial))
                .map(|l| Completion::new(format!("lsp {subcmd} {l}"), l.clone(), "lang"))
                .collect();
        }
        return Vec::new();
    }
    // Complete subcommand names
    LSP_SUBS
        .iter()
        .filter(|s| s.starts_with(sub))
        .map(|s| Completion::new(format!("lsp {s}"), s.to_string(), "command"))
        .collect()
}

fn resolve_path_parts<'a>(partial: &'a str, root: &Path) -> (PathBuf, &'a str, String) {
    if partial.is_empty() {
        return (root.to_path_buf(), "", String::new());
    }
    if partial.ends_with('/') || partial.ends_with(std::path::MAIN_SEPARATOR) {
        return (root.join(partial), "", partial.to_string());
    }
    if partial.contains('/') || partial.contains(std::path::MAIN_SEPARATOR) {
        let p = Path::new(partial);
        let parent = p.parent().map(|d| d.to_str().unwrap_or(".")).unwrap_or(".");
        let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let dir_prefix = format!("{}/", parent);
        return (root.join(parent), prefix, dir_prefix);
    }
    (root.to_path_buf(), partial, String::new())
}

fn complete_path(partial: &str, root: &Path) -> Vec<Completion> {
    let (search_dir, prefix, dir_prefix) = resolve_path_parts(partial, root);

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
        let is_dir = entry.path().is_dir();
        let display = if is_dir {
            format!("{name_str}/")
        } else {
            name_str.to_string()
        };
        results.push(Completion::new(
            text,
            display,
            if is_dir {
                "dir"
            } else {
                "file"
            },
        ));
    }
    results.sort_by(|a, b| a.display().cmp(b.display()));
    results
}

/// Refresh the command list with Tcl commands from the script engine.
pub fn refresh_commands(list: &CommandList, script: &crate::scripting::ScriptEngine) {
    let mut cmds: Vec<String> = crate::handler_exec::dispatch_table()
        .flat_map(|e| e.names.iter())
        .chain(BUILTIN_COMMANDS.iter())
        .map(|s| s.to_string())
        .collect();
    for name in script.command_names() {
        if !cmds.contains(&name) {
            cmds.push(name);
        }
    }
    cmds.sort();
    cmds.dedup();
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
