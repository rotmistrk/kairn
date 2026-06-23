//! Confirmation response handler — dispatches CM_CONFIRM_RESPONSE based on context.

use std::fs::read_to_string;

use txv_core::message::Message;
use txv_core::prelude::CM_QUIT;
use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::commands::*;
use crate::desktop::SlotId;
use crate::editor::save::save_file;
use crate::handler::downcast_desktop;
use crate::handler_close::save_all_dirty;
use crate::handler_evict::complete_pending_insert;
use crate::views::csv_view::CsvView;
use crate::views::editor::{EditorView, EditorViewExt};
use crate::views::todo_tree::TodoTreeView;

pub fn handle_confirm_response(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    let Some(context) = state.pending_mut().take_confirm_context() else {
        return;
    };
    let ch = ctx
        .data()
        .as_ref()
        .and_then(|b| b.downcast_ref::<char>())
        .copied()
        .unwrap_or('c'); // treat missing as cancel

    match context {
        ConfirmContext::EditorClose(path) => handle_editor_close(ctx, state, &path, ch),
        ConfirmContext::FileReload(path) => handle_file_reload(ctx, &path, ch),
        ConfirmContext::Quit => {
            if ch == 'y' {
                if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
                    save_all_dirty(desktop);
                }
                sink.push_command(CM_QUIT, None);
            }
        }
        ConfirmContext::TodoDelete => handle_todo_delete(ctx, state, ch),
        ConfirmContext::TodoCrypto => handle_todo_crypto(ctx, state, ch),
        ConfirmContext::CsvDeleteRow => handle_csv_delete(ctx, ch),
        ConfirmContext::McpToolConfirm => handle_mcp_confirm(state, ch),
    }
}

fn handle_editor_close(ctx: &mut CommandContext, state: &mut AppState, path: &str, ch: char) {
    match ch {
        'y' => save_and_close(ctx, state, path),
        'n' => discard_and_close(ctx, state, path),
        _ => {} // Cancel — do nothing
    }
    if ch == 'y' || ch == 'n' {
        if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
            complete_pending_insert(desktop, state);
        }
    }
}

fn save_and_close(ctx: &mut CommandContext, state: &mut AppState, path: &str) {
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_mut::<EditorView>() else {
            continue;
        };
        if editor.path().to_string_lossy() != path {
            continue;
        }
        let content = editor.editor().buf().content();
        let result = save_file(editor.path(), &content);
        if let Ok(()) = result {
            editor.editor().buf().mark_saved();
        }
        emit_save_result(&sink, result, state, path);
        break;
    }
}

fn emit_save_result(sink: &txv_core::prelude::EventSink, result: std::io::Result<()>, state: &AppState, path: &str) {
    match result {
        Ok(()) => {
            sink.push_command(CM_FILE_CLOSED, Some(Box::new(path.to_string())));
            if state.pending().pending_tab().is_none() {
                sink.push_command(CM_TAB_CLOSE, None);
            }
        }
        Err(e) => {
            let msg = Message::error("editor", format!("Save failed: {e}"));
            sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

fn discard_and_close(ctx: &mut CommandContext, state: &mut AppState, path: &str) {
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_mut::<EditorView>() else {
            continue;
        };
        if editor.path().to_string_lossy() != path {
            continue;
        }
        editor.editor().buf().mark_saved();
        sink.push_command(CM_FILE_CLOSED, Some(Box::new(path.to_string())));
        if state.pending().pending_tab().is_none() {
            sink.push_command(CM_TAB_CLOSE, None);
        }
        break;
    }
}

fn handle_file_reload(ctx: &mut CommandContext, path: &str, ch: char) {
    if ch != 'y' {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_mut::<EditorView>() else {
            continue;
        };
        if editor.path().to_string_lossy() != path {
            continue;
        }
        if let Ok(content) = read_to_string(editor.path()) {
            editor.editor_mut().replace_content(&content);
            editor.invalidate_highlight();
        }
        break;
    }
}

fn handle_todo_delete(ctx: &mut CommandContext, _state: &mut AppState, ch: char) {
    if ch != 'y' {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
        return;
    };
    let todo = panel
        .view_at_mut(2)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<TodoTreeView>());
    if let Some(todo) = todo {
        todo.confirm_delete_execute();
    }
}

fn handle_todo_crypto(ctx: &mut CommandContext, _state: &mut AppState, ch: char) {
    // For crypto, the 'char' is actually a commit signal; the passphrase comes via data.
    // In our simplified flow, we treat any non-cancel as "use the confirm text as passphrase".
    let passphrase = ctx
        .data()
        .as_ref()
        .and_then(|b| b.downcast_ref::<String>())
        .cloned()
        .unwrap_or_default();
    if ch == '\x1b' || passphrase.is_empty() {
        return; // cancelled
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
        return;
    };
    let todo = panel
        .view_at_mut(2)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<TodoTreeView>());
    if let Some(todo) = todo {
        todo.crypto_passphrase_response(&passphrase);
    }
}

fn handle_csv_delete(ctx: &mut CommandContext, ch: char) {
    use crate::views::csv_view::row_ops;
    if ch != 'y' {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let csv = panel
        .active_view_mut()
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<CsvView>());
    if let Some(csv) = csv {
        row_ops::execute_delete(csv);
    }
}

pub fn handle_set_confirm_context(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(context) = ctx.data().as_ref().and_then(|b| b.downcast_ref::<ConfirmContext>()) {
        state.pending_mut().set_confirm_context(Some(context.clone()));
    }
}

fn handle_mcp_confirm(state: &mut AppState, ch: char) {
    let reply = state.mcp_mut().take_confirm_reply();
    if let Some(tx) = reply {
        let allowed = ch == 'y';
        let _ = tx.send(Ok(serde_json::json!(allowed)));
    }
}
