//! LSP document sync senders — didOpen, didChange notifications.

use std::fs;
use std::path::PathBuf;

use txv_core::program::CommandContext;

use crate::commands::{ContentChanged, OpenFileRequest};
use crate::handler::AppState;

use super::protocol;
use super::send_helpers::start_lsp;

pub(super) fn send_did_open(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<OpenFileRequest>() else {
        return;
    };
    let path = req.path.clone();
    let lang = protocol::language_id(&path).to_string();
    let root = state.root_dir().clone();
    start_lsp(state, &lang, &root);

    if state.lsp_sub_mut().registry_mut().is_initializing(&lang) {
        state
            .lsp_sub_mut()
            .registry_mut()
            .pending_opens_mut()
            .push((lang.to_string(), path.clone()));
        return;
    }

    do_send_did_open(ctx, state, &path, &lang);
}

fn do_send_did_open(ctx: &mut CommandContext, state: &mut AppState, path: &PathBuf, lang: &str) {
    let (reg, lsp_st) = state.lsp_sub_mut().registry_and_state();
    let Some(client) = reg.get_client_mut(lang) else {
        report_lsp_error(ctx, state);
        return;
    };

    let key = path.to_string_lossy().to_string();
    if lsp_st.opened_files().contains(&key) {
        return;
    }
    lsp_st.opened_files_mut().insert(key);

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
    if let Some(err) = state.lsp_sub_mut().registry_mut().take_last_error() {
        use txv_core::message::{Message, MsgLevel};
        ctx.sink().push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::new(MsgLevel::Error, "lsp", err))),
        );
    }
}

pub(super) fn send_did_change(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(changed) = boxed.downcast_ref::<ContentChanged>() else {
        return;
    };

    let lang = protocol::language_id(&changed.path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);
    let (reg, lsp_st) = state.lsp_sub_mut().registry_and_state();
    let Some(client) = reg.get_client_mut(lang) else {
        return;
    };

    let uri = protocol::path_to_uri(&changed.path);
    let key = changed.path.to_string_lossy().to_string();
    let version = lsp_st.doc_versions_mut().entry(key).or_insert(1);
    *version += 1;
    protocol::did_change(client, &uri, *version, &changed.content);
}
