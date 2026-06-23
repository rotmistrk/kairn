//! LSP request senders — dispatch commands to language servers.

use std::path::{Path, PathBuf};

use txv_core::program::CommandContext;

use crate::commands::{CM_LSP_FIND_REFS, CM_LSP_GOTO_DEF};
use crate::handler::AppState;

use super::pending::PendingKind;
use super::{protocol, requests, send_helpers};

pub(super) use super::send_sync::{send_did_change, send_did_open};

pub(super) fn send_goto_def(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);

    if state.lsp_sub_mut().registry_mut().is_initializing(lang) {
        defer(ctx, state, CM_LSP_GOTO_DEF, lang, Box::new((path.clone(), *line, *col)));
        return;
    }

    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::goto_definition(client, &uri, *line, *col);
    state
        .lsp
        .pending_mut()
        .insert_with_lang(id, PendingKind::GotoDefinition, lang);
}

pub(super) fn send_goto_show(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };
    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::goto_definition(client, &uri, *line, *col);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::GotoShow, lang);
}

pub(super) fn send_find_refs(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col, symbol)) = boxed.downcast_ref::<(PathBuf, u32, u32, String)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);

    if state.lsp_sub_mut().registry_mut().is_initializing(lang) {
        defer(
            ctx,
            state,
            CM_LSP_FIND_REFS,
            lang,
            Box::new((path.clone(), *line, *col, symbol.clone())),
        );
        return;
    }

    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::find_references(client, &uri, *line, *col);
    state
        .lsp
        .pending_mut()
        .insert(id, PendingKind::FindReferences { symbol: symbol.clone() });
}

pub(super) fn send_hover(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::hover(client, &uri, *line, *col);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::Hover, lang);
}

pub(super) fn send_completion(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::completion(client, &uri, *line, *col);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::Completion, lang);
}

pub(super) fn send_signature_help(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };
    let lang = protocol::language_id(path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        return;
    };
    let uri = protocol::path_to_uri(path);
    let id = requests::signature_help(client, &uri, *line, *col);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::SignatureHelp, lang);
}

pub(super) fn send_rename(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(new_name) = boxed.downcast_ref::<String>() else {
        return;
    };

    let (uri, lang) = current_file_info(state);
    let root = state.root_dir().clone();
    start_lsp(state, &lang, &root);
    let (line, col) = state.cursor_pos;
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(&lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let id = requests::rename(client, &uri, line, col, new_name);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::Rename, &lang);
}

pub(super) fn send_code_action(ctx: &mut CommandContext, state: &mut AppState) {
    let (uri, lang) = current_file_info(state);
    let root = state.root_dir().clone();
    start_lsp(state, &lang, &root);
    let (line, col) = state.cursor_pos;
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(&lang) else {
        emit_last_error(ctx, state);
        return;
    };
    let id = requests::code_action(client, &uri, line, col);
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::CodeAction, &lang);
}

pub(super) use super::send_format::{send_format, send_jdt_class_contents};

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
