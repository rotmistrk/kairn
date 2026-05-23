//! Assembles ViewContext from current state and broadcasts CM_CONTEXT_UPDATE.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::commands::{ViewContext, CM_CONTEXT_UPDATE};
use crate::desktop::SlotId;
use crate::editor::keymap::Keymap;
use crate::handler::downcast_desktop;
use crate::views::editor::EditorView;

/// Collect context from the focused view and broadcast it.
pub fn broadcast_context(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let slot = desktop.focused_slot();
    let title = desktop.active_tab_title(slot).unwrap_or("").to_string();

    let mut vc = ViewContext {
        title,
        git_branch: read_branch(&state.root_dir),
        ..Default::default()
    };

    let mut selection_text = String::new();
    let mut current_line_text = String::new();

    if let Some(view) = desktop.active_view_mut(slot) {
        if let Some(any) = view.as_any_mut() {
            if let Some(editor) = any.downcast_ref::<EditorView>() {
                fill_from_editor(editor, state, &mut vc);
                current_line_text = editor.editor.buf().line(editor.editor.cursor_line).unwrap_or_default();
                if let Some((start, end)) = editor.editor.visual_range() {
                    let content = editor.editor.buf().content();
                    if end <= content.len() {
                        selection_text = content[start..end].to_string();
                    }
                }
            }
        }
    }

    // If no editor, set mode from slot type
    if vc.mode.is_empty() {
        vc.mode = mode_for_slot(slot);
    }

    // Determine split state
    let (split_dir, split_linked) = {
        let panel = desktop.panel_mut(SlotId::Center);
        if let Some(view) = panel.active_view_mut() {
            if let Some(es) = view
                .as_any_mut()
                .and_then(|a| a.downcast_ref::<crate::views::editor_split::EditorSplit>())
            {
                let dir = match es.split.direction() {
                    txv_widgets::tiled_workspace::types::SplitDir::Horizontal => "horizontal",
                    txv_widgets::tiled_workspace::types::SplitDir::Vertical => "vertical",
                };
                (dir, es.linked_scroll)
            } else {
                ("none", false)
            }
        } else {
            ("none", false)
        }
    };

    // Update script engine snapshot
    let root = state.root_dir.to_string_lossy().to_string();
    state
        .script
        .update_snapshot(&vc, &root, &selection_text, &current_line_text, split_dir, split_linked);

    ctx.sink.push_command(CM_CONTEXT_UPDATE, Some(Box::new(vc)));
}

fn fill_from_editor(editor: &EditorView, state: &AppState, vc: &mut ViewContext) {
    let e = &editor.editor;
    vc.line = e.cursor_line as u32 + 1;
    vc.col = e.cursor_col as u32 + 1;
    vc.mode = e.keymap.mode_label(e.mode).to_string();
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
        e.mode,
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
        SlotId::Right | SlotId::Bottom => "TERM".into(),
    }
}

fn read_branch(root: &std::path::Path) -> String {
    let Ok(head) = std::fs::read_to_string(root.join(".git/HEAD")) else {
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
