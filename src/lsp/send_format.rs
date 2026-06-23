//! LSP format and JDT class-contents requests.

use std::path::PathBuf;

use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;
use crate::lsp::pending::JdtRequest;

use super::pending::PendingKind;
use super::send::start_lsp;
use super::send_helpers;
use super::{protocol, requests};

pub(super) fn send_format(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((path, range, tab_size)) = extract_format_params(ctx, state) else {
        return;
    };

    let lang = protocol::language_id(&path);
    let root = state.root_dir().clone();
    start_lsp(state, lang, &root);

    if state.lsp_sub_mut().registry_mut().is_initializing(lang) {
        send_helpers::defer(
            ctx,
            state,
            CM_LSP_FORMAT,
            lang,
            Box::new((path.clone(), range, tab_size)),
        );
        return;
    }

    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut(lang) else {
        send_helpers::emit_last_error(ctx, state);
        return;
    };
    let uri = protocol::path_to_uri(&path);
    let id = if let Some((start, end)) = range {
        requests::range_formatting(client, &uri, start, end, tab_size)
    } else {
        requests::formatting(client, &uri, tab_size)
    };
    state
        .lsp_sub_mut()
        .pending_mut()
        .insert_with_lang(id, PendingKind::Format, lang);
}

type FormatParams = (PathBuf, Option<(u32, u32)>, u32);

fn extract_format_params(ctx: &mut CommandContext, state: &mut AppState) -> Option<FormatParams> {
    if let Some(boxed) = ctx.data().as_ref() {
        boxed
            .downcast_ref::<FormatParams>()
            .map(|(p, r, t)| (p.clone(), *r, *t))
    } else {
        let (_, lang_str) = send_helpers::current_file_info(state);
        if lang_str.is_empty() {
            return None;
        }
        let path = state
            .workspace_mut()
            .broker_mut()
            .last_opened()
            .map(PathBuf::from)
            .unwrap_or_default();
        Some((path, None, 4))
    }
}

pub(super) fn send_jdt_class_contents(jdt: &JdtRequest, state: &mut AppState) {
    let root = state.root_dir().clone();
    start_lsp(state, "java", &root);
    let Some(client) = state.lsp_sub_mut().registry_mut().get_client_mut("java") else {
        return;
    };
    let params = serde_json::json!({ "uri": jdt.uri });
    let id = client.send_request("java/classFileContents", params);
    state.lsp_sub_mut().pending_mut().insert_with_lang(
        id,
        PendingKind::JdtClassContents {
            line: jdt.line,
            character: jdt.character,
        },
        "java",
    );
}
