//! LSP command handler — wires LSP events into the editor.

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics;
use super::messages::LspMessage;
use super::requests;
use super::send;

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
    Completion,
    Rename,
    CodeAction,
    JdtClassContents { line: u32, character: u32 },
}

impl PendingRequests {
    pub(crate) fn insert(&mut self, id: u64, kind: PendingKind) {
        self.map.insert(id, kind);
    }

    pub(crate) fn take(&mut self, id: u64) -> Option<PendingKind> {
        self.map.remove(&id)
    }
}

/// Request for jdt:// class file contents from jdtls.
#[derive(Debug, Clone)]
pub(crate) struct JdtRequest {
    pub uri: String,
    pub line: u32,
    pub character: u32,
}

/// Handle LSP-related commands. Called before main dispatch.
pub fn handle_lsp_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command {
        CM_OPEN_FILE | CM_OPEN_FILE_FOCUS => send::send_did_open(ctx, state),
        CM_LSP_GOTO_DEF => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(jdt) = boxed.downcast_ref::<JdtRequest>() {
                    send::send_jdt_class_contents(jdt, state);
                    return;
                }
            }
            send::send_goto_def(ctx, state);
        }
        CM_LSP_FIND_REFS => send::send_find_refs(ctx, state),
        CM_LSP_HOVER => send::send_hover(ctx, state),
        CM_LSP_COMPLETION => send::send_completion(ctx, state),
        CM_LSP_RENAME => send::send_rename(ctx, state),
        CM_CODE_ACTION => send::send_code_action(ctx, state),
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
                    let req = crate::commands::OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
                    queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
                }
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
        PendingKind::Completion => {
            let items = requests::parse_completion(result);
            if !items.is_empty() {
                let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
                queue.put_command(CM_LSP_COMPLETION, Some(Box::new(labels)));
            }
        }
        PendingKind::Rename => {
            let count = requests::apply_workspace_edit(result);
            let msg = format!("Renamed in {count} location(s)");
            queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
        }
        PendingKind::CodeAction => {
            let actions = requests::parse_code_actions(result);
            if !actions.is_empty() {
                let text = actions.join("\n");
                queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(text)));
            }
        }
        PendingKind::JdtClassContents { line, character } => {
            if let Some(content) = result.as_str() {
                let msg = format!("[decompiled]:{}:{}\n{}", line + 1, character + 1, content);
                queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
            } else {
                let msg = "[Source not available]".to_string();
                queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(msg)));
            }
        }
    }
}

fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}
