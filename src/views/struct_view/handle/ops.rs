//! Structural operation handlers for StructuredView.

use crate::structured::NodeKind;
use crate::views::struct_view::{EditTarget, StructuredView};

pub fn handle_new_sibling(view: &mut StructuredView) {
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
    if let Ok(new_id) = view.inner_mut().data_mut().doc_mut().add_sibling(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view
            .inner_mut()
            .data_mut()
            .visible_nodes()
            .iter()
            .position(|&n| n == new_id)
        {
            view.inner_mut().set_cursor(pos);
        }
        view.group.mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
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
        if let Some(pos) = view
            .inner_mut()
            .data_mut()
            .visible_nodes()
            .iter()
            .position(|&n| n == new_id)
        {
            view.inner_mut().set_cursor(pos);
        }
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

pub fn handle_clone(view: &mut StructuredView) {
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
    if let Ok(new_id) = view.inner_mut().data_mut().doc_mut().clone_node(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view
            .inner_mut()
            .data_mut()
            .visible_nodes()
            .iter()
            .position(|&n| n == new_id)
        {
            view.inner_mut().set_cursor(pos);
        }
        view.group.mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_cycle_type(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.inner_mut().data_mut().doc_mut().cycle_type(node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.group.mark_dirty();
}

pub fn handle_convert_container(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.inner_mut().data_mut().doc_mut().convert_container(node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.group.mark_dirty();
}
