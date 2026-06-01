//! Handler logic for diff-split and open-in-split (navigation) commands.
//!
//! Side-by-side diff is a rendering mode within a single EditorView (no split).
//! Open-in-split uses TiledWorkspace's native subpanel mechanism.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::commands::{DiffSplitRequest, OpenFileRequest};
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::views::editor::diff_model::{build_diff_lines, DiffLine, DiffOpts, DiffState};
use crate::views::editor::sbs_model::{split_for_side_by_side, SbsDiffState};
use crate::views::editor::EditorView;

pub(crate) fn handle_diff_split(ctx: &mut CommandContext, _state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<DiffSplitRequest>() else {
        return;
    };
    let base_content = req.base_content.clone();
    let base_ref = req.base_ref.clone();
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };

    let current_content = panel
        .active_view_mut()
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_ref::<EditorView>())
        .map(|ev| ev.editor.buf().content());

    let identical = current_content.as_deref() == Some(&base_content);
    if identical {
        set_identical_diff(panel, &base_content, &base_ref);
        return;
    }

    if let Some(current_text) = current_content.as_deref() {
        set_sbs_diff(panel, &base_content, current_text, &base_ref);
    }
}

fn set_identical_diff(panel: &mut txv_widgets::tab_panel::TabPanel, base_content: &str, base_ref: &str) {
    let line_count = base_content.lines().count();
    if let Some(view) = panel.active_view_mut() {
        if let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
            ev.set_diff_state(DiffState {
                lines: vec![DiffLine::Folded { count: line_count }],
                scroll: 0,
                cursor: 0,
                base_ref: base_ref.to_string(),
                context_lines: 2,
                ignore_ws: false,
            });
            ev.editor.status = format!("[no changes vs {}]", base_ref);
        }
    }
}

fn set_sbs_diff(panel: &mut txv_widgets::tab_panel::TabPanel, base_content: &str, current_text: &str, base_ref: &str) {
    let opts = DiffOpts {
        base: base_ref.to_string(),
        context: 3,
        ignore_ws: false,
        side_by_side: false,
    };
    let unified = build_diff_lines(base_content, current_text, &opts);
    let (left_lines, right_lines) = split_for_side_by_side(&unified, base_content, current_text);

    if let Some(view) = panel.active_view_mut() {
        if let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
            ev.set_sbs_state(SbsDiffState {
                left: left_lines,
                right: right_lines,
                scroll: 0,
                cursor: 0,
                base_ref: base_ref.to_string(),
            });
            ev.editor.status = format!("[DIFF vs {}]", base_ref);
        }
    }
}

pub(crate) fn handle_open_in_split(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<OpenFileRequest>() else {
        return;
    };
    let path = req.path.clone();
    let line = req.line.unwrap_or(0);
    let col = req.col.unwrap_or(0);

    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };

    let is_split = desktop
        .split_panel(SlotId::Center as usize)
        .map(|sp| sp.child_count() > 1)
        .unwrap_or(false);

    if is_split {
        open_in_existing_split(desktop, state, &path, line, col);
    } else {
        create_split_with_file(desktop, state, &path, line, col);
    }
}

fn open_in_existing_split(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    path: &std::path::Path,
    line: u32,
    col: u32,
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
    open_into_editor(ev, path, line, col, state);
    let word_range = word_cols_at(ev, line as usize, col as usize);
    ev.highlight_word = Some((line as usize, word_range.0, word_range.1));
}

fn create_split_with_file(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    path: &std::path::Path,
    line: u32,
    col: u32,
) {
    let mut new_pane = open_new_pane(state, path, line, col);
    if let Some(ev) = new_pane.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
        let wr = word_cols_at(ev, line as usize, col as usize);
        ev.highlight_word = Some((line as usize, wr.0, wr.1));
    }

    let title = desktop
        .panel(SlotId::Center as usize)
        .and_then(|p| p.active_title().map(String::from))
        .unwrap_or_default();

    if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        sp.set_direction(SplitDir::Horizontal);
    }
    desktop.split_in_place(new_pane, &title);
    if let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) {
        sp.set_focused(0);
    }
}

pub(crate) fn open_into_editor(ev: &mut EditorView, path: &std::path::Path, line: u32, col: u32, state: &mut AppState) {
    let bounds = ev.bounds();
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let new_ev = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    *ev = new_ev;
    ev.set_bounds(bounds);
    ev.set_root_dir(state.roots().root_for(path).path().to_path_buf());
    ev.goto(line, col);
}

fn open_new_pane(state: &mut AppState, path: &std::path::Path, line: u32, col: u32) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ev = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    ev.set_root_dir(state.roots().root_for(path).path().to_path_buf());
    ev.goto(line, col);
    Box::new(ev)
}

/// Find word boundaries (col_start, col_end) at a given position.
fn word_cols_at(ev: &EditorView, line: usize, col: usize) -> (usize, usize) {
    let text = ev.editor.buf().line(line).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    if col >= chars.len() {
        return (col, col + 1);
    }
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if !is_word(chars[col]) {
        return (col, col + 1);
    }
    let start = (0..col).rev().take_while(|&i| is_word(chars[i])).count();
    let begin = col - start;
    let end = (col..chars.len()).take_while(|&i| is_word(chars[i])).count() + col;
    (begin, end)
}
