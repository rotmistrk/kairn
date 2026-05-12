//! Key handling for StructuredView — navigation and editing dispatch.

mod ops;

use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditResult;

use crate::structured::NodeKind;

use super::{ColFocus, EditTarget, StructuredView};

/// Handle a key event for the structured view.
pub fn handle_struct_key(view: &mut StructuredView, key: &KeyEvent, _queue: &mut EventQueue) -> HandleResult {
    // Route to inline editor first when active
    if let Some(ref mut editor) = view.editing {
        match editor.handle_key(key) {
            InlineEditResult::Continue => {}
            InlineEditResult::Commit(_) => {
                view.save_undo_point();
                view.commit_edit();
            }
            InlineEditResult::Cancel => view.cancel_edit(),
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
        KeyCode::Char('u') => {
            view.apply_undo();
            HandleResult::Consumed
        }
        KeyCode::Char('r') if key.modifiers.ctrl => {
            view.apply_redo();
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
