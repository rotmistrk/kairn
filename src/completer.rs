//! Completers — dynamic command completion + file path completion for kairn.

#[path = "completer_path.rs"]
mod path;

#[path = "completer_sub.rs"]
mod sub;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use txv_core::complete::{Completer, CompletionVisitor};

pub(crate) use crate::completer_entry::Entry;
use sub::{complete_lsp, complete_set_options, complete_split, complete_theme};

/// Built-in commands (always available).
pub const BUILTIN_COMMANDS: &[&str] = &["dir", "file", "only"];

/// Shared command list that can be updated at runtime (e.g. from plugins).
pub type CommandList = Arc<Mutex<Vec<String>>>;

/// Shared list of known LSP language IDs for completions.
pub type LspLanguageList = Arc<Mutex<Vec<String>>>;

/// Shared list of workspace root paths for completions.
pub type RootsList = Arc<Mutex<Vec<String>>>;

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
    roots: RootsList,
}

impl AppCompleter {
    pub fn new(root: PathBuf, commands: CommandList) -> Self {
        Self {
            root,
            commands,
            lsp_languages: Arc::new(Mutex::new(Vec::new())),
            roots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_lsp_languages(&mut self, langs: LspLanguageList) {
        self.lsp_languages = langs;
    }

    pub fn set_roots(&mut self, roots: RootsList) {
        self.roots = roots;
    }

    fn complete_roots(
        &self,
        partial: &str,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let roots = self.roots.lock().unwrap_or_else(|e| e.into_inner());
        for r in roots.iter().filter(|r| r.contains(partial)) {
            let e = Entry {
                text: format!("remove-root {r}"),
                display: r.clone(),
                kind: "path",
            };
            if !visitor(&e)? {
                break;
            }
        }
        Ok(())
    }
}

impl AppCompleter {
    fn complete_subcommand(
        &self,
        trimmed: &str,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Option<Result<(), Box<dyn std::error::Error>>> {
        if let Some(sub) = trimmed.strip_prefix("theme ") {
            return Some(complete_theme(sub, visitor));
        }
        if let Some(sub) = trimmed.strip_prefix("lsp ") {
            return Some(complete_lsp(sub, &self.lsp_languages, visitor));
        }
        if let Some(sub) = trimmed.strip_prefix("kiro ") {
            return Some(crate::completer_kiro::complete_kiro(sub, &self.root, visitor));
        }
        if let Some(sub) = trimmed.strip_prefix("split ") {
            return Some(complete_split(sub, &self.root, visitor));
        }
        if let Some(sub) = trimmed.strip_prefix("set ") {
            return Some(complete_set_options(sub, visitor));
        }
        None
    }

    fn complete_path_arg(
        &self,
        trimmed: &str,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Option<Result<(), Box<dyn std::error::Error>>> {
        let path_cmds: &[(&str, bool)] = &[
            ("edit ", false),
            ("e ", false),
            ("new-file ", false),
            ("delete-file ", false),
            ("rename-file ", false),
            ("copy-file ", false),
            ("git-stage ", false),
            ("git-unstage ", false),
            ("git-untrack ", false),
            ("add-root ", true),
            ("new-dir ", true),
        ];
        for &(prefix, dirs_only) in path_cmds {
            if let Some(path_part) = trimmed.strip_prefix(prefix) {
                let cmd = prefix.trim();
                let filter: &dyn Fn(&std::fs::DirEntry) -> bool = if dirs_only {
                    &path::accept_dirs
                } else {
                    &path::accept_all
                };
                return Some(path::complete_fs(path_part, &self.root, cmd, filter, visitor));
            }
        }
        None
    }
}

impl Completer for AppCompleter {
    fn complete(
        &self,
        input: &str,
        _cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let trimmed = input.trim_start();
        if let Some(partial) = trimmed.strip_prefix("remove-root ") {
            return self.complete_roots(partial, visitor);
        }
        if let Some(result) = self.complete_subcommand(trimmed, visitor) {
            return result;
        }
        if let Some(result) = self.complete_path_arg(trimmed, visitor) {
            return result;
        }
        let cmds = self.commands.lock().unwrap_or_else(|e| e.into_inner());
        for cmd in cmds.iter().filter(|c| c.starts_with(trimmed)) {
            let e = Entry {
                text: cmd.clone(),
                display: cmd.clone(),
                kind: "command",
            };
            if !visitor(&e)? {
                break;
            }
        }
        Ok(())
    }
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
        assert!(cmds.contains(&"build".to_string()), "should contain builtin 'build'");
        assert!(cmds.contains(&"quit".to_string()), "should contain builtin 'quit'");
        assert!(
            cmds.contains(&"set".to_string()),
            "should contain 'set' for :set options"
        );
        assert!(
            cmds.contains(&"editor".to_string()),
            "should contain Tcl bridge 'editor'"
        );
        let sorted: Vec<String> = {
            let mut c = cmds.clone();
            c.sort();
            c
        };
        assert_eq!(*cmds, sorted, "commands should be sorted");
    }

    #[test]
    fn set_subcommand_completes_options() {
        let completer = AppCompleter::new(std::path::PathBuf::from("/tmp"), new_command_list());
        let mut results = Vec::new();
        completer
            .complete("set ", 4, &mut |entry: &dyn txv_core::complete::Completion| {
                results.push(entry.text().to_string());
                Ok(true)
            })
            .unwrap();
        assert!(!results.is_empty(), "set should have completions, got: {:?}", results);
        assert!(
            results.iter().any(|r| r == "set wrap"),
            "should have 'set wrap', got: {:?}",
            results
        );
    }
}
