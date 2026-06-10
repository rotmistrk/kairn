//! Script-triggered editor mutations: replace-selection, delete-line, replace-word.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::editor::command::Command;
use crate::editor::keymap::EditorMode;
use crate::handler::downcast_desktop;
use crate::views::editor::EditorView;

/// Handle CM_EDITOR_REPLACE_SELECTION — replace visual selection with text.
pub fn handle_replace_selection(ctx: &mut CommandContext, _state: &AppState) {
    let text = {
        let Some(t) = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()) else {
            return;
        };
        t.clone()
    };
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    if let Some((start, end)) = editor.editor.visual_range() {
        editor.editor.buf().delete(start, end);
        editor.editor.buf().insert(start, &text);
        let (l, c) = editor.editor.buf().offset_to_line_col(start + text.len());
        editor.editor.set_cursor_line(l);
        editor.editor.set_cursor_col(c);
        editor.editor.set_mode(EditorMode::Normal);
        editor.editor.set_visual_anchor(None);
    }
}

/// Handle CM_EDITOR_DELETE_LINE — delete a specific line.
pub fn handle_delete_line(ctx: &mut CommandContext, _state: &AppState) {
    let line = ctx
        .data()
        .as_ref()
        .and_then(|d| d.downcast_ref::<Option<u32>>())
        .copied()
        .flatten();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    let target = line
        .map(|n| n.saturating_sub(1) as usize)
        .unwrap_or(editor.editor.cursor_line());
    let start = editor.editor.buf().line_col_to_offset(target, 0);
    let end = if target + 1 < editor.editor.buf().line_count() {
        editor.editor.buf().line_col_to_offset(target + 1, 0)
    } else {
        Some(editor.editor.buf().len())
    };
    if let (Some(s), Some(e)) = (start, end) {
        if e > s {
            editor.editor.buf().delete(s, e);
            editor.editor.clamp_cursor();
        }
    }
}

/// Handle CM_EDITOR_REPLACE_WORD — replace word under cursor.
pub fn handle_replace_word(ctx: &mut CommandContext, _state: &AppState) {
    let text = {
        let Some(t) = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()) else {
            return;
        };
        t.clone()
    };
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    let line_content = editor
        .editor
        .buf()
        .line(editor.editor.cursor_line())
        .unwrap_or_default();
    let chars: Vec<char> = line_content.chars().collect();
    let col = editor.editor.cursor_col();
    if col >= chars.len() || !chars[col].is_alphanumeric() && chars[col] != '_' {
        return;
    }
    let start = col - (0..col).rev().take_while(|&i| is_word(chars[i])).count();
    let end = col + (col..chars.len()).take_while(|&i| is_word(chars[i])).count();
    let line_start = editor
        .editor
        .buf()
        .line_col_to_offset(editor.editor.cursor_line(), start);
    let line_end = editor.editor.buf().line_col_to_offset(editor.editor.cursor_line(), end);
    if let (Some(s), Some(e)) = (line_start, line_end) {
        editor.editor.buf().delete(s, e);
        editor.editor.buf().insert(s, &text);
        editor.editor.set_cursor_col(start + text.chars().count());
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Handle CM_EDITOR_SEARCH — set search pattern and highlight matches.
pub fn handle_search(ctx: &mut CommandContext, _state: &AppState, pattern: &str) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    editor.editor.set_search_pattern(pattern.to_string());
    editor.editor.update_highlight();
}

/// Handle CM_EDITOR_CLEAR_HIGHLIGHT — clear search highlights.
pub fn handle_clear_highlight(ctx: &mut CommandContext, _state: &AppState) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    editor.editor.set_highlight(None);
}

pub fn handle_editor_set(ctx: &mut CommandContext, option: &str) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };
    let cmd = format!("set {option}");
    editor.editor.execute(Command::ExCommand(cmd));
}
