//! Handler logic for :split/:vsplit/:only commands.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::split_pane::SplitDirection;

use crate::commands::SplitRequest;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;
use crate::views::editor::EditorView;
use crate::views::editor_split::EditorSplit;

pub(crate) fn handle_split(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<SplitRequest>() else {
        return;
    };
    let vertical = req.vertical;
    let file = req.file.clone();
    let direction = if vertical {
        SplitDirection::Horizontal
    } else {
        SplitDirection::Vertical
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let panel = desktop.panel_mut(SlotId::Center);

    // If already in a split, just change orientation
    if let Some(view) = panel.active_view_mut() {
        if let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) {
            es.set_direction(direction);
            return;
        }
    }

    // Not split yet — take the current tab and wrap it in a split
    let active_idx = panel.active_index();
    let title = panel.active_title().map(String::from).unwrap_or_default();
    let Some(removed) = panel.take_tab(active_idx) else {
        return;
    };

    let second: Box<dyn View> = if let Some(ref filename) = file {
        open_second_file(state, filename)
    } else {
        open_same_file(state, &title)
    };

    let split = EditorSplit::new(direction, removed, second);
    panel.insert_tab_at(active_idx, &title, Box::new(split));
}

pub(crate) fn handle_split_close(ctx: &mut CommandContext, _state: &mut AppState) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let panel = desktop.panel_mut(SlotId::Center);
    let active_idx = panel.active_index();
    let title = panel.active_title().map(String::from).unwrap_or_default();

    // Get the focused child out of the split
    let focused_child = {
        let Some(view) = panel.active_view_mut() else {
            return;
        };
        let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) else {
            return;
        };
        es.take_focused()
    };
    let Some(child) = focused_child else {
        return;
    };

    // Replace the split tab with the focused child
    panel.remove_tab(active_idx);
    panel.insert_tab_at(active_idx, &title, child);
}

fn open_second_file(state: &mut AppState, filename: &str) -> Box<dyn View> {
    let path = state.root_dir.join(filename);
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ed = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
    ed.set_root_dir(state.root_dir.clone());
    let canon = path.canonicalize().unwrap_or(path);
    ed.buffer_id = Some(state.buffers.register(Some(canon)));
    Box::new(ed)
}

fn open_same_file(state: &mut AppState, title: &str) -> Box<dyn View> {
    let path = state.root_dir.join(title);
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ed = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
    ed.set_root_dir(state.root_dir.clone());
    let canon = path.canonicalize().unwrap_or(path);
    if let Some(buf_id) = state.buffers.find_by_path(&canon) {
        state.buffers.add_ref(buf_id);
        ed.buffer_id = Some(buf_id);
    }
    Box::new(ed)
}
