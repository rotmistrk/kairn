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

pub fn handle_sort(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    // Sort the current node if container, otherwise sort parent
    let target = if view.doc.node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.doc.parent(node_id) {
            Some(p) => p,
            None => return,
        }
    };
    // Toggle ascending/descending on repeated press on same node
    let ascending = if view.last_sort_node == Some(target) {
        !view.last_sort_asc
    } else {
        true
    };
    view.last_sort_node = Some(target);
    view.last_sort_asc = ascending;
    view.save_undo_point();
    view.doc.sort_children(target, ascending);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.state.mark_dirty();
}

pub fn handle_sort_by_path_start(view: &mut StructuredView) {
    let Some(&node_id) = view.visible_nodes.get(view.cursor) else {
        return;
    };
    let target = if view.doc.node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.doc.parent(node_id) {
            Some(p) => p,
            None => return,
        }
    };
    view.sort_path_target = Some(target);
    view.editing = Some(txv_widgets::inline_edit::InlineEditor::new(view.cursor, "."));
    view.state.mark_dirty();
}

pub fn handle_filter_start(view: &mut StructuredView) {
    view.filtering = true;
    view.editing = Some(txv_widgets::inline_edit::InlineEditor::new(
        view.cursor,
        &view.filter_text,
    ));
    view.edit_target = EditTarget::Meta; // reuse to distinguish
    view.state.mark_dirty();
}

pub fn handle_filter_clear(view: &mut StructuredView) {
    view.filter_text.clear();
    view.filtering = false;
    view.rebuild_visible();
    view.clamp_cursor();
    view.sync_scroll();
    view.sync_title();
    view.state.mark_dirty();
}
