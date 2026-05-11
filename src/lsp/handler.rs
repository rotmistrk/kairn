//! LSP command handler — wires LSP events into the editor.

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics;
use super::messages::LspMessage;
use super::protocol;
use super::requests;

/// Tracks pending LSP requests so responses can be routed.
#[derive(Default)]
pub struct PendingRequests {
    map: HashMap<u64, PendingKind>,
}

#[derive(Debug, Clone)]
pub(crate) enum PendingKind {
    GotoDefinition,
    FindReferences,
    Hover,
}

impl PendingRequests {
    pub(crate) fn insert(&mut self, id: u64, kind: PendingKind) {
        self.map.insert(id, kind);
    }

    pub(crate) fn take(&mut self, id: u64) -> Option<PendingKind> {
        self.map.remove(&id)
    }
}

/// Handle LSP-related commands. Called before main dispatch.
pub fn handle_lsp_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command {
        CM_OPEN_FILE | CM_OPEN_FILE_FOCUS => send_did_open(ctx, state),
        CM_LSP_GOTO_DEF => send_goto_def(ctx, state),
        CM_LSP_FIND_REFS => send_find_refs(ctx, state),
        CM_LSP_HOVER => send_hover(ctx, state),
        _ => {}
    }
}

/// Poll all LSP servers and dispatch notifications/responses.
pub fn poll_lsp(state: &mut AppState, queue: &mut EventQueue) {
    for (_lang, msg) in state.lsp.poll_all() {
        match msg {
            LspMessage::Notification { method, params } => {
                if method == "textDocument/publishDiagnostics" {
                    if let Some((uri, diags)) = diagnostics::parse_publish_diagnostics(&params) {
                        queue.put_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
                    }
                }
            }
            LspMessage::Response { id, result, .. } => {
                if let Some(kind) = state.lsp_pending.take(id) {
                    if let Some(result) = result {
                        handle_response(kind, &result, queue);
                    }
                }
            }
        }
    }
}

fn handle_response(kind: PendingKind, result: &serde_json::Value, queue: &mut EventQueue) {
    match kind {
        PendingKind::GotoDefinition => {
            let locs = requests::parse_locations(result);
            if let Some(loc) = locs.into_iter().next() {
                let path = uri_to_path(&loc.uri);
                queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(PathBuf::from(&path))));
                // TODO: jump to line/col after open
            }
        }
        PendingKind::FindReferences => {
            let locs = requests::parse_locations(result);
            let text = locs
                .iter()
                .map(|l| format!("{}:{}:{}", uri_to_path(&l.uri), l.line + 1, l.character + 1))
                .collect::<Vec<_>>()
                .join("\n");
            if !text.is_empty() {
                queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(text)));
            }
        }
        PendingKind::Hover => {
            if let Some(text) = requests::parse_hover(result) {
                queue.put_command(CM_DIAGNOSTIC, Some(Box::new(("hover".to_string(), text))));
            }
        }
    }
}

fn send_did_open(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(path) = boxed.downcast_ref::<PathBuf>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        return;
    };

    let uri = protocol::path_to_uri(path);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    protocol::did_open(client, &uri, lang, &text);
}

fn send_goto_def(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() else {
        return;
    };

    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(&lang, &root) else {
        return;
    };

    let id = requests::goto_definition(client, &uri, line, col);
    state.lsp_pending.insert(id, PendingKind::GotoDefinition);
}

fn send_find_refs(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() else {
        return;
    };

    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(&lang, &root) else {
        return;
    };

    let id = requests::find_references(client, &uri, line, col);
    state.lsp_pending.insert(id, PendingKind::FindReferences);
}

fn send_hover(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() else {
        return;
    };

    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(&lang, &root) else {
        return;
    };

    let id = requests::hover(client, &uri, line, col);
    state.lsp_pending.insert(id, PendingKind::Hover);
}

fn current_file_info(state: &AppState) -> (String, String) {
    // Use the last opened file from broker as current context
    if let Some(path) = state.broker.last_opened() {
        let p = std::path::Path::new(path);
        let uri = protocol::path_to_uri(p);
        let lang = protocol::language_id(p).to_string();
        (uri, lang)
    } else {
        (String::new(), String::new())
    }
}

fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}
