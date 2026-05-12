//! File-open command handlers — CM_OPEN_FILE, :edit.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::OpenResult;
use crate::commands::*;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;
use crate::views::editor::EditorView;

pub(crate) fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState, focus_center: bool) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::OpenFileRequest>() else {
        return;
    };
    let path = &req.path;
    let path_str = path.to_string_lossy().to_string();

    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let title = path
                    .strip_prefix(&state.root_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                desktop.focus_tab_by_title(SlotId::Center, &title);
                if let (Some(line), Some(col)) = (req.line, req.col) {
                    if let Some(view) = desktop.active_view_mut(SlotId::Center) {
                        if let Some(editor) = view
                            .as_any_mut()
                            .and_then(|a| a.downcast_mut::<crate::views::editor::EditorView>())
                        {
                            editor.goto(line, col);
                        }
                    }
                }
                if focus_center {
                    desktop.focus_slot(SlotId::Center);
                }
            }
            if req.diff {
                ctx.queue.put_command(CM_DIFF, Some(Box::new(String::new())));
            }
        }
        OpenResult::Opened => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.close_tab_by_title(SlotId::Center, "Welcome");
                let defaults = &state.settings.editor_defaults;
                let mut editor =
                    EditorView::open(path, defaults).unwrap_or_else(|_| EditorView::new_file(path, defaults));
                editor.set_root_dir(state.root_dir.clone());
                if let (Some(line), Some(col)) = (req.line, req.col) {
                    editor.goto(line, col);
                }
                if req.diff {
                    editor.toggle_diff("");
                }
                let title = path
                    .strip_prefix(&state.root_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                desktop.insert_tab(SlotId::Center, &title, Box::new(editor));
                if focus_center {
                    desktop.focus_slot(SlotId::Center);
                }
                ctx.queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::info("editor", format!("Opened: {title}")))),
                );
            }
        }
    }
}

pub(crate) fn handle_edit_file(desktop: &mut dyn View, state: &mut AppState, arg: &str) {
    let path = state.root_dir.join(arg);
    let path_str = path.to_string_lossy().to_string();
    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {}
        OpenResult::Opened => {
            let defaults = &state.settings.editor_defaults;
            let mut editor =
                EditorView::open(&path, defaults).unwrap_or_else(|_| EditorView::new_file(&path, defaults));
            editor.set_root_dir(state.root_dir.clone());
            let title = path
                .strip_prefix(&state.root_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if let Some(d) = downcast_desktop(desktop) {
                d.insert_tab(SlotId::Center, title, Box::new(editor));
            }
        }
    }
}

pub(crate) fn handle_shell_output(ctx: &mut CommandContext) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(output) = boxed.downcast_ref::<String>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = EditorView::from_text(output);
        desktop.insert_tab(SlotId::Center, "[cmd output]", Box::new(view));
    }
}

pub(crate) fn handle_show_results(ctx: &mut CommandContext) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((title, entries)) = boxed.downcast_ref::<(String, Vec<crate::views::results::ResultEntry>)>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = crate::views::results::ResultsView::new(title, entries.clone());
        desktop.insert_tab(SlotId::Right, title, Box::new(view));
        desktop.focus_slot(SlotId::Right);
    }
}
