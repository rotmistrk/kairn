//! LSP command handler — wires LSP events into the editor.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

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
    map: HashMap<u64, (PendingKind, Instant)>,
}

#[derive(Debug, Clone)]
pub(crate) enum PendingKind {
    GotoDefinition,
    FindReferences { symbol: String },
    Hover,
    Completion,
    Rename,
    CodeAction,
    JdtClassContents { line: u32, character: u32 },
}

impl PendingRequests {
    pub(crate) fn insert(&mut self, id: u64, kind: PendingKind) {
        self.map.insert(id, (kind, Instant::now()));
    }

    pub(crate) fn take(&mut self, id: u64) -> Option<PendingKind> {
        self.map.remove(&id).map(|(k, _)| k)
    }

    pub(crate) fn remove_timed_out(&mut self) {
        let timeout = std::time::Duration::from_secs(10);
        let expired: Vec<u64> = self
            .map
            .iter()
            .filter(|(_, (_, t))| t.elapsed() > timeout)
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            if let Some((kind, _)) = self.map.remove(&id) {
                log::warn!("LSP timeout: {kind:?} request (id={id}) got no response after 10s");
            }
        }
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
    log::debug!("LSP handler: cmd={}", ctx.command);
    match ctx.command {
        CM_OPEN_FILE | CM_OPEN_FILE_FOCUS => send::send_did_open(ctx, state),
        CM_CONTENT_CHANGED => send::send_did_change(ctx, state),
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
    state.lsp_pending.remove_timed_out();
    for (_lang, msg) in state.lsp.poll_all() {
        log::trace!("LSP poll: {:?}", &msg);
        match msg {
            LspMessage::Notification { method, params } => {
                if method == "textDocument/publishDiagnostics" {
                    if let Some((uri, diags)) = diagnostics::parse_publish_diagnostics(&params) {
                        queue.put_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
                    }
                }
            }
            LspMessage::Response { id, result, error } => {
                if let Some(kind) = state.lsp_pending.take(id) {
                    log::info!("LSP response: {kind:?} (id={id})");
                    if let Some(result) = result {
                        handle_response(kind, &result, queue);
                    } else if let Some(err) = error {
                        let msg = format!("{kind:?}: {}", err.message);
                        log::error!("LSP error: {msg}");
                        queue.put_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::error("lsp", msg))),
                        );
                    }
                } else {
                    log::warn!("LSP response id={id} doesn't match any pending request");
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
                    let req = crate::commands::OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
                    queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
                }
            }
        }
        PendingKind::FindReferences { symbol } => {
            let locs = requests::parse_locations(result);
            log::info!("LSP: references -> {} locations", locs.len());
            if locs.len() == 1 {
                let loc = &locs[0];
                let path = uri_to_path(&loc.uri);
                let req = crate::commands::OpenFileRequest::at(PathBuf::from(&path), loc.line, loc.character);
                queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
            } else if !locs.is_empty() {
                let entries: Vec<crate::views::results::ResultEntry> = locs
                    .iter()
                    .map(|l| crate::views::results::ResultEntry {
                        path: std::path::PathBuf::from(uri_to_path(&l.uri)),
                        line: l.line,
                        col: l.character,
                        text: String::new(),
                    })
                    .collect();
                let title = if symbol.is_empty() {
                    "References".to_string()
                } else {
                    format!("References: {symbol}")
                };
                queue.put_command(CM_SHOW_RESULTS, Some(Box::new((title, entries))));
            }
        }
        PendingKind::Hover => {
            if let Some(text) = requests::parse_hover(result) {
                log::info!("LSP: hover -> {} chars", text.len());
                queue.put_command(CM_DIAGNOSTIC, Some(Box::new(("hover".to_string(), text))));
            }
        }
        PendingKind::Completion => {
            let items = requests::parse_completion(result);
            log::info!("LSP: completion -> {} items", items.len());
            if !items.is_empty() {
                queue.put_command(CM_LSP_COMPLETION, Some(Box::new(items)));
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
