//! LSP lifecycle handling — initialization response and dead server detection.

use txv_core::prelude::*;

use crate::commands::*;
use crate::handler::AppState;

use super::progress::LspServerState;

pub(super) fn handle_init_response(
    state: &mut AppState,
    sink: &EventSink,
    ready_lang: &str,
    error: Option<super::messages::RpcError>,
) {
    if let Some(err) = error {
        handle_init_error(state, sink, ready_lang, &err.message);
    } else {
        handle_init_success(state, sink, ready_lang);
    }
    let snapshot = state.lsp_sub_mut().state_mut().status_mut().snapshot();
    sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
}

fn handle_init_error(state: &mut AppState, sink: &EventSink, lang: &str, err_msg: &str) {
    state.lsp_sub_mut().registry_mut().fail_init(lang);
    let msg = format!("LSP init failed for {lang}: {err_msg}");
    log::error!("{msg}");
    state
        .lsp_sub_mut()
        .state_mut()
        .status_mut()
        .set_state(lang, LspServerState::Error);
    sink.push_command(
        txv_widgets::CM_STATUS_MESSAGE,
        Some(Box::new(Message::error("lsp", msg))),
    );
    state
        .lsp_sub_mut()
        .state_mut()
        .deferred_mut()
        .retain(|r| r.language != lang);
    state
        .lsp_sub_mut()
        .registry_mut()
        .pending_opens_mut()
        .retain(|(l, _)| l != lang);
}

fn handle_init_success(state: &mut AppState, _sink: &EventSink, lang: &str) {
    state.lsp_sub_mut().registry_mut().complete_init(lang);
    log::info!("LSP initialized for {lang}");
    state.lsp_sub_mut().state_mut().status_mut().set_state(
        lang,
        LspServerState::Indexing {
            percent: None,
            message: None,
        },
    );
}

pub(super) fn poll_detect_dead(state: &mut AppState, sink: &EventSink) {
    let dead_langs = state.lsp_sub_mut().registry_mut().detect_dead();
    for lang in &dead_langs {
        state
            .lsp_sub_mut()
            .state_mut()
            .status_mut()
            .set_state(lang, LspServerState::Error);
        let msg = Message::error(
            "lsp",
            format!("LSP server for {lang} died \u{2014} disabled until restart"),
        );
        sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        state
            .lsp_sub_mut()
            .state_mut()
            .deferred_mut()
            .retain(|r| r.language != *lang);
        state
            .lsp_sub_mut()
            .registry_mut()
            .pending_opens_mut()
            .retain(|(l, _)| l != lang);
    }
    if !dead_langs.is_empty() {
        let snapshot = state.lsp_sub_mut().state_mut().status_mut().snapshot();
        sink.push_command(CM_LSP_STATUS_UPDATE, Some(Box::new(snapshot)));
    }
}
