//! Key handling for StructuredView — navigation and editing dispatch.

mod ops;

use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditResult;

use crate::structured::NodeKind;

use super::{ColFocus, EditTarget, StructuredView};

/// Handle CM_SAVE command for the structured view.
pub fn handle_save_command(view: &mut StructuredView, queue: &mut EventQueue) -> HandleResult {
    match view.save() {
        Ok(()) => {
            let name = view
                .path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            let msg = txv_core::message::Message::info("struct", format!("Saved: {name}"));
            queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = txv_core::message::Message::error("struct", format!("Save failed: {e}"));
            queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
    HandleResult::Consumed
}

/// Handle a key event for the structured view.
pub fn handle_struct_key(view: &mut StructuredView, key: &KeyEvent, queue: &mut EventQueue) -> HandleResult {
    // Route to inline editor first when active
    if let Some(ref mut editor) = view.editing {
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
                        let msg = txv_core::message::Message::error("struct", err);
                        queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
        return HandleResult::Consumed;
    }
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            let max = view.visible_nodes.len().saturating_sub(1);
            if view.cursor < max {
                view.cursor += 1;
                view.sync_scroll();
                view.state.mark_dirty();
            }
            HandleResult::Consumed
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if view.cursor > 0 {
                view.cursor -= 1;
                view.sync_scroll();
                view.state.mark_dirty();
            }
            HandleResult::Consumed
        }
        KeyCode::Char('g') => {
            view.cursor = 0;
            view.sync_scroll();
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        KeyCode::Char('G') => {
            view.cursor = view.visible_nodes.len().saturating_sub(1);
            view.sync_scroll();
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Right => {
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
        KeyCode::Char('h') | KeyCode::Left => {
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
        KeyCode::Tab => {
            view.col_focus = match view.col_focus {
                ColFocus::Key => ColFocus::Value,
                ColFocus::Value => ColFocus::Meta,
                ColFocus::Meta => ColFocus::Key,
            };
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        KeyCode::Enter => {
            handle_enter(view);
            HandleResult::Consumed
        }
        KeyCode::Char('n') => {
            ops::handle_new_sibling(view);
            HandleResult::Consumed
        }
        KeyCode::Char('b') => {
            ops::handle_new_child(view);
            HandleResult::Consumed
        }
        KeyCode::Char('d') => {
            ops::handle_delete(view);
            HandleResult::Consumed
        }
        KeyCode::Char('c') => {
            ops::handle_clone(view);
            HandleResult::Consumed
        }
        KeyCode::Char('t') => {
            ops::handle_cycle_type(view);
            HandleResult::Consumed
        }
        KeyCode::Char('T') => {
            ops::handle_convert_container(view);
            HandleResult::Consumed
        }
        KeyCode::Char('J') => {
            ops::handle_swap_down(view);
            HandleResult::Consumed
        }
        KeyCode::Char('K') => {
            ops::handle_swap_up(view);
            HandleResult::Consumed
        }
        KeyCode::Char('H') => {
            ops::handle_promote(view);
            HandleResult::Consumed
        }
        KeyCode::Char('L') => {
            ops::handle_demote(view);
            HandleResult::Consumed
        }
        KeyCode::Char('!') => {
            ops::handle_toggle_inline(view);
            HandleResult::Consumed
        }
        KeyCode::Char('s') => {
            ops::handle_sort(view);
            HandleResult::Consumed
        }
        KeyCode::Char('S') => {
            ops::handle_sort_by_path_start(view);
            HandleResult::Consumed
        }
        KeyCode::Char('f') => {
            ops::handle_filter_start(view);
            HandleResult::Consumed
        }
        KeyCode::Char('F') => {
            ops::handle_filter_clear(view);
            HandleResult::Consumed
        }
        KeyCode::Char('u') => {
            view.apply_undo();
            HandleResult::Consumed
        }
        KeyCode::Char('r') if key.modifiers.ctrl => {
            view.apply_redo();
            HandleResult::Consumed
        }
        KeyCode::Char(':') => {
            queue.put_command(crate::commands::CM_COMMAND_PREFILL, Some(Box::new(String::new())));
            HandleResult::Consumed
        }
        _ => HandleResult::Ignored,
    }
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
