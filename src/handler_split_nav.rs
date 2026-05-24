//! Handler logic for diff-split and open-in-split (navigation) commands.

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::views::editor::diff_model::{build_diff_lines, DiffLine, DiffOpts, DiffState};
use crate::views::editor::sbs_model::{split_for_side_by_side, SbsDiffState};
use crate::views::editor::EditorView;
use crate::views::editor_split::EditorSplit;
use crate::views::scroll_map::ScrollMap;

pub(crate) fn handle_diff_split(ctx: &mut CommandContext, _state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::DiffSplitRequest>() else {
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
    let active_idx = panel.active_index();
    let title = panel.active_title().map(String::from).unwrap_or_default();
    let Some(mut current_view) = panel.take_tab(active_idx) else {
        return;
    };

    // Create a read-only editor with the base content
    let mut base_ev = EditorView::from_text(&base_content);
    base_ev.editor.status = format!("[{base_ref}]");

    // Get current content for scroll map before wrapping in split
    let current_content = current_view
        .as_any_mut()
        .and_then(|a| a.downcast_ref::<EditorView>())
        .map(|ev| ev.editor.buf().content());

    // When files are identical, don't split — show folded summary in current pane
    let identical = current_content.as_deref() == Some(&base_content);
    if identical {
        let line_count = base_content.lines().count();
        if let Some(ev) = current_view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
            ev.set_diff_state(DiffState {
                lines: vec![DiffLine::Folded { count: line_count }],
                scroll: 0,
                cursor: 0,
                base_ref: base_ref.clone(),
                context_lines: 2,
                ignore_ws: false,
            });
            ev.editor.status = format!("[no changes vs {}]", base_ref);
        }
        panel.insert_tab_at(active_idx, &title, current_view);
        return;
    }

    // Non-identical: build side-by-side diff
    if let Some(current_text) = current_content.as_deref() {
        let opts = DiffOpts {
            base: base_ref.clone(),
            context: 3,
            ignore_ws: false,
            side_by_side: false,
        };
        let unified = build_diff_lines(&base_content, current_text, &opts);
        let (left_lines, right_lines) = split_for_side_by_side(&unified, &base_content, current_text);

        base_ev.set_sbs_state(SbsDiffState {
            lines: left_lines,
            scroll: 0,
            cursor: 0,
            base_ref: base_ref.clone(),
            is_left: true,
        });
        base_ev.editor.status = format!("[{base_ref}]");

        if let Some(ev) = current_view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
            ev.set_sbs_state(SbsDiffState {
                lines: right_lines,
                scroll: 0,
                cursor: 0,
                base_ref: base_ref.clone(),
                is_left: false,
            });
            ev.editor.status = format!("[DIFF vs {}]", base_ref);
        }
    }

    let map = current_content
        .as_ref()
        .map(|current| ScrollMap::from_diff(&base_content, current));
    let mut split = EditorSplit::new(SplitDir::Horizontal, Box::new(base_ev), current_view);
    split.set_linked_scroll(true, map);
    // Focus the right (current) pane
    split.set_focused(1);
    panel.insert_tab_at(active_idx, &title, Box::new(split));
    panel.set_active(active_idx);
}

pub(crate) fn handle_open_in_split(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::OpenFileRequest>() else {
        return;
    };
    let path = req.path.clone();
    let line = req.line.unwrap_or(0);
    let col = req.col.unwrap_or(0);

    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };

    // If already in a split, navigate the unfocused pane
    if let Some(view) = panel.active_view_mut() {
        if let Some(es) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorSplit>()) {
            let other_idx = 1 - es.focused_index();
            if let Some(child) = es.child_mut(other_idx) {
                if let Some(ev) = child.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                    open_into_editor(ev, &path, line, col, state);
                    let word_range = word_cols_at(ev, line as usize, col as usize);
                    ev.highlight_word = Some((line as usize, word_range.0, word_range.1));
                    return;
                }
            }
        }
    }

    // Not in a split — create one, then open target in the new pane
    let active_idx = panel.active_index();
    let title = panel.active_title().map(String::from).unwrap_or_default();
    let Some(existing) = panel.take_tab(active_idx) else {
        return;
    };

    let mut new_pane = open_new_pane(state, &path, line, col);
    let word_range = if let Some(ev) = new_pane.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
        let wr = word_cols_at(ev, line as usize, col as usize);
        ev.highlight_word = Some((line as usize, wr.0, wr.1));
        wr
    } else {
        (col as usize, col as usize + 1)
    };
    let _ = word_range;

    let mut split = EditorSplit::new(SplitDir::Horizontal, new_pane, existing);
    // Focus the second pane (right — where the user was editing)
    split.set_focused(1);
    panel.insert_tab_at(active_idx, &title, Box::new(split));
    panel.set_active(active_idx);
}

pub(crate) fn open_into_editor(ev: &mut EditorView, path: &std::path::Path, line: u32, col: u32, state: &mut AppState) {
    let bounds = ev.bounds();
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let new_ev = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    *ev = new_ev;
    ev.set_bounds(bounds);
    ev.set_root_dir(state.root_dir.clone());
    ev.goto(line, col);
}

fn open_new_pane(state: &mut AppState, path: &std::path::Path, line: u32, col: u32) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut ev = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    ev.set_root_dir(state.root_dir.clone());
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
