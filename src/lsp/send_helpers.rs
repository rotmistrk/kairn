//! Shared helpers for LSP send modules.

use std::path::Path;
use std::time::Instant;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::deferred_lsp_request::DeferredLspRequest;
use crate::handler::AppState;
use crate::handler_script_util::fire_lsp_start_hook;

use super::protocol;

pub(super) fn defer(
    ctx: &mut CommandContext,
    state: &mut AppState,
    command: CommandId,
    lang: &str,
    data: Box<dyn std::any::Any + Send>,
) {
    use txv_core::message::{Message, MsgLevel};
    ctx.sink().push_command(
        txv_widgets::CM_STATUS_MESSAGE,
        Some(Box::new(Message::new(
            MsgLevel::Info,
            "lsp",
            format!("Waiting for LSP ({lang})..."),
        ))),
    );
    state.lsp_sub_mut().state_mut().deferred_mut().push(DeferredLspRequest {
        command,
        data,
        language: lang.to_string(),
        created: Instant::now(),
    });
}

pub(super) fn emit_last_error(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(err) = state.lsp_sub_mut().registry_mut().take_last_error() {
        use txv_core::message::{Message, MsgLevel};
        ctx.sink().push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
        );
    }
}

pub(super) fn current_file_info(state: &AppState) -> (String, String) {
    if let Some(path) = state.workspace().broker().last_opened() {
        let p = Path::new(path);
        let uri = protocol::path_to_uri(p);
        let lang = protocol::language_id(p).to_string();
        (uri, lang)
    } else {
        (String::new(), String::new())
    }
}

/// Fire lsp-start hook (once per language) then call ensure_started.
pub(super) fn start_lsp(state: &mut AppState, lang: &str, root: &Path) {
    if state.lsp_sub_mut().registry_mut().take_start_hook(lang) {
        fire_lsp_start_hook(state, lang);
    }
    state.lsp_sub_mut().registry_mut().ensure_started(lang, root);
}
