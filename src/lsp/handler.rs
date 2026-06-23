//! LSP command handler — wires LSP events into the editor.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics::parse_publish_diagnostics;
use super::messages::{encode_response, LspMessage};
use super::pending::JdtRequest;
use super::progress::LspServerState;
use super::protocol::{did_open, language_id, path_to_uri};
use super::response::handle_response;
use super::send;

use super::handler_lifecycle;

pub use super::pending::PendingRequests;

/// Handle LSP-related commands. Called before main dispatch.
pub fn handle_lsp_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command() {
        CM_OPEN_FILE | CM_OPEN_FILE_FOCUS => send::send_did_open(ctx, state),
        CM_CONTENT_CHANGED => send::send_did_change(ctx, state),
        CM_LSP_GOTO_DEF => {
            if let Some(boxed) = ctx.data().as_ref() {
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
        CM_LSP_SIGNATURE_HELP => send::send_signature_help(ctx, state),
        CM_LSP_RENAME => send::send_rename(ctx, state),
        CM_CODE_ACTION => send::send_code_action(ctx, state),
        CM_LSP_FORMAT => send::send_format(ctx, state),
        _ => {}
    }
}

/// Poll all LSP servers and dispatch notifications/responses.
pub fn poll_lsp(state: &mut AppState, sink: &EventSink) {
    state.lsp_sub_mut().remove_timed_out(sink);
    poll_track_starting(state, sink);
    poll_expire_deferred(state, sink);
    poll_advance_warming_up(state, sink);
    poll_messages(state, sink);
    handler_lifecycle::poll_detect_dead(state, sink);
}

fn poll_track_starting(state: &mut AppState, sink: &EventSink) {
    for lang in state.lsp_sub_mut().registry_mut().starting_languages() {
        if state.lsp_sub_mut().state_mut().status_mut().get(&lang).is_none() {
            state
                .lsp_sub_mut()
                .state_mut()
                .status_mut()
                .set_state(&lang, LspServerState::Starting);
        }
    }
    if state.lsp_sub_mut().state_mut().status_mut().has_starting() {
        let snapshot = state.lsp_sub_mut().state_mut().status_mut().snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }
}

fn poll_expire_deferred(state: &mut AppState, sink: &EventSink) {
    let timeout = Duration::from_secs(10);
    state.lsp_sub_mut().state_mut().deferred_mut().retain(|r| {
        if r.created.elapsed() > timeout {
            let msg = Message::error("lsp", "LSP not ready \u{2014} request timed out");
            sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            false
        } else {
            true
        }
    });
}

fn poll_advance_warming_up(state: &mut AppState, sink: &EventSink) {
    let ready_langs = state.lsp_sub_mut().registry_mut().advance_warming_up();
    for lang in &ready_langs {
        send_pending_did_opens(state, lang);
        state
            .lsp_sub_mut()
            .registry_mut()
            .pending_opens_mut()
            .retain(|(l, _)| l != lang);
        let had_deferred = state
            .lsp_sub_mut()
            .state_mut()
            .deferred_mut()
            .iter()
            .any(|r| r.language == *lang);
        state
            .lsp_sub_mut()
            .state_mut()
            .deferred_mut()
            .retain(|r| r.language != *lang);
        let msg = if had_deferred {
            Message::info("lsp", format!("LSP ready ({lang}) — retry when ✓ appears"))
        } else {
            Message::info("lsp", format!("LSP ready: {lang}"))
        };
        sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    if !ready_langs.is_empty() {
        let snapshot = state.lsp_sub_mut().state_mut().status_mut().snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }
}

fn send_pending_did_opens(state: &mut AppState, lang: &str) {
    let paths: Vec<PathBuf> = state
        .workspace()
        .broker()
        .open_paths()
        .iter()
        .map(|p| PathBuf::from(*p))
        .filter(|p| language_id(p) == lang)
        .collect();
    for path in &paths {
        let key = path.to_string_lossy().to_string();
        if state.lsp_sub_mut().state_mut().opened_files_mut().contains(&key) {
            continue;
        }
        if let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) {
            let uri = path_to_uri(path);
            let lid = language_id(path);
            let text = fs::read_to_string(path).unwrap_or_default();
            did_open(client, &uri, lid, &text);
            state.lsp_sub_mut().state_mut().opened_files_mut().insert(key);
        }
    }
}

fn poll_messages(state: &mut AppState, sink: &EventSink) {
    for (lang, msg) in state.lsp_sub_mut().registry_mut().poll_all() {
        match msg {
            LspMessage::ServerRequest { id, method } => {
                log::debug!("LSP server request: {method} (id={id})");
                if let Some(client) = state.lsp_sub_mut().registry_mut().get_client_any_mut(&lang) {
                    client.send_raw(encode_response(id));
                }
            }
            LspMessage::Notification { method, params } => {
                handle_notification(state, sink, &lang, &method, &params);
            }
            LspMessage::Response { id, result, error } => {
                handle_lsp_response(state, sink, &lang, id, result, error);
            }
        }
    }
}

fn handle_notification(state: &mut AppState, sink: &EventSink, lang: &str, method: &str, params: &serde_json::Value) {
    log::debug!("LSP notification: {method}");
    match method {
        "textDocument/publishDiagnostics" => {
            if let Some((uri, diags)) = parse_publish_diagnostics(params) {
                sink.push_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
            }
        }
        "$/progress" => {
            if state
                .lsp_sub_mut()
                .state_mut()
                .status_mut()
                .handle_progress(lang, params)
            {
                let snapshot = state.lsp_sub_mut().state_mut().status_mut().snapshot();
                sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
            }
        }
        _ => {}
    }
}

fn handle_lsp_response(
    state: &mut AppState,
    sink: &EventSink,
    _lang: &str,
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<super::messages::RpcError>,
) {
    if let Some(ready_lang) = state.lsp_sub_mut().registry_mut().is_init_response(id) {
        handler_lifecycle::handle_init_response(state, sink, &ready_lang, error);
        return;
    }
    if let Some(kind) = state.lsp_sub_mut().pending_mut().take(id) {
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
    }
}
