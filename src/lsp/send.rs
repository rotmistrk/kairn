//! LSP request senders — dispatch commands to language servers.

use std::path::PathBuf;

use txv_core::program::CommandContext;

use crate::handler::AppState;

use super::handler::{JdtRequest, PendingKind};
use super::{protocol, requests};

pub(super) fn send_did_open(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::OpenFileRequest>() else {
        return;
    };
    let path = &req.path;

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    let uri = protocol::path_to_uri(path);
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("LSP didOpen: cannot read {}: {e}", path.display());
            String::new()
        }
    };
    protocol::did_open(client, &uri, lang, &text);
}

pub(super) fn send_did_change(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(changed) = boxed.downcast_ref::<crate::commands::ContentChanged>() else {
        return;
    };

    let lang = protocol::language_id(&changed.path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    let uri = protocol::path_to_uri(&changed.path);
    let key = changed.path.to_string_lossy().to_string();
    let version = state.doc_versions.entry(key).or_insert(1);
    *version += 1;
    protocol::did_change(client, &uri, *version, &changed.content);
}

pub(super) fn send_goto_def(ctx: &mut CommandContext, state: &mut AppState) {
    log::debug!("send_goto_def called, data={:?}", ctx.data.is_some());
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let uri = protocol::path_to_uri(path);
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    log::info!("LSP: textDocument/definition at {uri}:{line}:{col}");
    let id = requests::goto_definition(client, &uri, *line, *col);
    state.lsp_pending.insert(id, PendingKind::GotoDefinition);
}

pub(super) fn send_find_refs(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col, symbol)) = boxed.downcast_ref::<(PathBuf, u32, u32, String)>() else {
        return;
    };

    let uri = protocol::path_to_uri(path);
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    log::info!("LSP: textDocument/references at {uri}:{line}:{col}");
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

    let uri = protocol::path_to_uri(path);
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    log::info!("LSP: textDocument/hover at {uri}:{line}:{col}");
    let id = requests::hover(client, &uri, *line, *col);
    state.lsp_pending.insert(id, PendingKind::Hover);
}

pub(super) fn send_completion(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((path, line, col)) = boxed.downcast_ref::<(PathBuf, u32, u32)>() else {
        return;
    };

    let uri = protocol::path_to_uri(path);
    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        if let Some(err) = state.lsp.last_error.take() {
            use txv_core::message::{Message, MsgLevel};
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
            );
        }
        return;
    };

    log::info!("LSP: textDocument/completion at {uri}:{line}:{col}");
    let id = requests::completion(client, &uri, *line, *col);
    state.lsp_pending.insert(id, PendingKind::Completion);
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
    let Some(client) = state.lsp.get_or_start(&lang, &root) else {
        return;
    };

    let (line, col) = state.cursor_pos;
    let id = requests::rename(client, &uri, line, col, new_name);
    state.lsp_pending.insert(id, PendingKind::Rename);
}

pub(super) fn send_code_action(ctx: &mut CommandContext, state: &mut AppState) {
    let (uri, lang) = current_file_info(state);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(&lang, &root) else {
        return;
    };

    let (line, col) = state.cursor_pos;
    let id = requests::code_action(client, &uri, line, col);
    state.lsp_pending.insert(id, PendingKind::CodeAction);
    let _ = ctx;
}

pub(super) fn send_jdt_class_contents(jdt: &JdtRequest, state: &mut AppState) {
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start("java", &root) else {
        return;
    };
    let params = serde_json::json!({ "uri": jdt.uri });
    let id = client.send_request("java/classFileContents", params);
    state.lsp_pending.insert(
        id,
        PendingKind::JdtClassContents {
            line: jdt.line,
            character: jdt.character,
        },
    );
}

fn current_file_info(state: &AppState) -> (String, String) {
    if let Some(path) = state.broker.last_opened() {
        let p = std::path::Path::new(path);
        let uri = protocol::path_to_uri(p);
        let lang = protocol::language_id(p).to_string();
        (uri, lang)
    } else {
        (String::new(), String::new())
    }
}
