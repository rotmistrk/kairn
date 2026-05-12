//! File-open command handlers — CM_OPEN_FILE, :edit.

use std::path::Path;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::OpenResult;
use crate::commands::*;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;
use crate::settings::EditorSettings;
use crate::structured::json_doc::JsonDoc;
use crate::structured::jsonl_doc::JsonlDoc;
use crate::views::editor::EditorView;
use crate::views::struct_view::StructuredView;

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
                let title = path
                    .strip_prefix(&state.root_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                let view: Box<dyn View> = try_open_structured(path)
                    .unwrap_or_else(|| open_editor(path, &state.root_dir, &state.settings.editor_defaults, req));
                crate::handler_evict::try_insert_tab(desktop, state, SlotId::Center, title.clone(), view);
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
                crate::handler_evict::try_insert_tab(d, state, SlotId::Center, title, Box::new(editor));
            }
        }
    }
}

pub(crate) fn handle_shell_output(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(output) = boxed.downcast_ref::<String>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = EditorView::from_text(output);
        crate::handler_evict::try_insert_tab(desktop, state, SlotId::Center, "[cmd output]".into(), Box::new(view));
    }
}

pub(crate) fn handle_show_results(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((title, entries)) = boxed.downcast_ref::<(String, Vec<crate::views::results::ResultEntry>)>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = crate::views::results::ResultsView::new(title, entries.clone());
        crate::handler_evict::try_insert_tab(desktop, state, SlotId::Right, title.clone(), Box::new(view));
        desktop.focus_slot(SlotId::Right);
    }
}

/// Try to open a file as a structured view (JSON/JSONC/JSONL). Returns None if not applicable or parse fails.
fn try_open_structured(path: &Path) -> Option<Box<dyn View>> {
    let ext = path.extension()?.to_str()?;
    let content = std::fs::read_to_string(path).ok()?;
    match ext {
        "json" => {
            let doc = JsonDoc::parse(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        "jsonc" => {
            let doc = JsonDoc::parse_jsonc(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        "jsonl" | "ndjson" => {
            let doc = JsonlDoc::parse(&content).ok()?;
            Some(Box::new(StructuredView::new(path, Box::new(doc))))
        }
        _ => None,
    }
}

/// Open a file as an EditorView (fallback).
fn open_editor(path: &Path, root_dir: &Path, defaults: &EditorSettings, req: &OpenFileRequest) -> Box<dyn View> {
    let mut editor = EditorView::open(path, defaults).unwrap_or_else(|_| EditorView::new_file(path, defaults));
    editor.set_root_dir(root_dir.to_path_buf());
    if let (Some(line), Some(col)) = (req.line, req.col) {
        editor.goto(line, col);
    }
    if req.diff {
        editor.toggle_diff("");
    }
    Box::new(editor)
}
