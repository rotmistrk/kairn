//! LSP command handler — wires LSP events into the editor.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

use super::diagnostics;
use super::messages::LspMessage;
use super::protocol;

/// Handle LSP-related commands. Called before main dispatch.
pub fn handle_lsp_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command {
        CM_OPEN_FILE | CM_OPEN_FILE_FOCUS => send_did_open(ctx, state),
        _ => {}
    }
}

/// Poll all LSP servers and dispatch notifications.
pub fn poll_lsp(state: &AppState, queue: &mut EventQueue) {
    for (_lang, msg) in state.lsp.poll_all() {
        match msg {
            LspMessage::Notification { method, params } => {
                if method == "textDocument/publishDiagnostics" {
                    if let Some((uri, diags)) = diagnostics::parse_publish_diagnostics(&params) {
                        queue.put_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
                    }
                }
            }
            LspMessage::Response { .. } => {}
        }
    }
}

fn send_did_open(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(path) = boxed.downcast_ref::<std::path::PathBuf>() else {
        return;
    };

    let lang = protocol::language_id(path);
    let root = state.root_dir.clone();
    let Some(client) = state.lsp.get_or_start(lang, &root) else {
        return;
    };

    let uri = protocol::path_to_uri(path);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    protocol::did_open(client, &uri, lang, &text);
}
