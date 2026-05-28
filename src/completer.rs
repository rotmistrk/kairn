//! Completers — dynamic command completion + file path completion for kairn.

#[path = "completer_path.rs"]
mod path;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use txv_core::complete::{Completer, CompletionVisitor};

pub(crate) use crate::completer_entry::Entry;

/// Built-in commands (always available).
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
    fn complete(
        &self,
        input: &str,
        _cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let trimmed = input.trim();
        if let Some(path_part) = trimmed.strip_prefix("edit ") {
            return path::complete_path(path_part, &self.root, visitor);
        }
        if let Some(path_part) = trimmed.strip_prefix("e ") {
            return path::complete_path(path_part, &self.root, visitor);
        }
        if let Some(sub) = trimmed.strip_prefix("theme ") {
            return complete_theme(sub, visitor);
        }
        if let Some(sub) = trimmed.strip_prefix("lsp ") {
            return complete_lsp(sub, &self.lsp_languages, visitor);
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

/// Theme sub-argument completions.
fn complete_theme(sub: &str, visitor: &mut CompletionVisitor<'_>) -> Result<(), Box<dyn std::error::Error>> {
    const THEME_SUBS: &[&str] = &["auto", "dark", "glyphs", "light", "syntax"];
    const GLYPH_OPTS: &[&str] = &["ascii", "nerd", "utf"];

    if let Some(partial) = sub.strip_prefix("syntax ") {
        return complete_syntax_themes(partial, visitor);
    }
    if let Some(partial) = sub.strip_prefix("glyphs ") {
        return complete_options(GLYPH_OPTS, "theme glyphs", partial, "option", visitor);
    }
    complete_options(THEME_SUBS, "theme", sub, "command", visitor)
}

fn complete_syntax_themes(
    partial: &str,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let themes = crate::highlight::Highlighter::new();
    let mut names: Vec<&str> = themes.available_themes();
    names.sort();
    for t in names.into_iter().filter(|t| t.starts_with(partial)) {
        let e = Entry {
            text: format!("theme syntax {t}"),
            display: t.to_string(),
            kind: "theme",
        };
        if !visitor(&e)? {
            break;
        }
    }
    Ok(())
}

fn complete_options(
    opts: &[&str],
    prefix: &str,
    partial: &str,
    kind: &'static str,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    for o in opts.iter().filter(|o| o.starts_with(partial)) {
        let e = Entry {
            text: format!("{prefix} {o}"),
            display: o.to_string(),
            kind,
        };
        if !visitor(&e)? {
            break;
        }
    }
    Ok(())
}

/// LSP sub-argument completions.
fn complete_lsp(
    sub: &str,
    langs: &LspLanguageList,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    const LSP_SUBS: &[&str] = &["args", "restart", "start", "status", "stop", "timeout"];

    if let Some((subcmd, partial)) = sub.split_once(' ') {
        if !LSP_SUBS.contains(&subcmd) || subcmd == "status" {
            return Ok(());
        }
        let languages = langs.lock().unwrap_or_else(|e| e.into_inner());
        for l in languages.iter().filter(|l| l.starts_with(partial)) {
            let e = Entry {
                text: format!("lsp {subcmd} {l}"),
                display: l.clone(),
                kind: "lang",
            };
            if !visitor(&e)? {
                break;
            }
        }
        return Ok(());
    }
    for s in LSP_SUBS.iter().filter(|s| s.starts_with(sub)) {
        let e = Entry {
            text: format!("lsp {s}"),
            display: s.to_string(),
            kind: "command",
        };
        if !visitor(&e)? {
            break;
        }
    }
    Ok(())
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
}
