//! Assembles ViewContext from current state and broadcasts CM_CONTEXT_UPDATE.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;

use txv_core::message::{Message, MsgLevel};
use txv_core::program::CommandContext;
use txv_core::view::EventSink;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::app_state::AppState;
use crate::commands::{ViewContext, CM_CONTEXT_UPDATE};
use crate::desktop::{active_tab_title, slot_from, SlotId};
use crate::editor::keymap::Keymap;
use crate::handler::downcast_desktop;
use crate::views::editor::{EditorView, EditorViewExt};

/// Collect context from the focused view and broadcast it.
pub fn broadcast_context(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = slot_from(desktop.focused_panel());
    let title = active_tab_title(desktop, slot).unwrap_or("").to_string();

    let mut vc = ViewContext {
        title,
        git_branch: read_branch(&state.root_dir),
        ..Default::default()
    };

    let (selection_text, current_line_text) = collect_editor_context(desktop, slot, state, &mut vc);

    if vc.mode.is_empty() {
        vc.mode = mode_for_slot(slot);
    }

    let (split_dir, split_linked) = collect_split_state(desktop, state);

    let root = state.root_dir.to_string_lossy().to_string();
    let busy_count = state.kiro_registry().count();
    state
        .script
        .update_snapshot(&vc, &root, &selection_text, &current_line_text, split_dir, split_linked);
    state.script.set_busy_count(busy_count);
    let root_paths: Vec<&str> = state
        .roots()
        .all()
        .iter()
        .map(|r| r.path.to_str().unwrap_or(""))
        .collect();
    state.script.set_roots(&root_paths);

    sink.push_command(CM_CONTEXT_UPDATE, Some(Box::new(vc)));
}

fn collect_editor_context(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    slot: SlotId,
    state: &AppState,
    vc: &mut ViewContext,
) -> (String, String) {
    let mut selection_text = String::new();
    let mut current_line_text = String::new();
    let editor = desktop
        .panel_mut(slot as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_ref::<EditorView>());
    let Some(editor) = editor else {
        return (selection_text, current_line_text);
    };
    fill_from_editor(editor, state, vc);
    current_line_text = editor
        .editor()
        .buf()
        .line(editor.editor().cursor_line())
        .unwrap_or_default();
    if let Some((start, end)) = editor.editor().visual_range() {
        let content = editor.editor().buf().content();
        if end <= content.len() {
            selection_text = content[start..end].to_string();
        }
    }
    (selection_text, current_line_text)
}

fn collect_split_state(
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &AppState,
) -> (&'static str, bool) {
    let is_split = desktop
        .split_panel(SlotId::Center as usize)
        .map(|sp| sp.child_count() > 1)
        .unwrap_or(false);
    if is_split {
        let dir = desktop
            .split_panel(SlotId::Center as usize)
            .map(|sp| match sp.direction() {
                SplitDir::Horizontal => "horizontal",
                SplitDir::Vertical => "vertical",
            })
            .unwrap_or("none");
        (dir, state.linked_scroll)
    } else {
        ("none", false)
    }
}

fn fill_from_editor(editor: &EditorView, state: &AppState, vc: &mut ViewContext) {
    let e = &editor.editor();
    vc.line = e.cursor_line() as u32 + 1;
    vc.col = e.cursor_col() as u32 + 1;
    vc.total_lines = e.buf().line_count() as u32;
    vc.mode = e.keymap().mode_label(e.mode()).to_string();
    vc.modified = e.buf().is_dirty();
    vc.language = editor.language().to_string();
    vc.file = Some(
        editor
            .path()
            .strip_prefix(&state.root_dir)
            .unwrap_or(editor.path())
            .to_string_lossy()
            .into_owned(),
    );
    if matches!(
        e.mode(),
        crate::editor::keymap::EditorMode::Visual | crate::editor::keymap::EditorMode::VisualLine
    ) {
        if let Some((start, end)) = e.visual_range() {
            let sl = e.buf().offset_to_line_col(start).0;
            let el = e.buf().offset_to_line_col(end).0;
            vc.selection_lines = (el - sl + 1) as u32;
        }
    }
    let lang = editor.language();
    if state.lsp.has_config(lang) {
        vc.lsp_status = "ready".to_string();
    }
}

fn mode_for_slot(slot: SlotId) -> String {
    match slot {
        SlotId::Left => "TREE".into(),
        SlotId::Center => "NOR".into(),
        SlotId::Tools => "TERM".into(),
    }
}

fn read_branch(root: &std::path::Path) -> String {
    let Ok(head) = fs::read_to_string(root.join(".git/HEAD")) else {
        return String::new();
    };
    let head = head.trim();
    if let Some(r) = head.strip_prefix("ref: refs/heads/") {
        r.to_string()
    } else if head.len() >= 7 {
        head[..7].to_string()
    } else {
        String::new()
    }
}

pub(crate) fn handle_cursor_moved(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(boxed) = ctx.data().as_ref() {
        if let Some(pos) = boxed.downcast_ref::<txv_widgets::CursorPos>() {
            state.cursor_pos = (pos.line().saturating_sub(1), pos.col().saturating_sub(1));
        }
    }
}

/// Evaluate window.title-expr and emit OSC 2 if the title changed.
pub(crate) fn update_window_title(state: &mut AppState, sink: &EventSink) {
    let expr = state
        .script
        .interpreter()
        .get_var("window.title-expr")
        .map(|v| v.to_string());
    let Some(expr) = expr else {
        return;
    };
    if expr.is_empty() {
        return;
    }
    let title = match state.script.subst(&expr) {
        Ok(t) => t,
        Err(e) => {
            let msg = Message::new(MsgLevel::Error, "title", format!("eval: {e}"));
            sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            return;
        }
    };
    if title != state.last_window_title {
        state.last_window_title = title.clone();
        if let Some(tty) = state.tty_file.as_mut() {
            let _ = write!(tty, "\x1b]2;{}\x07", title);
        }
    }
}

/// Open /dev/tty for OSC 2 title writes, unless the terminal doesn't support it.
pub(crate) fn open_tty_for_title() -> Option<File> {
    let term = env::var("TERM").unwrap_or_default();
    if term == "linux" || term == "dumb" || term.is_empty() {
        return None;
    }
    OpenOptions::new().write(true).open("/dev/tty").ok()
}
