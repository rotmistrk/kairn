//! Filter operations for StructuredView.

use super::super::StructuredView;

pub fn handle_filter_start(view: &mut StructuredView) {
    view.filtering = true;
    let ft = view.tree.data_mut().filter_text().to_string();
    view.start_input_line(&ft);
}

pub fn handle_filter_clear(view: &mut StructuredView) {
    view.tree.data_mut().clear_filter_text();
    view.filtering = false;
    view.rebuild_visible();
    view.clamp_cursor();
    view.sync_title();
    view.tree.state_mut().mark_dirty();
}
