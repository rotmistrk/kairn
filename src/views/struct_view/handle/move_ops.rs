//! Move/reorder operations for StructuredView (swap, promote, demote, sort).

use crate::structured::NodeKind;
use crate::views::struct_view::StructuredView;

pub fn handle_swap_down(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.inner_mut().data_mut().doc_mut().swap_down(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        find_and_set_cursor(view, node_id);
        view.group.mark_dirty();
    }
}

pub fn handle_swap_up(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.inner_mut().data_mut().doc_mut().swap_up(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        find_and_set_cursor(view, node_id);
        view.group.mark_dirty();
    }
}

pub fn handle_promote(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.inner_mut().data_mut().doc_mut().promote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        find_and_set_cursor(view, node_id);
        view.group.mark_dirty();
    }
}

pub fn handle_demote(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if view.inner_mut().data_mut().doc_mut().demote(node_id).is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        find_and_set_cursor(view, node_id);
        view.group.mark_dirty();
    }
}

pub fn handle_toggle_inline(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    view.inner_mut().data_mut().doc_mut().toggle_inline(node_id);
    view.dirty = true;
    view.sync_title();
    view.group.mark_dirty();
}

pub fn handle_sort(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let target = if view.inner_mut().data_mut().doc().node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.inner_mut().data_mut().doc().parent(node_id) {
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
    view.inner_mut().data_mut().doc_mut().sort_children(target, ascending);
    view.dirty = true;
    view.sync_title();
    view.rebuild_visible();
    view.group.mark_dirty();
}

pub fn handle_sort_by_path_start(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    let target = if view.inner_mut().data_mut().doc().node_kind(node_id) != NodeKind::Scalar {
        node_id
    } else {
        match view.inner_mut().data_mut().doc().parent(node_id) {
            Some(p) => p,
            None => return,
        }
    };
    view.sort_path_target = Some(target);
    view.start_input_line(".");
}

fn find_and_set_cursor(view: &mut StructuredView, node_id: crate::structured::NodeId) {
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
