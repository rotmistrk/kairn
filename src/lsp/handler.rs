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
pub struct PendingRequests {
    map: HashMap<u64, (PendingKind, String, Instant)>,
    pub timeout_secs: u64,
}

impl Default for PendingRequests {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PendingKind {
    GotoDefinition,
    GotoShow,
    FindReferences { symbol: String },
    Hover,
    Completion,
    Rename,
    CodeAction,
    JdtClassContents { line: u32, character: u32 },
}

impl PendingRequests {
    pub(crate) fn insert(&mut self, id: u64, kind: PendingKind) {
        self.insert_with_lang(id, kind, "");
    }

    pub(crate) fn insert_with_lang(&mut self, id: u64, kind: PendingKind, lang: &str) {
        self.map.insert(id, (kind, lang.to_string(), Instant::now()));
    }

    pub(crate) fn take(&mut self, id: u64) -> Option<PendingKind> {
        self.map.remove(&id).map(|(k, _, _)| k)
    }

    pub(crate) fn remove_timed_out(&mut self, sink: &EventSink, registry: &super::registry::LspRegistry) {
        let global_timeout = self.timeout_secs;
        let expired: Vec<u64> = self
            .map
            .iter()
            .filter(|(_, (_, lang, t))| {
                let secs = registry.timeout(lang).unwrap_or(global_timeout);
                t.elapsed() > std::time::Duration::from_secs(secs)
            })
            .map(|(&id, _)| id)
            .collect();
        for id in expired {
            if let Some((kind, lang, _)) = self.map.remove(&id) {
                let secs = registry.timeout(&lang).unwrap_or(global_timeout);
                let label = friendly_kind_label(&kind);
                let msg = format!("{label}: no response after {secs}s");
                log::warn!("LSP timeout: {msg}");
                sink.push_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::error("lsp", msg))),
                );
            }
        }
    }
}

fn friendly_kind_label(kind: &PendingKind) -> &'static str {
    match kind {
        PendingKind::GotoDefinition => "Go to definition",
        PendingKind::GotoShow => "Go to definition",
        PendingKind::FindReferences { .. } => "Find references",
        PendingKind::Hover => "Hover",
        PendingKind::Completion => "Completion",
        PendingKind::Rename => "Rename",
        PendingKind::CodeAction => "Code action",
        PendingKind::JdtClassContents { .. } => "Class contents",
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
        CM_LSP_GOTO_SHOW => send::send_goto_show(ctx, state),
        CM_LSP_FIND_REFS => send::send_find_refs(ctx, state),
        CM_LSP_HOVER => send::send_hover(ctx, state),
        CM_LSP_COMPLETION => send::send_completion(ctx, state),
        CM_LSP_RENAME => send::send_rename(ctx, state),
        CM_CODE_ACTION => send::send_code_action(ctx, state),
        _ => {}
    }
}

/// Poll all LSP servers and dispatch notifications/responses.
pub fn poll_lsp(state: &mut AppState, sink: &EventSink) {
    state.lsp_pending.remove_timed_out(sink, &state.lsp);

    // Track Starting state for languages in pending_init
    for lang in state.lsp.pending_init.values() {
        use super::progress::LspServerState;
        if state.lsp_status.get(lang).is_none() {
            state.lsp_status.set_state(lang, LspServerState::Starting);
        }
    }
    // Refresh status bar while any server is starting (shows elapsed time)
    if state.lsp_status.has_starting() {
        let snapshot = state.lsp_status.snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }

    // Expire deferred requests older than 10s
    let timeout = std::time::Duration::from_secs(10);
    state.deferred_lsp.retain(|r| {
        if r.created.elapsed() > timeout {
            sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::error("lsp", "LSP not ready — request timed out"))),
            );
            false
        } else {
            true
        }
    });
    for (lang, msg) in state.lsp.poll_all() {
        log::trace!("LSP poll: {:?}", &msg);
        match msg {
            LspMessage::Notification { method, params } => match method.as_str() {
                "textDocument/publishDiagnostics" => {
                    if let Some((uri, diags)) = diagnostics::parse_publish_diagnostics(&params) {
                        sink.push_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
                    }
                }
                "$/progress" => {
                    if state.lsp_status.handle_progress(&lang, &params) {
                        let snapshot = state.lsp_status.snapshot();
                        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
                    }
                }
                _ => {}
            },
            LspMessage::Response { id, result, error } => {
                // Check if this is an initialize response
                if let Some(lang) = state.lsp.pending_init.remove(&id) {
                    if let Some(err) = error {
                        let msg = format!("LSP init failed for {lang}: {}", err.message);
                        log::error!("{msg}");
                        state
                            .lsp_status
                            .set_state(&lang, super::progress::LspServerState::Error);
                        sink.push_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::error("lsp", msg))),
                        );
                        // Drop deferred requests for this language
                        state.deferred_lsp.retain(|r| r.language != lang);
                    } else {
                        log::info!("LSP initialized for {lang}");
                        state
                            .lsp_status
                            .set_state(&lang, super::progress::LspServerState::Ready);
                        if let Some(client) = state.lsp.get_client_mut(&lang) {
                            super::protocol::initialized(client);
                        }
                        sink.push_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::info("lsp", format!("LSP ready: {lang}")))),
                        );
                        // Retry deferred requests for this language
                        let ready_lang = lang.clone();
                        let mut remaining = Vec::new();
                        for req in state.deferred_lsp.drain(..) {
                            if req.created.elapsed() > std::time::Duration::from_secs(10) {
                                sink.push_command(
                                    txv_widgets::CM_STATUS_MESSAGE,
                                    Some(Box::new(Message::error("lsp", "Deferred request timed out"))),
                                );
                            } else if req.language == ready_lang {
                                sink.push_command(req.command, Some(req.data));
                            } else {
                                remaining.push(req);
                            }
                        }
                        state.deferred_lsp = remaining;
                    }
                    let snapshot = state.lsp_status.snapshot();
                    sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
                    continue;
                }
                if let Some(kind) = state.lsp_pending.take(id) {
                    log::info!("LSP response: {kind:?} (id={id})");
                    if let Some(result) = result {
                        handle_response(kind, &result, sink);
                    } else if let Some(err) = error {
                        let msg = format!("{kind:?}: {}", err.message);
                        log::error!("LSP error: {msg}");
                        sink.push_command(
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

fn handle_response(kind: PendingKind, result: &serde_json::Value, sink: &EventSink) {
    super::response::handle_response(kind, result, sink);
}

fn uri_to_path(uri: &str) -> String {
    super::response::uri_to_path(uri)
}
