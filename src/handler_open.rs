//! File-open command handlers — CM_OPEN_FILE, :edit.

use std::path::Path;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::OpenResult;
use crate::commands::*;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

/// Compute a short tab title: relative path within project, or filename for external files.
fn tab_title(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        })
}
use crate::structured::json_doc::JsonDoc;
use crate::structured::jsonl_doc::JsonlDoc;
use crate::views::csv_view::CsvView;
use crate::views::editor::EditorView;
use crate::views::struct_view::StructuredView;

pub(crate) fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState, focus_center: bool) {
    let Some(boxed) = ctx.data.as_ref() else {
        log::warn!("CM_OPEN_FILE with no data");
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::OpenFileRequest>() else {
        log::warn!("CM_OPEN_FILE data is not OpenFileRequest");
        return;
    };
    let path = &req.path;
    let title = tab_title(path, &state.root_dir);
    log::info!("Open file: {title} (broker check)");

    match state.broker.open(&title, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {
            log::info!("Already open: {title}");
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if !desktop.focus_tab_by_title(SlotId::Center, &title) {
                    // Tab was evicted but broker wasn't updated — reopen
                    state.broker.close(&title);
                    let view: Box<dyn View> =
                        try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
                    crate::handler_evict::try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title.clone(), view);
                    if focus_center {
                        desktop.focus_slot(SlotId::Center);
                    }
                    ctx.sink.push_command(
                        txv_widgets::CM_STATUS_MESSAGE,
                        Some(Box::new(Message::info("editor", format!("Opened: {title}")))),
                    );
                    return;
                }
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
                ctx.sink.push_command(CM_DIFF, Some(Box::new(String::new())));
            }
        }
        OpenResult::Opened => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.close_tab_by_title(SlotId::Center, "Welcome");
                let title = tab_title(path, &state.root_dir);
                let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
                crate::handler_evict::try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title.clone(), view);
                if focus_center {
                    desktop.focus_slot(SlotId::Center);
                }
                ctx.sink.push_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::info("editor", format!("Opened: {title}")))),
                );
            }
        }
    }
}

pub(crate) fn handle_edit_file(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState, arg: &str) {
    let path = state.root_dir.join(arg);
    let title = tab_title(&path, &state.root_dir);
    match state.broker.open(&title, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {}
        OpenResult::Opened => {
            let view: Box<dyn View> = try_open_structured(&path).unwrap_or_else(|| {
                let syntax_theme = state.current_syntax_theme().to_string();
                let defaults = state.settings.editor_defaults.clone();
                let mut editor = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
                    .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
                editor.set_root_dir(state.root_dir.clone());
                let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                let buf_id = state.buffers.register(Some(canon));
                editor.buffer_id = Some(buf_id);
                Box::new(editor)
            });
            if let Some(d) = downcast_desktop(desktop) {
                crate::handler_evict::try_insert_tab(d, state, sink, SlotId::Center, title, view);
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
            ctx.sink,
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
        let view = crate::views::results::ResultsView::new(title, entries.clone()).with_root(&state.root_dir);
        crate::handler_evict::try_insert_tab(desktop, state, ctx.sink, SlotId::Right, title.clone(), Box::new(view));
        desktop.focus_slot(SlotId::Right);
    }
}

/// Toggle the active center tab between structured and text view.
/// `to_structured`: true = switch to structured, false = switch to text.
pub(crate) fn toggle_view_mode(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState, to_structured: bool) {
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
    state.broker.close(&title);
    let _ = state.broker.open(&title, SlotId::Center, 0);
    let view: Box<dyn View> = if to_structured {
        try_open_structured(&path).unwrap_or_else(|| {
            let syntax_theme = state.current_syntax_theme().to_string();
            let defaults = state.settings.editor_defaults.clone();
            let mut ed = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
                .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
            ed.set_root_dir(state.root_dir.clone());
            let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
            ed.buffer_id = Some(state.buffers.register(Some(canon)));
            Box::new(ed)
        })
    } else {
        let syntax_theme = state.current_syntax_theme().to_string();
        let defaults = state.settings.editor_defaults.clone();
        let mut ed = EditorView::open_with_theme(&path, &defaults, &syntax_theme)
            .unwrap_or_else(|_| EditorView::new_file(&path, &defaults));
        ed.set_root_dir(state.root_dir.clone());
        let canon = path.canonicalize().unwrap_or_else(|_| path.clone());
        ed.buffer_id = Some(state.buffers.register(Some(canon)));
        Box::new(ed)
    };
    crate::handler_evict::try_insert_tab(d, state, sink, SlotId::Center, title, view);
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
fn open_editor(path: &Path, state: &mut AppState, req: &OpenFileRequest) -> Box<dyn View> {
    let syntax_theme = state.current_syntax_theme().to_string();
    let defaults = state.settings.editor_defaults.clone();
    let mut editor = EditorView::open_with_theme(path, &defaults, &syntax_theme)
        .unwrap_or_else(|_| EditorView::new_file(path, &defaults));
    editor.set_root_dir(state.root_dir.clone());
    if let (Some(line), Some(col)) = (req.line, req.col) {
        editor.goto(line, col);
    }
    if req.diff {
        editor.toggle_diff("");
    }
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let buf_id = state.buffers.register(Some(canon));
    editor.buffer_id = Some(buf_id);
    Box::new(editor)
}

/// Open the current file as a CSV table view.
pub(crate) fn open_as_csv(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState) {
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
    state.broker.close(&title);
    let _ = state.broker.open(&title, SlotId::Center, 0);
    let view: Box<dyn View> = Box::new(CsvView::new(&path, &content));
    crate::handler_evict::try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
