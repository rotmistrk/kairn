//! Confirmation response handler — dispatches CM_CONFIRM_RESPONSE based on context.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::commands::*;
use crate::handler::downcast_desktop;
use crate::layout_group::SlotId;
use crate::views::editor::EditorView;

pub fn handle_confirm_response(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(context) = state.confirm_context.take() else {
        return;
    };
    let ch = ctx
        .data
        .as_ref()
        .and_then(|b| b.downcast_ref::<char>())
        .copied()
        .unwrap_or('c'); // treat missing as cancel

    match context {
        ConfirmContext::EditorClose(path) => handle_editor_close(ctx, state, &path, ch),
        ConfirmContext::TodoDelete => handle_todo_delete(ctx, state, ch),
        ConfirmContext::TodoCrypto => handle_todo_crypto(ctx, state, ch),
    }
}

fn handle_editor_close(ctx: &mut CommandContext, state: &mut AppState, path: &str, ch: char) {
    match ch {
        'y' => {
            // Save and close
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let panel = desktop.panel_mut(SlotId::Center);
                for i in 0..panel.tab_count() {
                    if let Some(view) = panel.view_at_mut(i) {
                        if let Some(any) = view.as_any_mut() {
                            if let Some(editor) = any.downcast_mut::<EditorView>() {
                                if editor.path().to_string_lossy() == path {
                                    let content = editor.editor.buf().content();
                                    match crate::editor::save::save_file(editor.path(), &content) {
                                        Ok(()) => {
                                            editor.editor.buf().mark_saved();
                                            ctx.sink.push_command(CM_FILE_CLOSED, Some(Box::new(path.to_string())));
                                            if state.pending_tab.is_none() {
                                                ctx.sink.push_command(CM_TAB_CLOSE, None);
                                            }
                                        }
                                        Err(e) => {
                                            let msg = txv_core::message::Message::error(
                                                "editor",
                                                format!("Save failed: {e}"),
                                            );
                                            ctx.sink
                                                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        'n' => {
            // Discard and close
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let panel = desktop.panel_mut(SlotId::Center);
                for i in 0..panel.tab_count() {
                    if let Some(view) = panel.view_at_mut(i) {
                        if let Some(any) = view.as_any_mut() {
                            if let Some(editor) = any.downcast_mut::<EditorView>() {
                                if editor.path().to_string_lossy() == path {
                                    editor.editor.buf().mark_saved();
                                    ctx.sink.push_command(CM_FILE_CLOSED, Some(Box::new(path.to_string())));
                                    if state.pending_tab.is_none() {
                                        ctx.sink.push_command(CM_TAB_CLOSE, None);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {} // Cancel — do nothing
    }
    // Complete pending eviction if applicable
    if ch == 'y' || ch == 'n' {
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            crate::handler_evict::complete_pending_insert(desktop, state);
        }
    }
}

fn handle_todo_delete(ctx: &mut CommandContext, _state: &mut AppState, ch: char) {
    if ch != 'y' {
        return;
    }
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let panel = desktop.panel_mut(SlotId::Left);
        if let Some(view) = panel.view_at_mut(2) {
            if let Some(any) = view.as_any_mut() {
                if let Some(todo) = any.downcast_mut::<crate::views::todo_tree::TodoTreeView>() {
                    todo.confirm_delete_execute();
                }
            }
        }
    }
}

fn handle_todo_crypto(ctx: &mut CommandContext, _state: &mut AppState, ch: char) {
    // For crypto, the 'char' is actually a commit signal; the passphrase comes via data.
    // In our simplified flow, we treat any non-cancel as "use the confirm text as passphrase".
    let passphrase = ctx
        .data
        .as_ref()
        .and_then(|b| b.downcast_ref::<String>())
        .cloned()
        .unwrap_or_default();
    if ch == '\x1b' || passphrase.is_empty() {
        return; // cancelled
    }
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let panel = desktop.panel_mut(SlotId::Left);
        if let Some(view) = panel.view_at_mut(2) {
            if let Some(any) = view.as_any_mut() {
                if let Some(todo) = any.downcast_mut::<crate::views::todo_tree::TodoTreeView>() {
                    todo.crypto_passphrase_response(&passphrase);
                }
            }
        }
    }
}
