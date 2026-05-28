//! Key handling for StructuredView — navigation and editing dispatch.

mod ops;

use txv_core::message::Message;
use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditResult;

use crate::commands::CM_COMMAND_PREFILL;
use crate::structured::NodeKind;

use super::{ColFocus, EditTarget, StructuredView};

/// Handle CM_SAVE command for the structured view.
pub fn handle_save_command(view: &mut StructuredView) -> HandleResult {
    match view.save() {
        Ok(()) => {
            let name = view
                .path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            let msg = Message::info("struct", format!("Saved: {name}"));
            view.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("struct", format!("Save failed: {e}"));
            view.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
    HandleResult::Consumed
}

/// Handle a key event for the structured view.
pub fn handle_struct_key(view: &mut StructuredView, key: &KeyEvent) -> HandleResult {
    // Route to inline editor first when active
    if view.editing.is_some() {
        return handle_editing_mode(view, key);
    }
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => handle_move_down(view),
        KeyCode::Up | KeyCode::Char('k') => handle_move_up(view),
        KeyCode::Char('g') => handle_jump_top(view),
        KeyCode::Char('G') => handle_jump_bottom(view),
        KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Right => handle_expand(view),
        KeyCode::Char('h') | KeyCode::Left => handle_collapse_or_parent(view),
        KeyCode::Tab => handle_tab_focus(view),
        KeyCode::Enter => {
            handle_enter(view);
            HandleResult::Consumed
        }
        KeyCode::Char('r') if key.modifiers.ctrl => {
            view.apply_redo();
            HandleResult::Consumed
        }
        KeyCode::Char(':') => {
            view.state
                .put_command(CM_COMMAND_PREFILL, Some(Box::new(String::new())));
            HandleResult::Consumed
        }
        _ => handle_struct_ops(view, key),
    }
}

fn handle_struct_ops(view: &mut StructuredView, key: &KeyEvent) -> HandleResult {
    match key.code {
        KeyCode::Char('n') => ops::handle_new_sibling(view),
        KeyCode::Char('b') => ops::handle_new_child(view),
        KeyCode::Char('d') => ops::handle_delete(view),
        KeyCode::Char('c') => ops::handle_clone(view),
        KeyCode::Char('t') => ops::handle_cycle_type(view),
        KeyCode::Char('T') => ops::handle_convert_container(view),
        KeyCode::Char('J') => ops::handle_swap_down(view),
        KeyCode::Char('K') => ops::handle_swap_up(view),
        KeyCode::Char('H') => ops::handle_promote(view),
        KeyCode::Char('L') => ops::handle_demote(view),
        KeyCode::Char('!') => ops::handle_toggle_inline(view),
        KeyCode::Char('s') => ops::handle_sort(view),
        KeyCode::Char('S') => ops::handle_sort_by_path_start(view),
        KeyCode::Char('f') => ops::handle_filter_start(view),
        KeyCode::Char('F') => ops::handle_filter_clear(view),
        KeyCode::Char('u') => view.apply_undo(),
        _ => return HandleResult::Ignored,
    }
    HandleResult::Consumed
}

fn handle_jump_top(view: &mut StructuredView) -> HandleResult {
    view.cursor = 0;
    view.sync_scroll();
    view.state.mark_dirty();
    HandleResult::Consumed
}

fn handle_jump_bottom(view: &mut StructuredView) -> HandleResult {
    view.cursor = view.visible_nodes.len().saturating_sub(1);
    view.sync_scroll();
    view.state.mark_dirty();
    HandleResult::Consumed
}

fn handle_editing_mode(view: &mut StructuredView, key: &KeyEvent) -> HandleResult {
    let Some(editor) = view.editing.as_mut() else {
        return HandleResult::Ignored;
    };
    match editor.handle_key(key) {
        InlineEditResult::Continue => {}
        InlineEditResult::Commit(_) => {
            if view.filtering {
                let text = view.editing.take().map(|e| e.buffer).unwrap_or_default();
                view.filter_text = text;
                view.filtering = false;
                view.rebuild_visible();
                view.clamp_cursor();
                view.sync_scroll();
                view.sync_title();
            } else if let Some(sort_target) = view.sort_path_target.take() {
                let text = view.editing.take().map(|e| e.buffer).unwrap_or_default();
                view.save_undo_point();
                view.doc.sort_children_by_path(sort_target, &text, true);
                view.dirty = true;
                view.sync_title();
                view.rebuild_visible();
            } else {
                view.save_undo_point();
                if let Some(err) = view.commit_edit() {
                    let msg = Message::error("struct", err);
                    view.state
                        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
            }
        }
        InlineEditResult::Cancel => {
            view.filtering = false;
            view.sort_path_target = None;
            view.cancel_edit();
        }
    }
    view.state.mark_dirty();
    HandleResult::Consumed
}

fn handle_move_down(view: &mut StructuredView) -> HandleResult {
    let max = view.visible_nodes.len().saturating_sub(1);
    if view.cursor < max {
        view.cursor += 1;
        view.sync_scroll();
        view.state.mark_dirty();
    }
    HandleResult::Consumed
}

fn handle_move_up(view: &mut StructuredView) -> HandleResult {
    if view.cursor > 0 {
        view.cursor -= 1;
        view.sync_scroll();
        view.state.mark_dirty();
    }
    HandleResult::Consumed
}

fn handle_expand(view: &mut StructuredView) -> HandleResult {
    if let Some(&node_id) = view.visible_nodes.get(view.cursor) {
        if view.doc.node_kind(node_id) != NodeKind::Scalar {
            view.doc.toggle_expand(node_id);
            view.rebuild_visible();
            view.clamp_cursor();
            view.sync_scroll();
            view.state.mark_dirty();
        }
    }
    HandleResult::Consumed
}

fn handle_collapse_or_parent(view: &mut StructuredView) -> HandleResult {
    if let Some(&node_id) = view.visible_nodes.get(view.cursor) {
        let kind = view.doc.node_kind(node_id);
        if kind != NodeKind::Scalar && view.doc.is_expanded(node_id) {
            view.doc.toggle_expand(node_id);
            view.rebuild_visible();
            view.sync_scroll();
            view.state.mark_dirty();
        } else if let Some(parent) = view.doc.parent(node_id) {
            if let Some(pos) = view.visible_nodes.iter().position(|&n| n == parent) {
                view.cursor = pos;
                view.sync_scroll();
                view.state.mark_dirty();
            }
        }
    }
    HandleResult::Consumed
}

fn handle_tab_focus(view: &mut StructuredView) -> HandleResult {
    view.col_focus = match view.col_focus {
        ColFocus::Key => ColFocus::Value,
        ColFocus::Value => ColFocus::Meta,
        ColFocus::Meta => ColFocus::Key,
    };
    view.state.mark_dirty();
    HandleResult::Consumed
}

fn handle_enter(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    match view.col_focus {
        ColFocus::Value => {
            if view.doc.node_kind(node_id) == NodeKind::Scalar {
                view.start_edit(EditTarget::Value);
            }
        }
        ColFocus::Key => {
            if view.doc.key(node_id).is_some() {
                view.start_edit(EditTarget::Key);
            }
        }
        ColFocus::Meta => {
            view.start_edit(EditTarget::Meta);
        }
    }
}
