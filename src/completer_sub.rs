//! Sub-command completers for theme, split, and lsp.

use txv_core::complete::CompletionVisitor;

use super::path;
use crate::completer::LspLanguageList;
use crate::completer_entry::Entry;
use crate::highlight::Highlighter;

/// Theme sub-argument completions.
pub(crate) fn complete_theme(sub: &str, visitor: &mut CompletionVisitor<'_>) -> Result<(), Box<dyn std::error::Error>> {
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
    let themes = Highlighter::new();
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

pub(crate) fn complete_options(
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

/// Split sub-argument completions.
pub(crate) fn complete_split(
    sub: &str,
    root: &std::path::Path,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    const SPLIT_SUBS: &[&str] = &["close", "focus", "hsplit", "linked", "open", "vsplit"];

    if let Some(path_part) = sub.strip_prefix("open ") {
        return path::complete_fs(path_part, root, "split open", &path::accept_all, visitor);
    }
    if let Some(path_part) = sub.strip_prefix("vsplit ") {
        return path::complete_fs(path_part, root, "split vsplit", &path::accept_all, visitor);
    }
    if let Some(path_part) = sub.strip_prefix("hsplit ") {
        return path::complete_fs(path_part, root, "split hsplit", &path::accept_all, visitor);
    }
    complete_options(SPLIT_SUBS, "split", sub, "command", visitor)
}

/// LSP sub-argument completions.
pub(crate) fn complete_lsp(
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

/// :set option completions.
pub(crate) fn complete_set_options(
    sub: &str,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    const SET_OPTS: &[&str] = &[
        "wrap", "nowrap", "list", "nolist", "number", "nonumber",
        "rainbow", "norainbow", "guides", "noguides",
        "gutter-signs", "nogutter-signs",
        "incsearch", "noincsearch", "matchparen", "nomatchparen",
    ];
    complete_options(SET_OPTS, "set", sub, "option", visitor)
}
