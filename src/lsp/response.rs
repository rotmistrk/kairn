//! LSP response routing — dispatches parsed responses to the right UI action.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::*;
use crate::views::results::ResultEntry;

use super::pending::{JdtRequest, PendingKind};
use super::requests;
use super::uri;

pub(super) fn handle_response(kind: PendingKind, result: &serde_json::Value, sink: &EventSink) {
    match kind {
        PendingKind::GotoDefinition => handle_goto_def(result, sink),
        PendingKind::GotoShow => handle_goto_show(result, sink),
        PendingKind::FindReferences { symbol } => handle_references(result, sink, &symbol),
        PendingKind::Hover => handle_hover(result, sink),
        PendingKind::Completion => handle_completion(result, sink),
        PendingKind::SignatureHelp => handle_signature_help(result, sink),
        PendingKind::Rename => handle_rename(result, sink),
        PendingKind::CodeAction => handle_code_action(result, sink),
        PendingKind::Format => handle_format(result, sink),
        PendingKind::JdtClassContents { line, character } => handle_jdt(result, sink, line, character),
    }
}

fn handle_goto_def(result: &serde_json::Value, sink: &EventSink) {
    let locs = requests::parse_locations(result);
    if let Some(loc) = locs.into_iter().next() {
        log::info!("LSP: definition -> {}:{}", &loc.uri, loc.line);
        if loc.uri.starts_with("jdt://") {
            sink.push_command(
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
            sink.push_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
        }
    } else {
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No definition found"))),
        );
    }
}

fn handle_goto_show(result: &serde_json::Value, sink: &EventSink) {
    let locs = requests::parse_locations(result);
    if let Some(loc) = locs.into_iter().next() {
        let path = uri_to_path(&loc.uri);
        let req = OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
        sink.push_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
    } else {
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No definition found"))),
        );
    }
}

fn handle_references(result: &serde_json::Value, sink: &EventSink, symbol: &str) {
    let locs = requests::parse_locations(result);
    log::info!("LSP: references -> {} locations", locs.len());
    if locs.len() == 1 {
        let loc = &locs[0];
        let path = uri_to_path(&loc.uri);
        let req = OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
        sink.push_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    } else if !locs.is_empty() {
        let entries: Vec<ResultEntry> = locs
            .iter()
            .map(|l| {
                let path = PathBuf::from(uri_to_path(&l.uri));
                let text = read_line_from_file(&path, l.line);
                ResultEntry {
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
        sink.push_command(CM_SHOW_RESULTS, Some(Box::new((title, entries))));
    } else {
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info(
                "lsp",
                format!("No references found for `{symbol}`"),
            ))),
        );
    }
}

fn handle_hover(result: &serde_json::Value, sink: &EventSink) {
    if let Some(text) = requests::parse_hover(result) {
        log::info!("LSP: hover -> {} chars", text.len());
        sink.push_command(CM_DIAGNOSTIC, Some(Box::new(("hover".to_string(), text))));
    }
}

fn handle_completion(result: &serde_json::Value, sink: &EventSink) {
    let items = requests::parse_completion(result);
    log::info!("LSP: completion -> {} items", items.len());
    if !items.is_empty() {
        sink.push_command(CM_LSP_COMPLETION, Some(Box::new(items)));
    } else {
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "No completions"))),
        );
    }
}

fn handle_signature_help(result: &serde_json::Value, sink: &EventSink) {
    if let Some(sig) = requests::parse_signature_help(result) {
        sink.push_command(CM_LSP_SIGNATURE_HELP, Some(Box::new(sig)));
    }
}

fn handle_rename(result: &serde_json::Value, sink: &EventSink) {
    let count = requests::apply_workspace_edit(result);
    let msg = format!("Renamed in {count} location(s)");
    sink.push_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
}

fn handle_code_action(result: &serde_json::Value, sink: &EventSink) {
    let actions = requests::parse_code_actions(result);
    if !actions.is_empty() {
        let text = actions.join("\n");
        sink.push_command(CM_SHELL_OUTPUT, Some(Box::new(text)));
    }
}

fn handle_format(result: &serde_json::Value, sink: &EventSink) {
    // Result is either null (no edits) or an array of TextEdits
    if result.is_null() {
        use txv_core::message::Message;
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "Already formatted".to_string()))),
        );
        return;
    }
    let Some(edits) = result.as_array() else {
        return;
    };
    if edits.is_empty() {
        use txv_core::message::Message;
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", "Already formatted".to_string()))),
        );
        return;
    }
    // Send the raw edits array to the editor for in-buffer application
    sink.push_command(CM_LSP_FORMAT_RESULT, Some(Box::new(result.clone())));
}

fn handle_jdt(result: &serde_json::Value, sink: &EventSink, line: u32, character: u32) {
    if let Some(content) = result.as_str() {
        let msg = format!("[decompiled]:{}:{}\n{}", line + 1, character + 1, content);
        sink.push_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
    } else {
        let msg = "[Source not available]".to_string();
        sink.push_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
    }
}

pub(super) fn uri_to_path(uri: &str) -> String {
    uri::uri_to_path(uri)
}

/// Read a single line from a file (0-indexed). Returns trimmed content or empty string.
fn read_line_from_file(path: &Path, line: u32) -> String {
    let Ok(file) = File::open(path) else {
        return String::new();
    };
    BufReader::new(file)
        .lines()
        .nth(line as usize)
        .and_then(|l| l.ok())
        .map(|l| l.trim().to_string())
        .unwrap_or_default()
}
