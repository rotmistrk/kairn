//! LSP request senders — dispatch commands to language servers.

use std::path::{Path, PathBuf};

use txv_core::program::CommandContext;

use crate::commands::{CM_LSP_FIND_REFS, CM_LSP_FORMAT, CM_LSP_GOTO_DEF};
use crate::handler::AppState;

use super::pending::{JdtRequest, PendingKind};
use super::{protocol, requests, send_helpers};

pub(super) use super::send_sync::{send_did_change, send_did_open};

pub(super) fn send_goto_def(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);

    if state.lsp.is_initializing(lang) {
        defer(ctx, state, CM_LSP_GOTO_DEF, lang, Box::new((path.clone(), *line, *col)));
        return;
    }

    let Some(client) = state.lsp.get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::goto_definition(client, &uri, *line, *col);
    state
        .lsp_pending
        .insert_with_lang(id, PendingKind::GotoDefinition, lang);
}

pub(super) fn send_goto_show(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp.get_client_mut(lang) else {
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::goto_definition(client, &uri, *line, *col);
    state.lsp_pending.insert_with_lang(id, PendingKind::GotoShow, lang);
}

pub(super) fn send_find_refs(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col, symbol)) = boxed.downcast_ref::<(PathBuf, u32, u32, String)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);

    if state.lsp.is_initializing(lang) {
        defer(
            ctx,
            state,
            CM_LSP_FIND_REFS,
            lang,
            Box::new((path.clone(), *line, *col, symbol.clone())),
        );
        return;
    }

    let Some(client) = state.lsp.get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::find_references(client, &uri, *line, *col);
    state
        .lsp_pending
        .insert(id, PendingKind::FindReferences { symbol: symbol.clone() });
}

pub(super) fn send_hover(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp.get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::hover(client, &uri, *line, *col);
    state.lsp_pending.insert_with_lang(id, PendingKind::Hover, lang);
}

pub(super) fn send_completion(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp.get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::completion(client, &uri, *line, *col);
    state.lsp_pending.insert_with_lang(id, PendingKind::Completion, lang);
}

pub(super) fn send_signature_help(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp.get_client_mut(lang) else {
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::signature_help(client, &uri, *line, *col);
    state.lsp_pending.insert_with_lang(id, PendingKind::SignatureHelp, lang);
}

pub(super) fn send_rename(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(new_name) = boxed.downcast_ref::<String>() else {
        return;
    };

    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    start_lsp(state, &lang, &root);
    let Some(client) = state.lsp.get_client_mut(&lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let (line, col) = state.cursor_pos;
    let id = requests::rename(client, &uri, line, col, new_name);
    state.lsp_pending.insert_with_lang(id, PendingKind::Rename, &lang);
}

pub(super) fn send_code_action(ctx: &mut CommandContext, state: &mut AppState) {
    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    start_lsp(state, &lang, &root);
    let Some(client) = state.lsp.get_client_mut(&lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let (line, col) = state.cursor_pos;
    let id = requests::code_action(client, &uri, line, col);
    state.lsp_pending.insert_with_lang(id, PendingKind::CodeAction, &lang);
}

/// Data: (PathBuf, Option<(u32, u32)>, u32) = (path, optional range, tab_size)
/// If no data provided, uses current file info.
pub(super) fn send_format(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((path, range, tab_size)) = extract_format_params(ctx, state) else {
        return;
    };

    let lang = protocol::language_id(&path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);

    if state.lsp.is_initializing(lang) {
        defer(
            ctx,
            state,
            CM_LSP_FORMAT,
            lang,
            Box::new((path.clone(), range, tab_size)),
        );
        return;
    }

    let Some(client) = state.lsp.get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(&path);
    let id = if let Some((start, end)) = range {
        requests::range_formatting(client, &uri, start, end, tab_size)
    } else {
        requests::formatting(client, &uri, tab_size)
    };
    state.lsp_pending.insert_with_lang(id, PendingKind::Format, lang);
}

type FormatParams = (PathBuf, Option<(u32, u32)>, u32);

fn extract_format_params(ctx: &mut CommandContext, state: &mut AppState) -> Option<FormatParams> {
    if let Some(boxed) = ctx.data.as_ref() {
        boxed
            .downcast_ref::<FormatParams>()
            .map(|(p, r, t)| (p.clone(), *r, *t))
    } else {
        let (_, lang_str) = current_file_info(state);
        if lang_str.is_empty() {
            return None;
        }
        let path = state.broker.last_opened().map(PathBuf::from).unwrap_or_default();
        Some((path, None, 4))
    }
}

pub(super) fn send_jdt_class_contents(jdt: &JdtRequest, state: &mut AppState) {
    let root = state.root_dir.clone();
    start_lsp(state, "java", &root);
    let Some(client) = state.lsp.get_client_mut("java") else {
        return;
    };
    let params = serde_json::json!({ "uri": jdt.uri });
    let id = client.send_request("java/classFileContents", params);
    state.lsp_pending.insert_with_lang(
        id,
        PendingKind::JdtClassContents {
            line: jdt.line,
            character: jdt.character,
        },
        "java",
    );
}

fn defer(
    ctx: &mut CommandContext,
    state: &mut AppState,
    command: txv_core::prelude::CommandId,
    lang: &str,
    data: Box<dyn std::any::Any + Send>,
) {
    send_helpers::defer(ctx, state, command, lang, data);
}

fn emit_last_error(ctx: &mut CommandContext, state: &mut AppState) {
    send_helpers::emit_last_error(ctx, state);
}

fn current_file_info(state: &AppState) -> (String, String) {
    send_helpers::current_file_info(state)
}

/// Fire lsp-start hook (once per language) then call ensure_started.
pub(super) fn start_lsp(state: &mut AppState, lang: &str, root: &Path) {
    send_helpers::start_lsp(state, lang, root);
}
