//! Yank/paste operations for StructuredView.

use crate::views::struct_view::StructuredView;

pub fn handle_yank(view: &mut StructuredView) {
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.yanked = Some(view.inner_mut().data_mut().doc().serialize_node(node_id));
}

pub fn handle_paste(view: &mut StructuredView) {
    let Some(json) = view.yanked.clone() else {
        return;
    };
    let cursor = view.inner().cursor();
    let Some(&node_id) = view.inner_mut().data_mut().visible_nodes().get(cursor) else {
        return;
    };
    view.save_undo_point();
    if let Ok(new_id) = view.inner_mut().data_mut().doc_mut().paste_after(node_id, &json) {
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
    }
}
