//! Structural operation handlers for StructuredView.

use crate::structured::NodeKind;
use crate::views::struct_view::{EditTarget, StructuredView};

pub fn handle_new_sibling(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    let parent_kind = view.doc.parent(node_id).map(|p| view.doc.node_kind(p));
    view.save_undo_point();
    if let Ok(new_id) = view.doc.add_sibling(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == new_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_new_child(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    if view.doc.node_kind(node_id) == NodeKind::Scalar {
        return;
    }
    view.save_undo_point();
    if let Ok(new_id) = view.doc.add_child(node_id) {
        view.dirty = true;
        view.sync_title();
        if !view.doc.is_expanded(node_id) {
            view.doc.toggle_expand(node_id);
        }
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == new_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_delete(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    if view.doc.remove(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        view.clamp_cursor();
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_clone(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    let parent_kind = view.doc.parent(node_id).map(|p| view.doc.node_kind(p));
    view.save_undo_point();
    if let Ok(new_id) = view.doc.clone_node(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == new_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_cycle_type(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    view.doc.cycle_type(node_id);
    view.dirty = true;
    view.sync_title();
    view.state.mark_dirty();
}

pub fn handle_convert_container(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    view.doc.convert_container(node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.state.mark_dirty();
}

pub fn handle_swap_down(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    if view.doc.swap_down(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == node_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_swap_up(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    if view.doc.swap_up(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == node_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_promote(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    if view.doc.promote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == node_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_demote(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    if view.doc.demote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.visible_nodes.iter().position(|&n| n == node_id) {
            view.cursor = pos;
        }
        view.sync_scroll();
        view.state.mark_dirty();
    }
}

pub fn handle_toggle_inline(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    view.save_undo_point();
    view.doc.toggle_inline(node_id);
    view.dirty = true;
    view.sync_title();
    view.state.mark_dirty();
}
