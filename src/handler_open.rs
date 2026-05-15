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
use crate::views::csv_view::CsvView;
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
                let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| {
                    open_editor(
                        path,
                        &state.root_dir,
                        &state.settings.editor_defaults,
                        req,
                        state.current_syntax_theme(),
                    )
                });
                crate::handler_evict::try_insert_tab(desktop, state, ctx.queue, SlotId::Center, title.clone(), view);
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

pub(crate) fn handle_edit_file(desktop: &mut dyn View, queue: &mut EventQueue, state: &mut AppState, arg: &str) {
    let path = state.root_dir.join(arg);
    let path_str = path.to_string_lossy().to_string();
    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {}
        OpenResult::Opened => {
            let title = path
                .strip_prefix(&state.root_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let view: Box<dyn View> = try_open_structured(&path).unwrap_or_else(|| {
                let defaults = &state.settings.editor_defaults;
                let syntax_theme = state.current_syntax_theme();
                let mut editor = EditorView::open_with_theme(&path, defaults, syntax_theme)
                    .unwrap_or_else(|_| EditorView::new_file(&path, defaults));
                editor.set_root_dir(state.root_dir.clone());
                Box::new(editor)
            });
            if let Some(d) = downcast_desktop(desktop) {
                crate::handler_evict::try_insert_tab(d, state, queue, SlotId::Center, title, view);
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
        crate::handler_evict::try_insert_tab(
            desktop,
            state,
            ctx.queue,
            SlotId::Center,
            "[cmd output]".into(),
            Box::new(view),
        );
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
        crate::handler_evict::try_insert_tab(desktop, state, ctx.queue, SlotId::Right, title.clone(), Box::new(view));
        desktop.focus_slot(SlotId::Right);
    }
}

/// Toggle the active center tab between structured and text view.
/// `to_structured`: true = switch to structured, false = switch to text.
pub(crate) fn toggle_view_mode(
    desktop: &mut dyn View,
    queue: &mut EventQueue,
    state: &mut AppState,
    to_structured: bool,
) {
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let Some(title) = d.active_tab_title(SlotId::Center).map(String::from) else {
        return;
    };
    let path = state.root_dir.join(&title);
    if !path.is_file() {
        return;
    }
    d.close_tab_by_title(SlotId::Center, &title);
    state.broker.close(&path.to_string_lossy());
    let _ = state.broker.open(&path.to_string_lossy(), SlotId::Center, 0);
    let view: Box<dyn View> = if to_structured {
        try_open_structured(&path).unwrap_or_else(|| {
            let defaults = &state.settings.editor_defaults;
            let syntax_theme = state.current_syntax_theme();
            let mut ed = EditorView::open_with_theme(&path, defaults, syntax_theme)
                .unwrap_or_else(|_| EditorView::new_file(&path, defaults));
            ed.set_root_dir(state.root_dir.clone());
            Box::new(ed)
        })
    } else {
        let defaults = &state.settings.editor_defaults;
        let syntax_theme = state.current_syntax_theme();
        let mut ed = EditorView::open_with_theme(&path, defaults, syntax_theme)
            .unwrap_or_else(|_| EditorView::new_file(&path, defaults));
        ed.set_root_dir(state.root_dir.clone());
        Box::new(ed)
    };
    crate::handler_evict::try_insert_tab(d, state, queue, SlotId::Center, title, view);
}

/// Try to open a file as a structured view (JSON/JSONC/JSONL/CSV/TSV). Returns None if not applicable or parse fails.
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
        "csv" | "tsv" | "tab" | "psv" => Some(Box::new(CsvView::new(path, &content))),
        _ => None,
    }
}

/// Open a file as an EditorView (fallback).
fn open_editor(
    path: &Path,
    root_dir: &Path,
    defaults: &EditorSettings,
    req: &OpenFileRequest,
    syntax_theme: &str,
) -> Box<dyn View> {
    let mut editor = EditorView::open_with_theme(path, defaults, syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, defaults));
    editor.set_root_dir(root_dir.to_path_buf());
    if let (Some(line), Some(col)) = (req.line, req.col) {
        editor.goto(line, col);
    }
    if req.diff {
        editor.toggle_diff("");
    }
    Box::new(editor)
}

/// Open the current file as a CSV table view.
pub(crate) fn open_as_csv(desktop: &mut dyn View, queue: &mut EventQueue, state: &mut AppState) {
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let Some(title) = d.active_tab_title(SlotId::Center).map(String::from) else {
        return;
    };
    let path = state.root_dir.join(&title);
    if !path.is_file() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    d.close_tab_by_title(SlotId::Center, &title);
    state.broker.close(&path.to_string_lossy());
    let _ = state.broker.open(&path.to_string_lossy(), SlotId::Center, 0);
    let view: Box<dyn View> = Box::new(CsvView::new(&path, &content));
    crate::handler_evict::try_insert_tab(d, state, queue, SlotId::Center, title, view);
}
