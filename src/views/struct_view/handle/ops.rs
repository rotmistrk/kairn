//! Structural operation handlers for StructuredView.

use crate::structured::{NodeId, NodeKind};
use crate::views::struct_view::{EditTarget, StructuredView};

/// Apply a doc operation that creates a new sibling-like node.
/// Handles: save undo, call op, rebuild, move cursor to new node,
/// and start key edit if parent is a dict.
fn apply_sibling_op(
    view: &mut StructuredView,
    op: fn(&mut dyn crate::structured::StructuredDoc, NodeId) -> Result<NodeId, String>,
) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let parent_kind = view
        .inner()
        .data()
        .doc()
        .parent(node_id)
        .map(|p| view.inner_mut().data_mut().doc().node_kind(p));
    view.save_undo_point();
    let result = op(&mut **view.inner_mut().data_mut().doc_mut(), node_id);
    if let Ok(new_id) = result {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        move_cursor_to_node(view, new_id);
        view.group.mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_new_sibling(view: &mut StructuredView) {
    apply_sibling_op(view, |doc, id| doc.add_sibling(id));
}

pub fn handle_clone(view: &mut StructuredView) {
    apply_sibling_op(view, |doc, id| doc.clone_node(id));
}

pub fn handle_new_child(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    if view.inner_mut().data_mut().doc().node_kind(node_id) == NodeKind::Scalar {
        return;
    }
    view.save_undo_point();
    if let Ok(new_id) = view.inner_mut().data_mut().doc_mut().add_child(node_id) {
        view.dirty = true;
        view.sync_title();
        if !view.inner_mut().data_mut().doc().is_expanded(node_id) {
            view.inner_mut().data_mut().doc_mut().toggle_expand(node_id);
        }
        view.rebuild_visible();
        move_cursor_to_node(view, new_id);
        view.group.mark_dirty();
    }
}

pub fn handle_delete(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.inner_mut().data_mut().doc_mut().remove(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        view.clamp_cursor();
        view.group.mark_dirty();
    }
}

/// Apply a void doc mutation that doesn't return a new node.
/// Handles: get cursor, save undo, call op, rebuild, mark dirty.
fn apply_void_op(view: &mut StructuredView, op: fn(&mut dyn crate::structured::StructuredDoc, NodeId)) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    op(&mut **view.inner_mut().data_mut().doc_mut(), node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.group.mark_dirty();
}

pub fn handle_cycle_type(view: &mut StructuredView) {
    apply_void_op(view, |doc, id| doc.cycle_type(id));
}

pub fn handle_convert_container(view: &mut StructuredView) {
    apply_void_op(view, |doc, id| doc.convert_container(id));
}

fn move_cursor_to_node(view: &mut StructuredView, node_id: NodeId) {
    if let Some(pos) = view
        .inner_mut()
        .data_mut()
        .visible_nodes()
        .iter()
        .position(|&n| n == node_id)
    {
        view.inner_mut().set_cursor(pos);
    }
}
