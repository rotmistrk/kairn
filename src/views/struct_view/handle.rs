//! Key handling for StructuredView — navigation (read-only).

use txv_core::prelude::*;

use crate::structured::NodeKind;

use super::{ColFocus, StructuredView};

/// Handle a key event for the structured view. Navigation only for now.
pub fn handle_struct_key(view: &mut StructuredView, key: &KeyEvent, _queue: &mut EventQueue) -> HandleResult {
    match key.code {
        // Cursor down
        KeyCode::Down | KeyCode::Char('j') => {
            let max = view.visible_nodes.len().saturating_sub(1);
            if view.cursor < max {
                view.cursor += 1;
                view.sync_scroll();
                view.state.mark_dirty();
            }
            HandleResult::Consumed
        }
        // Cursor up
        KeyCode::Up | KeyCode::Char('k') => {
            if view.cursor > 0 {
                view.cursor -= 1;
                view.sync_scroll();
                view.state.mark_dirty();
            }
            HandleResult::Consumed
        }
        // First row
        KeyCode::Char('g') => {
            view.cursor = 0;
            view.sync_scroll();
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        // Last row
        KeyCode::Char('G') => {
            view.cursor = view.visible_nodes.len().saturating_sub(1);
            view.sync_scroll();
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        // Toggle expand / expand
        KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Right => {
            if let Some(&node_id) = view.visible_nodes.get(view.cursor) {
                if view.doc.node_kind(node_id) != NodeKind::Scalar {
                    view.doc.toggle_expand(node_id);
                    view.rebuild_visible();
                    // Clamp cursor
                    if view.cursor >= view.visible_nodes.len() {
                        view.cursor = view.visible_nodes.len().saturating_sub(1);
                    }
                    view.sync_scroll();
                    view.state.mark_dirty();
                }
            }
            HandleResult::Consumed
        }
        // Collapse or go to parent
        KeyCode::Char('h') | KeyCode::Left => {
            if let Some(&node_id) = view.visible_nodes.get(view.cursor) {
                let kind = view.doc.node_kind(node_id);
                if kind != NodeKind::Scalar && view.doc.is_expanded(node_id) {
                    // Collapse this container
                    view.doc.toggle_expand(node_id);
                    view.rebuild_visible();
                    view.sync_scroll();
                    view.state.mark_dirty();
                } else if let Some(parent) = view.doc.parent(node_id) {
                    // Go to parent
                    if let Some(pos) = view.visible_nodes.iter().position(|&n| n == parent) {
                        view.cursor = pos;
                        view.sync_scroll();
                        view.state.mark_dirty();
                    }
                }
            }
            HandleResult::Consumed
        }
        // Cycle column focus
        KeyCode::Tab => {
            view.col_focus = match view.col_focus {
                ColFocus::Key => ColFocus::Value,
                ColFocus::Value => ColFocus::Meta,
                ColFocus::Meta => ColFocus::Key,
            };
            view.state.mark_dirty();
            HandleResult::Consumed
        }
        _ => HandleResult::Ignored,
    }
}
