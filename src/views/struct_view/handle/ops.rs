//! Structural operation handlers for StructuredView.

use crate::structured::NodeKind;
use crate::views::struct_view::{EditTarget, StructuredView};

pub fn handle_new_sibling(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let parent_kind = view
        .tree
        .data()
        .doc()
        .parent(node_id)
        .map(|p| view.tree.data_mut().doc().node_kind(p));
    view.save_undo_point();
    if let Ok(new_id) = view.tree.data_mut().doc_mut().add_sibling(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == new_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_new_child(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    if view.tree.data_mut().doc().node_kind(node_id) == NodeKind::Scalar {
        return;
    }
    view.save_undo_point();
    if let Ok(new_id) = view.tree.data_mut().doc_mut().add_child(node_id) {
        view.dirty = true;
        view.sync_title();
        if !view.tree.data_mut().doc().is_expanded(node_id) {
            view.tree.data_mut().doc_mut().toggle_expand(node_id);
        }
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == new_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_delete(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.tree.data_mut().doc_mut().remove(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        view.clamp_cursor();
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_clone(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let parent_kind = view
        .tree
        .data()
        .doc()
        .parent(node_id)
        .map(|p| view.tree.data_mut().doc().node_kind(p));
    view.save_undo_point();
    if let Ok(new_id) = view.tree.data_mut().doc_mut().clone_node(node_id) {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == new_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
        if parent_kind == Some(NodeKind::Dict) {
            view.start_edit(EditTarget::Key);
        }
    }
}

pub fn handle_cycle_type(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.tree.data_mut().doc_mut().cycle_type(node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.tree.state_mut().mark_dirty();
}

pub fn handle_convert_container(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.tree.data_mut().doc_mut().convert_container(node_id);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.tree.state_mut().mark_dirty();
}

pub fn handle_swap_down(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.tree.data_mut().doc_mut().swap_down(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == node_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_swap_up(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.tree.data_mut().doc_mut().swap_up(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == node_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_promote(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.tree.data_mut().doc_mut().promote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == node_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_demote(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.tree.data_mut().doc_mut().demote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        if let Some(pos) = view.tree.data_mut().visible_nodes().iter().position(|&n| n == node_id) {
            view.tree.set_cursor(pos);
        }
        view.tree.state_mut().mark_dirty();
    }
}

pub fn handle_toggle_inline(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.tree.data_mut().doc_mut().toggle_inline(node_id);
    view.dirty = true;
    view.sync_title();
    view.tree.state_mut().mark_dirty();
}

pub fn handle_sort(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let target = if view.tree.data_mut().doc().node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.tree.data_mut().doc().parent(node_id) {
            Some(p) => p,
            None => return,
        }
    };
    let ascending = if view.last_sort_node == Some(target) {
        !view.last_sort_asc
    } else {
        true
    };
    view.last_sort_node = Some(target);
    view.last_sort_asc = ascending;
    view.save_undo_point();
    view.tree.data_mut().doc_mut().sort_children(target, ascending);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.tree.state_mut().mark_dirty();
}

pub fn handle_sort_by_path_start(view: &mut StructuredView) {
    let cursor = view.tree.cursor();
    let Some(&node_id) = view.tree.data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let target = if view.tree.data_mut().doc().node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.tree.data_mut().doc().parent(node_id) {
            Some(p) => p,
            None => return,
        }
    };
    view.sort_path_target = Some(target);
    view.start_input_line(".");
}
