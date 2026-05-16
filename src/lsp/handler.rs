//! LSP command handler — wires LSP events into the editor.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::messages::LspMessage;
use super::pending::JdtRequest;
use super::send;

pub use super::pending::PendingRequests;

/// Handle LSP-related commands. Called before main dispatch.
pub fn handle_lsp_command(ctx: &mut CommandContext, state: &mut AppState) {
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

    // Refresh status bar while any server is starting
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

    // Advance WarmingUp → Ready (server had a full tick to process 'initialized')
    let ready_langs = state.lsp.advance_warming_up();
    for lang in &ready_langs {
        // Replay pending didOpen
        let mut keep = Vec::new();
        let opens: Vec<_> = state.lsp.pending_opens.drain(..).collect();
        for (l, path) in opens {
            if l == *lang {
                if let Some(client) = state.lsp.get_client_mut(&l) {
                    let uri = super::protocol::path_to_uri(&path);
                    let lid = super::protocol::language_id(&path);
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    super::protocol::did_open(client, &uri, lid, &text);
                }
            } else {
                keep.push((l, path));
            }
        }
        state.lsp.pending_opens = keep;
        // Replay deferred requests
        let mut keep = Vec::new();
        for req in state.deferred_lsp.drain(..) {
            if req.language == *lang {
                sink.push_command(req.command, Some(req.data));
            } else {
                keep.push(req);
            }
        }
        state.deferred_lsp = keep;
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("lsp", format!("LSP ready: {lang}")))),
        );
    }
    if !ready_langs.is_empty() {
        let snapshot = state.lsp_status.snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }

    // Poll messages from all servers
    for (lang, msg) in state.lsp.poll_all() {
        match msg {
            LspMessage::Notification { method, params } => match method.as_str() {
                "textDocument/publishDiagnostics" => {
                    if let Some((uri, diags)) = super::diagnostics::parse_publish_diagnostics(&params) {
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
                if let Some(ready_lang) = state.lsp.is_init_response(id) {
                    if let Some(err) = error {
                        state.lsp.fail_init(&ready_lang);
                        let msg = format!("LSP init failed for {ready_lang}: {}", err.message);
                        log::error!("{msg}");
                        state
                            .lsp_status
                            .set_state(&ready_lang, super::progress::LspServerState::Error);
                        sink.push_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::error("lsp", msg))),
                        );
                        state.deferred_lsp.retain(|r| r.language != ready_lang);
                        state.lsp.pending_opens.retain(|(l, _)| l != &ready_lang);
                    } else {
                        state.lsp.complete_init(&ready_lang);
                        log::info!("LSP initialized for {ready_lang}");
                        state.lsp_status.set_state(
                            &ready_lang,
                            super::progress::LspServerState::Indexing {
                                percent: None,
                                message: None,
                            },
                        );
                    }
                    let snapshot = state.lsp_status.snapshot();
                    sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
                    continue;
                }
                if let Some(kind) = state.lsp_pending.take(id) {
                    if let Some(result) = result {
                        super::response::handle_response(kind, &result, sink);
                    } else if let Some(err) = error {
                        let msg = format!("{kind:?}: {}", err.message);
                        log::error!("LSP error: {msg}");
                        sink.push_command(
                            txv_widgets::CM_STATUS_MESSAGE,
                            Some(Box::new(Message::error("lsp", msg))),
                        );
                    }
                }
            }
        }
    }

    // Detect dead servers
    let dead_langs = state.lsp.detect_dead();
    for lang in &dead_langs {
        state.lsp_status.set_state(lang, super::progress::LspServerState::Error);
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::error(
                "lsp",
                format!("LSP server for {lang} died — disabled until restart"),
            ))),
        );
        state.deferred_lsp.retain(|r| r.language != *lang);
        state.lsp.pending_opens.retain(|(l, _)| l != lang);
    }
    if !dead_langs.is_empty() {
        let snapshot = state.lsp_status.snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }
}
