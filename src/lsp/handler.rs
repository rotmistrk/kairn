//! LSP command handler — wires LSP events into the editor.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics;
use super::messages::LspMessage;
use super::pending::{JdtRequest, PendingKind};
use super::send;

pub use super::pending::PendingRequests;

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
                        state.lsp.pending_opens.retain(|(l, _)| l != &lang);
                    } else {
                        log::info!("LSP initialized for {lang}");
                        state.lsp_status.set_state(
                            &lang,
                            super::progress::LspServerState::Indexing {
                                percent: None,
                                message: None,
                            },
                        );
                        if let Some(client) = state.lsp.get_client_mut(&lang) {
                            super::protocol::initialized(client);
                        }
                        // Replay pending didOpen notifications
                        let mut remaining = Vec::new();
                        let opens: Vec<_> = state.lsp.pending_opens.drain(..).collect();
                        for (l, path) in opens {
                            if l == lang {
                                if let Some(client) = state.lsp.get_client_mut(&l) {
                                    let uri = super::protocol::path_to_uri(&path);
                                    let lid = super::protocol::language_id(&path);
                                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                                    super::protocol::did_open(client, &uri, lid, &text);
                                }
                            } else {
                                remaining.push((l, path));
                            }
                        }
                        state.lsp.pending_opens = remaining;
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
    // Detect servers that died during initialization
    let dead_langs: Vec<String> = state
        .lsp
        .pending_init
        .values()
        .filter(|lang| state.lsp.active.get(lang.as_str()).is_some_and(|c| !c.is_alive()))
        .cloned()
        .collect();
    for lang in &dead_langs {
        state.lsp.pending_init.retain(|_, v| v != lang);
        state.lsp.active.remove(lang.as_str());
        state.lsp_status.set_state(lang, super::progress::LspServerState::Error);
        let msg = format!("LSP server for {lang} died during startup");
        log::error!("{msg}");
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::error("lsp", msg))),
        );
        state.deferred_lsp.retain(|r| r.language != *lang);
    }
    if !dead_langs.is_empty() {
        let snapshot = state.lsp_status.snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }
}

fn handle_response(kind: PendingKind, result: &serde_json::Value, sink: &EventSink) {
    super::response::handle_response(kind, result, sink);
}

fn uri_to_path(uri: &str) -> String {
    super::response::uri_to_path(uri)
}
