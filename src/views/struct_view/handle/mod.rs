//! Key handling for StructuredView — navigation and editing dispatch.

pub(crate) mod filter_ops;
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
            view.tree
                .state_mut()
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("struct", format!("Save failed: {e}"));
            view.tree
                .state_mut()
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
            view.tree
                .state_mut()
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
        KeyCode::Char('y') => handle_yank(view),
        KeyCode::Char('p') => handle_paste(view),
        KeyCode::Char('t') => ops::handle_cycle_type(view),
        KeyCode::Char('T') => ops::handle_convert_container(view),
        KeyCode::Char('J') => ops::handle_swap_down(view),
        KeyCode::Char('K') => ops::handle_swap_up(view),
        KeyCode::Char('H') => ops::handle_promote(view),
        KeyCode::Char('L') => ops::handle_demote(view),
        KeyCode::Char('!') => ops::handle_toggle_inline(view),
        KeyCode::Char('s') => ops::handle_sort(view),
        KeyCode::Char('S') => ops::handle_sort_by_path_start(view),
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
        view.input_line = None;
        view.editing_row = None;
        view.tree.data_mut().filter_text = text;
        view.filtering = false;
        view.rebuild_visible();
        view.clamp_cursor();
        view.sync_title();
    } else if let Some(sort_target) = view.sort_path_target.take() {
        view.input_line = None;
        view.editing_row = None;
        view.save_undo_point();
        view.tree.data_mut().doc.sort_children_by_path(sort_target, &text, true);
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
    } else {
        view.save_undo_point();
        if let Some(err) = view.commit_edit() {
            let msg = Message::error("struct", err);
            view.tree
                .state_mut()
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
    view.tree.state_mut().mark_dirty();
}

fn handle_jump_top(view: &mut StructuredView) -> HandleResult {
    view.tree.set_cursor(0);
    HandleResult::Consumed
}

fn handle_jump_bottom(view: &mut StructuredView) -> HandleResult {
    let max = view.tree.data_mut().visible_nodes.len().saturating_sub(1);
    view.tree.set_cursor(max);
    HandleResult::Consumed
}

fn handle_move_down(view: &mut StructuredView) -> HandleResult {
    let max = view.tree.data_mut().visible_nodes.len().saturating_sub(1);
    if view.tree.cursor() < max {
        view.tree.set_cursor(view.tree.cursor() + 1);
    }
    HandleResult::Consumed
}

fn handle_move_up(view: &mut StructuredView) -> HandleResult {
    if view.tree.cursor() > 0 {
        view.tree.set_cursor(view.tree.cursor() - 1);
    }
    HandleResult::Consumed
}

fn handle_expand(view: &mut StructuredView) -> HandleResult {
    let cursor = view.tree.cursor();
    if let Some(&node_id) = view.tree.data_mut().visible_nodes.get(cursor) {
        if view.tree.data_mut().doc.node_kind(node_id) != NodeKind::Scalar {
            view.tree.data_mut().doc.toggle_expand(node_id);
            view.rebuild_visible();
            view.clamp_cursor();
            view.tree.state_mut().mark_dirty();
        }
    }
    HandleResult::Consumed
}

fn handle_collapse_or_parent(view: &mut StructuredView) -> HandleResult {
    let cursor = view.tree.cursor();
    if let Some(&node_id) = view.tree.data_mut().visible_nodes.get(cursor) {
        let kind = view.tree.data_mut().doc.node_kind(node_id);
        if kind != NodeKind::Scalar && view.tree.data_mut().doc.is_expanded(node_id) {
            view.tree.data_mut().doc.toggle_expand(node_id);
            view.rebuild_visible();
            view.tree.state_mut().mark_dirty();
        } else if let Some(parent) = view.tree.data_mut().doc.parent(node_id) {
            if let Some(pos) = view.tree.data_mut().visible_nodes.iter().position(|&n| n == parent) {
                view.tree.set_cursor(pos);
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
    view.tree.set_focused_col(Some(view.col_focus.as_col_index()));
    HandleResult::Consumed
}

fn handle_enter(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes.get(cursor) else {
        return;
    };
    match view.col_focus {
        ColFocus::Value => {
            if view.tree.data_mut().doc.node_kind(node_id) == NodeKind::Scalar {
                view.start_edit(EditTarget::Value);
            }
        }
        ColFocus::Key => {
            if view.tree.data_mut().doc.key(node_id).is_some() {
                view.start_edit(EditTarget::Key);
            }
        }
        ColFocus::Meta => {
            view.start_edit(EditTarget::Meta);
        }
    }
}

fn handle_yank(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes.get(cursor) else {
        return;
    };
    view.yanked = Some(view.tree.data_mut().doc.serialize_node(node_id));
}

fn handle_paste(view: &mut StructuredView) {
    let Some(json) = view.yanked.clone() else {
        return;
    };
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes.get(cursor) else {
        return;
    };
    view.save_undo_point();
    if let Ok(new_id) = view.tree.data_mut().doc.paste_after(node_id, &json) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes.iter().position(|&n| n == new_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}
