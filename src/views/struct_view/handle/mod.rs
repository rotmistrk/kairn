//! Key handling for StructuredView — navigation and editing dispatch.

pub(crate) mod clipboard_ops;
pub(crate) mod filter_ops;
pub(crate) mod move_ops;
pub(crate) mod ops;

use txv_core::message::Message;
use txv_core::prelude::*;

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
            view.group
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("struct", format!("Save failed: {e}"));
            view.group
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
    HandleResult::Consumed
}

/// Handle a key event for the structured view.
pub fn handle_struct_key(view: &mut StructuredView, key: &KeyEvent) -> HandleResult {
    match key.code() {
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
        KeyCode::Char('r') if key.modifiers().ctrl() => {
            view.apply_redo();
            HandleResult::Consumed
        }
        KeyCode::Char(':') => {
            view.group
                .put_command(CM_COMMAND_PREFILL, Some(Box::new(String::new())));
            HandleResult::Consumed
        }
        _ => handle_struct_ops(view, key),
    }
}

fn handle_struct_ops(view: &mut StructuredView, key: &KeyEvent) -> HandleResult {
    match key.code() {
        KeyCode::Char('n') => ops::handle_new_sibling(view),
        KeyCode::Char('b') => ops::handle_new_child(view),
        KeyCode::Char('d') => ops::handle_delete(view),
        KeyCode::Char('c') => ops::handle_clone(view),
        KeyCode::Char('y') => clipboard_ops::handle_yank(view),
        KeyCode::Char('p') => clipboard_ops::handle_paste(view),
        KeyCode::Char('t') => ops::handle_cycle_type(view),
        KeyCode::Char('T') => ops::handle_convert_container(view),
        KeyCode::Char('J') => move_ops::handle_swap_down(view),
        KeyCode::Char('K') => move_ops::handle_swap_up(view),
        KeyCode::Char('H') => move_ops::handle_promote(view),
        KeyCode::Char('L') => move_ops::handle_demote(view),
        KeyCode::Char('!') => move_ops::handle_toggle_inline(view),
        KeyCode::Char('s') => move_ops::handle_sort(view),
        KeyCode::Char('S') => move_ops::handle_sort_by_path_start(view),
        KeyCode::Char('f') => filter_ops::handle_filter_start(view),
        KeyCode::Char('F') => filter_ops::handle_filter_clear(view),
        KeyCode::Char('u') => view.apply_undo(),
        _ => return HandleResult::Ignored,
    }
    HandleResult::Consumed
}

pub fn drain_edit_commands(view: &mut StructuredView) {
    for ev in view.child_sink.drain() {
        if let Event::Command { id, data, .. } = ev {
            match id {
                CM_OK => {
                    let text = data
                        .and_then(|d| d.downcast::<String>().ok())
                        .map(|s| *s)
                        .unwrap_or_default();
                    handle_commit(view, text);
                    return;
                }
                CM_CANCEL => {
                    view.filtering = false;
                    view.sort_path_target = None;
                    view.cancel_edit();
                    return;
                }
                _ => {}
            }
        }
    }
}

fn handle_commit(view: &mut StructuredView, text: String) {
    if view.filtering {
        view.remove_input_child();
        view.editing_row = None;
        view.inner_mut().data_mut().set_filter_text(text);
        view.filtering = false;
        view.rebuild_visible();
        view.clamp_cursor();
        view.sync_title();
    } else if let Some(sort_target) = view.sort_path_target.take() {
        view.remove_input_child();
        view.editing_row = None;
        view.save_undo_point();
        view.inner_mut()
            .data_mut()
            .doc_mut()
            .sort_children_by_path(sort_target, &text, true);
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
    } else {
        view.save_undo_point();
        if let Some(err) = view.commit_edit() {
            let msg = Message::error("struct", err);
            view.group
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
    view.group.mark_dirty();
}

fn handle_jump_top(view: &mut StructuredView) -> HandleResult {
    view.inner_mut().set_cursor(0);
    HandleResult::Consumed
}

fn handle_jump_bottom(view: &mut StructuredView) -> HandleResult {
    let max = view.inner_mut().data_mut().visible_nodes().len().saturating_sub(1);
    view.inner_mut().set_cursor(max);
    HandleResult::Consumed
}

fn handle_move_down(view: &mut StructuredView) -> HandleResult {
    let max = view.inner_mut().data_mut().visible_nodes().len().saturating_sub(1);
    let cur = view.inner().cursor();
    if cur < max {
        view.inner_mut().set_cursor(cur + 1);
    }
    HandleResult::Consumed
}

fn handle_move_up(view: &mut StructuredView) -> HandleResult {
    let cur = view.inner().cursor();
    if cur > 0 {
        view.inner_mut().set_cursor(cur - 1);
    }
    HandleResult::Consumed
}

fn handle_expand(view: &mut StructuredView) -> HandleResult {
    let cursor = view.inner().cursor();
    if let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) {
        if view.inner_mut().data_mut().doc().node_kind(node_id) != NodeKind::Scalar {
            view.inner_mut().data_mut().doc_mut().toggle_expand(node_id);
            view.rebuild_visible();
            view.clamp_cursor();
            view.group.mark_dirty();
        }
    }
    HandleResult::Consumed
}

fn handle_collapse_or_parent(view: &mut StructuredView) -> HandleResult {
    let cursor = view.inner().cursor();
    if let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) {
        let kind = view.inner_mut().data_mut().doc().node_kind(node_id);
        if kind != NodeKind::Scalar && view.inner_mut().data_mut().doc().is_expanded(node_id) {
            view.inner_mut().data_mut().doc_mut().toggle_expand(node_id);
            view.rebuild_visible();
            view.group.mark_dirty();
        } else if let Some(parent) = view.inner_mut().data_mut().doc().parent(node_id) {
            if let Some(pos) = view
                .inner_mut()
                .data_mut()
                .visible_nodes()
                .iter()
                .position(|&n| n == parent)
            {
                view.inner_mut().set_cursor(pos);
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
    let idx = view.col_focus.as_col_index();
    view.inner_mut().set_focused_col(Some(idx));
    HandleResult::Consumed
}

fn handle_enter(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    match view.col_focus {
        ColFocus::Value => {
            if view.inner_mut().data_mut().doc().node_kind(node_id) == NodeKind::Scalar {
                view.start_edit(EditTarget::Value);
            }
        }
        ColFocus::Key => {
            if view.inner_mut().data_mut().doc().key(node_id).is_some() {
                view.start_edit(EditTarget::Key);
            }
        }
        ColFocus::Meta => {
            view.start_edit(EditTarget::Meta);
        }
    }
}
