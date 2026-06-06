//! LSP document sync senders — didOpen, didChange notifications.

use std::fs;

use txv_core::program::CommandContext;

use crate::commands::{ContentChanged, OpenFileRequest};
use crate::handler::AppState;

use super::protocol;
use super::send_helpers::start_lsp;

pub(super) fn send_did_open(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<OpenFileRequest>() else {
        return;
    };
    let path = &req.path;

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);

    if state.lsp.is_initializing(lang) {
        state.lsp.pending_opens.push((lang.to_string(), path.clone()));
        return;
    }

    let Some(client) = state.lsp.get_client_mut(lang) else {
        report_lsp_error(ctx, state);
        return;
    };

    let key = path.to_string_lossy().to_string();
    if state.lsp_opened_files.contains(&key) {
        return;
    }
    state.lsp_opened_files.insert(key);

    let uri = protocol::path_to_uri(path);
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("LSP didOpen: cannot read {}: {e}", path.display());
            String::new()
        }
    };
    protocol::did_open(client, &uri, lang, &text);
}

fn report_lsp_error(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(err) = state.lsp.last_error.take() {
        use txv_core::message::{Message, MsgLevel};
        ctx.sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
        );
    }
}

pub(super) fn send_did_change(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(changed) = boxed.downcast_ref::<ContentChanged>() else {
        return;
    };

    let lang = protocol::language_id(&changed.path);
    let root = state.root_dir.clone();
    start_lsp(state, lang, &root);
    let Some(client) = state.lsp.get_client_mut(lang) else {
        return;
    };

    let uri = protocol::path_to_uri(&changed.path);
    let key = changed.path.to_string_lossy().to_string();
    let version = state.doc_versions.entry(key).or_insert(1);
    *version += 1;
    protocol::did_change(client, &uri, *version, &changed.content);
}
