//! Handler logic for :split/:vsplit/:only commands.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::commands::SplitRequest;
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
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
        SplitDir::Horizontal
    } else {
        SplitDir::Vertical
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };

    // If focused panel is not Center, split via move-tab-to-subpanel
    let focused = desktop.focused_panel();
    if focused != SlotId::Center as usize {
        desktop.move_tab_to_subpanel();
        return;
    }

    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };

    // If already in a split
    if let Some(view) = panel.active_view_mut() {
        if let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) {
            if let Some(ref filename) = file {
                // Open file in the other pane
                let other_idx = 1 - es.focused_index();
                if let Some(child) = es.child_mut(other_idx) {
                    if let Some(ev) = child.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                        crate::handler_split_nav::open_into_editor(ev, &state.root_dir.join(filename), 0, 0, state);
                    }
                }
            } else {
                // No file arg — toggle orientation
                es.set_direction(direction);
            }
            return;
        }
    }

    // Not split yet — take the current tab and wrap it in a split
    // New pane goes first (top/left), existing stays second (bottom/right)
    let active_idx = panel.active_index();
    let title = panel.active_title().map(String::from).unwrap_or_default();
    let Some(mut existing) = panel.take_tab(active_idx) else {
        return;
    };

    let new_pane: Box<dyn View> = if let Some(ref filename) = file {
        open_second_file(state, filename)
    } else {
        open_same_file_shared(&mut existing, state)
    };

    // new_pane = first (top/left), existing = second (bottom/right)
    let mut split = EditorSplit::new(direction, new_pane, existing);
    // Focus the second pane (bottom/right) where the user was editing
    split.set_focused(1);
    panel.insert_tab_at(active_idx, &title, Box::new(split));
    panel.set_active(active_idx);
}

pub(crate) fn handle_split_close(ctx: &mut CommandContext, _state: &mut AppState) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
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

pub(crate) fn handle_split_focus(ctx: &mut CommandContext) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let Some(view) = panel.active_view_mut() else {
        return;
    };
    let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) else {
        return;
    };
    es.cycle_focus();
}

pub(crate) fn handle_split_linked(ctx: &mut CommandContext) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(&on) = boxed.downcast_ref::<bool>() else {
        return;
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    let Some(view) = panel.active_view_mut() else {
        return;
    };
    let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) else {
        return;
    };
    es.set_linked_scroll(on, None);
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

fn open_same_file_shared(first: &mut Box<dyn View>, state: &mut AppState) -> Box<dyn View> {
    let first_ev = first.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>());
    let Some(first_ev) = first_ev else {
        return Box::new(EditorView::from_text(""));
    };
    let defaults = state.settings.editor_defaults.clone();
    let syntax_theme = state.current_syntax_theme().to_string();
    let buf_id = first_ev.buffer_id;
    let shared_buf = first_ev.editor.buffer.clone();
    let file_path = first_ev.editor.buf().file_path.clone();
    let mut ed = EditorView::from_arc_buffer(shared_buf, file_path, &defaults, &syntax_theme);
    ed.set_root_dir(state.root_dir.clone());
    ed.buffer_id = buf_id;
    Box::new(ed)
}
