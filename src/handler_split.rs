//! Handler logic for :split/:vsplit/:only commands.
//!
//! Uses TiledWorkspace's native subpanel mechanism: the center panel's
//! SplitPanel gets a second TabPanel child when split.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::commands::SplitRequest;
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_split_nav::open_into_editor;
use crate::views::editor::EditorView;

pub(crate) fn handle_split(ctx: &mut CommandContext, state: &mut AppState) {
    let (vertical, file) = {
        let Some(boxed) = ctx.data().as_ref() else {
            return;
        };
        let Some(req) = boxed.downcast_ref::<SplitRequest>() else {
            return;
        };
        (req.vertical, req.file.clone())
    };
    let direction = if vertical {
        SplitDir::Horizontal
    } else {
        SplitDir::Vertical
    };
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    do_split_inner(desktop, state, &sink, direction, file.as_deref());
}

fn do_split_inner(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    _sink: &EventSink,
    direction: SplitDir,
    file: Option<&str>,
) {
    let focused = desktop.focused_panel();
    if focused != SlotId::Center as usize {
        desktop.move_tab_to_subpanel();
        return;
    }

    let is_split = desktop
        .split_panel(SlotId::Center as usize)
        .map(|sp| sp.child_count() > 1)
        .unwrap_or(false);

    if is_split {
        handle_split_existing(desktop, state, file.map(|s| s.to_string()), direction);
    } else {
        handle_split_new(desktop, state, file.map(|s| s.to_string()), direction);
    }
}

fn handle_split_existing(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    file: Option<String>,
    direction: SplitDir,
) {
    if let Some(ref filename) = file {
        open_in_other_subpanel(desktop, state, filename);
    } else if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        sp.set_direction(direction);
    }
}

fn handle_split_new(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    file: Option<String>,
    direction: SplitDir,
) {
    let title = desktop
        .panel(SlotId::Center as usize)
        .and_then(|p| p.active_title().map(String::from))
        .unwrap_or_default();

    let new_pane: Box<dyn View> = if let Some(ref filename) = file {
        open_second_file(state, filename)
    } else {
        let pane = match desktop.panel_mut(SlotId::Center as usize) {
            Some(p) => create_shared_pane(p, state),
            None => return,
        };
        pane
    };

    if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        sp.set_direction(direction);
    }
    desktop.split_in_place(new_pane, &title);
}

pub(crate) fn handle_split_close(ctx: &mut CommandContext, _state: &mut AppState) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    desktop.collapse_other_subpanel();
}

pub(crate) fn handle_split_focus(ctx: &mut CommandContext) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    desktop.with_split_panel(|sp| sp.cycle_focus());
}

pub(crate) fn handle_split_linked(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(&on) = boxed.downcast_ref::<bool>() else {
        return;
    };
    state.linked_scroll = on;
}

fn open_in_other_subpanel(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    filename: &str,
) {
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    let other_idx = 1 - sp.focused_index();
    let Some(other_child) = sp.child_mut(other_idx) else {
        return;
    };
    let Some(other_tp) = other_child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
        return;
    };
    let Some(view) = other_tp.active_view_mut() else {
        return;
    };
    let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    open_into_editor(ev, &state.root_dir.join(filename), 0, 0, state);
}

fn create_shared_pane(panel: &mut TabPanel, state: &mut AppState) -> Box<dyn View> {
    let Some(view) = panel.active_view_mut() else {
        return Box::new(EditorView::from_text(""));
    };
    let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return Box::new(EditorView::from_text(""));
    };
    let defaults = state.settings.editor_defaults.clone();
    let syntax_theme = state.current_syntax_theme().to_string();
    let buf_id = ev.buffer_id;
    let shared_buf = ev.editor.buffer_arc();
    let file_path = ev.editor.buf().file_path().map(|s| s.to_string());
    let cursor_line = ev.editor.cursor_line();
    let cursor_col = ev.editor.cursor_col();
    let scroll = ev.editor.viewport_scroll();
    let ev_path = ev.path().to_path_buf();
    let mut ed = EditorView::from_arc_buffer(shared_buf, file_path, &defaults, &syntax_theme);
    ed.set_root_dir(state.roots().root_for(&ev_path).path().to_path_buf());
    ed.editor_mut()
        .set_shared_state(state.shared_register.clone(), state.clipboard.clone());
    ed.buffer_id = buf_id;
    ed.editor.set_cursor_line(cursor_line);
    ed.editor.set_cursor_col(cursor_col);
    ed.editor.set_viewport_scroll(scroll);
    Box::new(ed)
}

fn open_second_file(state: &mut AppState, filename: &str) -> Box<dyn View> {
    let path = state.root_dir.join(filename);
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ed = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
    ed.set_root_dir(state.roots().root_for(&path).path().to_path_buf());
    ed.editor_mut()
        .set_shared_state(state.shared_register.clone(), state.clipboard.clone());
    let canon = path.canonicalize().unwrap_or(path);
    ed.buffer_id = Some(state.buffers.register(Some(canon)));
    Box::new(ed)
}

pub(crate) fn handle_split_h(ctx: &mut CommandContext, state: &mut AppState) {
    let req = SplitRequest {
        vertical: false,
        file: None,
    };
    do_split(ctx, state, req);
}

pub(crate) fn handle_split_v(ctx: &mut CommandContext, state: &mut AppState) {
    let req = SplitRequest {
        vertical: true,
        file: None,
    };
    do_split(ctx, state, req);
}

fn do_split(ctx: &mut CommandContext, state: &mut AppState, req: SplitRequest) {
    let vertical = req.vertical;
    let file = req.file.clone();
    let direction = if vertical {
        SplitDir::Horizontal
    } else {
        SplitDir::Vertical
    };
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    do_split_inner(desktop, state, &sink, direction, file.as_deref());
}
