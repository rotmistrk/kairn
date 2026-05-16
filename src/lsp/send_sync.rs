//! LSP document sync senders — didOpen, didChange notifications.

use txv_core::program::CommandContext;

use crate::handler::AppState;

use super::protocol;

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
            ctx.sink.push_command(
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
            ctx.sink.push_command(
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
