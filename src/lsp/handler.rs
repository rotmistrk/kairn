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

    // Track Starting servers in progress display
    for lang in state.lsp.starting_languages() {
        if state.lsp_status.get(&lang).is_none() {
            state
                .lsp_status
                .set_state(&lang, super::progress::LspServerState::Starting);
        }
    }

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
        // Send didOpen for ALL open files of this language (includes session-restored)
        let paths: Vec<std::path::PathBuf> = state
            .broker
            .open_paths()
            .iter()
            .map(|p| std::path::PathBuf::from(*p))
            .filter(|p| super::protocol::language_id(p) == lang.as_str())
            .collect();
        for path in &paths {
            let key = path.to_string_lossy().to_string();
            if state.lsp_opened_files.contains(&key) {
                continue;
            }
            if let Some(client) = state.lsp.get_client_mut(lang) {
                let uri = super::protocol::path_to_uri(path);
                let lid = super::protocol::language_id(path);
                let text = std::fs::read_to_string(path).unwrap_or_default();
                super::protocol::did_open(client, &uri, lid, &text);
                state.lsp_opened_files.insert(key);
            }
        }
        // Drop pending_opens for this language (already covered above)
        state.lsp.pending_opens.retain(|(l, _)| l != lang);
        // Drop deferred requests — server is ready but still indexing, user can retry
        let had_deferred = state.deferred_lsp.iter().any(|r| r.language == *lang);
        state.deferred_lsp.retain(|r| r.language != *lang);
        if had_deferred {
            sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::info(
                    "lsp",
                    format!("LSP ready ({lang}) — retry when ✓ appears"),
                ))),
            );
        } else {
            sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::info("lsp", format!("LSP ready: {lang}")))),
            );
        }
    }
    if !ready_langs.is_empty() {
        let snapshot = state.lsp_status.snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }

    // Poll messages from all servers
    for (lang, msg) in state.lsp.poll_all() {
        match msg {
            LspMessage::ServerRequest { id, method } => {
                log::debug!("LSP server request: {method} (id={id})");
                // Respond with null result (acknowledges window/workDoneProgress/create etc.)
                if let Some(client) = state.lsp.get_client_any_mut(&lang) {
                    let resp = super::messages::encode_response(id);
                    client.send_raw(resp);
                }
            }
            LspMessage::Notification { method, params } => {
                log::debug!("LSP notification: {method}");
                match method.as_str() {
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
                }
            }
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
