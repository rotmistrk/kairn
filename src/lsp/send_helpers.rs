//! Shared helpers for LSP send modules.

use std::path::{Path, PathBuf};
use std::time::Instant;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::deferred_lsp_request::DeferredLspRequest;
use crate::handler::AppState;
use crate::handler_script_util::fire_lsp_start_hook;
use crate::lsp::client::LspClient;

use super::pending::PendingKind;
use super::protocol;

/// Builder function type: given client, URI, line, col, returns request id.
type PositionBuilder = fn(&mut LspClient, &str, u32, u32) -> u64;

/// Send a position-based LSP request (no defer on initialization).
///
/// Extracts `(PathBuf, u32, u32)` from command data, starts LSP,
/// gets client, calls `builder`, registers `kind` as pending.
/// If client is unavailable, silently returns (no error shown).
pub(super) fn send_position_request_silent(
    ctx: &mut CommandContext,
    state: &mut AppState,
    kind: PendingKind,
    builder: PositionBuilder,
) {
    let Some((path, line, col)) = extract_position(ctx) else {
        return;
    };
    let lang = protocol::language_id(&path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        return;
    };
    let uri = protocol::path_to_uri(&path);
    let id = builder(client, &uri, line, col);
    state.lsp_sub_mut().pending_mut().insert_with_lang(id, kind, lang);
}

/// Send a position-based LSP request, showing last error if client unavailable.
///
/// Same as `send_position_request_silent` but calls `emit_last_error`
/// when no client is found.
pub(super) fn send_position_request(
    ctx: &mut CommandContext,
    state: &mut AppState,
    kind: PendingKind,
    builder: PositionBuilder,
) {
    let Some((path, line, col)) = extract_position(ctx) else {
        return;
    };
    let lang = protocol::language_id(&path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(&path);
    let id = builder(client, &uri, line, col);
    state.lsp_sub_mut().pending_mut().insert_with_lang(id, kind, lang);
}

/// Send a position-based LSP request that defers if the server is still initializing.
pub(super) fn send_position_request_deferred(
    ctx: &mut CommandContext,
    state: &mut AppState,
    kind: PendingKind,
    builder: PositionBuilder,
    defer_cmd: CommandId,
) {
    let Some((path, line, col)) = extract_position(ctx) else {
        return;
    };
    let lang = protocol::language_id(&path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);

    if state.lsp_sub_mut().registry_mut().is_initializing(lang) {
        defer(ctx, state, defer_cmd, lang, Box::new((path, line, col)));
        return;
    }
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(&path);
    let id = builder(client, &uri, line, col);
    state.lsp.pending_mut().insert_with_lang(id, kind, lang);
}

/// Extract `(PathBuf, u32, u32)` from command context data.
fn extract_position(ctx: &mut CommandContext) -> Option<(PathBuf, u32, u32)> {
    let boxed = ctx.data().as_ref()?;
    boxed
        .downcast_ref::<(PathBuf, u32, u32)>()
        .map(|(p, l, c)| (p.clone(), *l, *c))
}

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
