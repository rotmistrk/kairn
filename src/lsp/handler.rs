//! LSP command handler — wires LSP events into the editor.

use std::collections::HashMap;
use std::time::Instant;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics;
use super::messages::LspMessage;
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

    pub(crate) fn remove_timed_out(&mut self, queue: &mut EventQueue) {
        let timeout = std::time::Duration::from_secs(10);
        let expired: Vec<u64> = self
            .map
            .iter()
            .filter(|(_, (_, t))| t.elapsed() > timeout)
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            if let Some((kind, _)) = self.map.remove(&id) {
                let msg = format!("{kind:?}: no response after 10s");
                log::warn!("LSP timeout: {msg}");
                queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::error("lsp", msg))),
                );
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
    state.lsp_pending.remove_timed_out(queue);
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
                // Check if this is an initialize response
                if let Some(lang) = state.lsp.pending_init.remove(&id) {
                    if let Some(err) = error {
                        let msg = format!("LSP init failed for {lang}: {}", err.message);
                        log::error!("{msg}");
                        queue.put_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::error("lsp", msg))),
                        );
                    } else {
                        log::info!("LSP initialized for {lang}");
                        if let Some(client) = state.lsp.get_client_mut(&lang) {
                            super::protocol::initialized(client);
                        }
                        queue.put_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::info("lsp", format!("LSP ready: {lang}")))),
                        );
                    }
                    continue;
                }
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
    super::response::handle_response(kind, result, queue);
}

fn uri_to_path(uri: &str) -> String {
    super::response::uri_to_path(uri)
}
