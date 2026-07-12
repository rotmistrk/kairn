//! Move/reorder operations for StructuredView (swap, promote, demote, sort).

use crate::structured::{NodeId, NodeKind};
use crate::views::struct_view::StructuredView;

/// Apply a doc operation that moves a node (swap/promote/demote).
/// Gets cursor node, saves undo, calls the operation, then rebuilds
/// and re-locates the cursor on the same node.
fn apply_move_op(
    view: &mut StructuredView,
    op: fn(&mut dyn crate::structured::StructuredDoc, NodeId) -> Result<(), String>,
) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    let result = op(&mut **view.inner_mut().data_mut().doc_mut(), node_id);
    if result.is_ok() {
        view.dirty = true;
        view.sync_title();
        view.rebuild_visible();
        find_and_set_cursor(view, node_id);
        view.group.mark_dirty();
    }
}

pub fn handle_swap_down(view: &mut StructuredView) {
    apply_move_op(view, |doc, id| doc.swap_down(id));
}

pub fn handle_swap_up(view: &mut StructuredView) {
    apply_move_op(view, |doc, id| doc.swap_up(id));
}

pub fn handle_promote(view: &mut StructuredView) {
    apply_move_op(view, |doc, id| doc.promote(id));
}

pub fn handle_demote(view: &mut StructuredView) {
    apply_move_op(view, |doc, id| doc.demote(id));
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

fn find_and_set_cursor(view: &mut StructuredView, node_id: NodeId) {
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
