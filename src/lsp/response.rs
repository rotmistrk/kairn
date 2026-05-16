//! LSP response routing — dispatches parsed responses to the right UI action.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::*;

use super::handler::{JdtRequest, PendingKind};
use super::requests;

pub(super) fn handle_response(kind: PendingKind, result: &serde_json::Value, queue: &mut EventQueue) {
    match kind {
        PendingKind::GotoDefinition => handle_goto_def(result, queue),
        PendingKind::GotoShow => handle_goto_show(result, queue),
        PendingKind::FindReferences { symbol } => handle_references(result, queue, &symbol),
        PendingKind::Hover => handle_hover(result, queue),
        PendingKind::Completion => handle_completion(result, queue),
        PendingKind::Rename => handle_rename(result, queue),
        PendingKind::CodeAction => handle_code_action(result, queue),
        PendingKind::JdtClassContents { line, character } => handle_jdt(result, queue, line, character),
    }
}

fn handle_goto_def(result: &serde_json::Value, queue: &mut EventQueue) {
    let locs = requests::parse_locations(result);
    if let Some(loc) = locs.into_iter().next() {
        log::info!("LSP: definition -> {}:{}", &loc.uri, loc.line);
        if loc.uri.starts_with("jdt://") {
            queue.put_command(
                CM_LSP_GOTO_DEF,
                Some(Box::new(JdtRequest {
                    uri: loc.uri,
                    line: loc.line,
                    character: loc.character,
                })),
            );
        } else {
            let path = uri_to_path(&loc.uri);
            let req = OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
            queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
        }
    } else {
        queue.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No definition found"))),
        );
    }
}

fn handle_goto_show(result: &serde_json::Value, queue: &mut EventQueue) {
    let locs = requests::parse_locations(result);
    if let Some(loc) = locs.into_iter().next() {
        let path = uri_to_path(&loc.uri);
        let req = OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
        queue.put_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
    } else {
        queue.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No definition found"))),
        );
    }
}

fn handle_references(result: &serde_json::Value, queue: &mut EventQueue, symbol: &str) {
    let locs = requests::parse_locations(result);
    log::info!("LSP: references -> {} locations", locs.len());
    if locs.len() == 1 {
        let loc = &locs[0];
        let path = uri_to_path(&loc.uri);
        let req = OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
        queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    } else if !locs.is_empty() {
        let entries: Vec<crate::views::results::ResultEntry> = locs
            .iter()
            .map(|l| {
                let path = PathBuf::from(uri_to_path(&l.uri));
                let text = read_line_from_file(&path, l.line);
                crate::views::results::ResultEntry {
                    path,
                    line: l.line,
                    col: l.character,
                    text,
                }
            })
            .collect();
        let title = if symbol.is_empty() {
            "References".to_string()
        } else {
            format!("References: {symbol}")
        };
        queue.put_command(CM_SHOW_RESULTS, Some(Box::new((title, entries))));
    } else {
        queue.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No references found"))),
        );
    }
}

fn handle_hover(result: &serde_json::Value, queue: &mut EventQueue) {
    if let Some(text) = requests::parse_hover(result) {
        log::info!("LSP: hover -> {} chars", text.len());
        queue.put_command(CM_DIAGNOSTIC, Some(Box::new(("hover".to_string(), text))));
    }
}

fn handle_completion(result: &serde_json::Value, queue: &mut EventQueue) {
    let items = requests::parse_completion(result);
    log::info!("LSP: completion -> {} items", items.len());
    if !items.is_empty() {
        queue.put_command(CM_LSP_COMPLETION, Some(Box::new(items)));
    } else {
        queue.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No completions"))),
        );
    }
}

fn handle_rename(result: &serde_json::Value, queue: &mut EventQueue) {
    let count = requests::apply_workspace_edit(result);
    let msg = format!("Renamed in {count} location(s)");
    queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
}

fn handle_code_action(result: &serde_json::Value, queue: &mut EventQueue) {
    let actions = requests::parse_code_actions(result);
    if !actions.is_empty() {
        let text = actions.join("\n");
        queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(text)));
    }
}

fn handle_jdt(result: &serde_json::Value, queue: &mut EventQueue, line: u32, character: u32) {
    if let Some(content) = result.as_str() {
        let msg = format!("[decompiled]:{}:{}\n{}", line + 1, character + 1, content);
        queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
    } else {
        let msg = "[Source not available]".to_string();
        queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
    }
}

pub(super) fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}

/// Read a single line from a file (0-indexed). Returns trimmed content or empty string.
fn read_line_from_file(path: &Path, line: u32) -> String {
    use std::io::BufRead;
    let Ok(file) = std::fs::File::open(path) else {
        return String::new();
    };
    std::io::BufReader::new(file)
        .lines()
        .nth(line as usize)
        .and_then(|l| l.ok())
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}
