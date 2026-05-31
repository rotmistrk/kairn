//! File-open command handlers — CM_OPEN_FILE, :edit.

use std::fs::File;
use std::path::Path;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::OpenResult;
use crate::commands::*;
use crate::desktop::{close_tab_by_title, focus_editor_by_path, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::views::results::{ResultEntry, ResultsView};

/// Initial tab label — just the filename. `sync_tab_titles` will disambiguate.
fn initial_title(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string()
}
pub(crate) use crate::handler_open_view::open_as_csv;
use crate::handler_open_view::{open_editor, open_editor_view, try_open_structured};
use crate::views::editor::EditorView;

pub(crate) fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState, focus_center: bool) {
    let Some(boxed) = ctx.data.as_ref() else {
        log::warn!("CM_OPEN_FILE with no data");
        return;
    };
    let Some(req) = boxed.downcast_ref::<OpenFileRequest>() else {
        log::warn!("CM_OPEN_FILE data is not OpenFileRequest");
        return;
    };
    let path = &req.path;
    let abs_key = path.to_string_lossy().to_string();
    log::info!("Open file: {abs_key} (broker check)");

    match state.broker.open(&abs_key, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => handle_already_open(ctx, state, req, &abs_key, focus_center),
        OpenResult::Opened => handle_fresh_open(ctx, state, req, &abs_key, focus_center),
    }
}

fn handle_already_open(
    ctx: &mut CommandContext,
    state: &mut AppState,
    req: &OpenFileRequest,
    abs_key: &str,
    focus_center: bool,
) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        if !focus_editor_by_path(desktop, abs_key) {
            state.broker.close(abs_key);
            let path = &req.path;
            let title = initial_title(path);
            let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
            try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title, view);
            if focus_center {
                desktop.focus_panel(SlotId::Center as usize);
            }
            ctx.sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::info("editor", format!("Opened: {abs_key}")))),
            );
            return;
        }
        if let Some(line) = req.line {
            let col = req.col.unwrap_or(0);
            let ed = desktop
                .panel_mut(SlotId::Center as usize)
                .and_then(|p| p.active_view_mut())
                .and_then(|v| v.as_any_mut())
                .and_then(|a| a.downcast_mut::<EditorView>());
            if let Some(ed) = ed {
                ed.goto(line, col);
            }
        }
        if focus_center {
            desktop.focus_panel(SlotId::Center as usize);
        }
    }
    if req.diff {
        ctx.sink.push_command(CM_DIFF, Some(Box::new(String::new())));
    }
}

fn handle_fresh_open(
    ctx: &mut CommandContext,
    state: &mut AppState,
    req: &OpenFileRequest,
    _abs_key: &str,
    focus_center: bool,
) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        close_tab_by_title(desktop, SlotId::Center, "Welcome");
        let path = &req.path;
        let title = initial_title(path);
        let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
        try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title.clone(), view);
        state.tab_titles_dirty = true;
        if focus_center {
            desktop.focus_panel(SlotId::Center as usize);
        }
        ctx.sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("editor", format!("Opened: {title}")))),
        );
    }
}

pub(crate) fn handle_edit_file(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState, arg: &str) {
    let path = state.root_dir.join(arg);
    if path.is_dir() {
        let msg = Message::warn("edit", format!("Cannot open directory: {arg}"));
        sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        return;
    }
    if path.exists() && File::open(&path).is_err() {
        let msg = Message::warn("edit", format!("Cannot read file: {arg}"));
        sink.push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        return;
    }
    let abs_key = path.to_string_lossy().to_string();
    let title = initial_title(&path);
    match state.broker.open(&abs_key, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {
            if let Some(d) = downcast_desktop(desktop) {
                focus_editor_by_path(d, &abs_key);
            }
        }
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
                try_insert_tab(d, state, sink, SlotId::Center, title, view);
                state.tab_titles_dirty = true;
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
        try_insert_tab(
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
    let Some((title, entries)) = boxed.downcast_ref::<(String, Vec<ResultEntry>)>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = ResultsView::new(title, entries.clone()).with_root(&state.root_dir);
        try_insert_tab(desktop, state, ctx.sink, SlotId::Tools, title.clone(), Box::new(view));
        desktop.focus_panel(SlotId::Tools as usize);
    }
}

/// Toggle the active center tab between structured and text view.
/// `to_structured`: true = switch to structured, false = switch to text.
pub(crate) fn toggle_view_mode(desktop: &mut dyn View, sink: &EventSink, state: &mut AppState, to_structured: bool) {
    let Some(d) = downcast_desktop(desktop) else {
        return;
    };
    let abs_path = d
        .panel_mut(SlotId::Center as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_ref::<EditorView>())
        .map(|ev| ev.path().to_path_buf());
    let Some(path) = abs_path else {
        return;
    };
    if !path.is_file() {
        return;
    }
    let abs_key = path.to_string_lossy().to_string();
    let title = initial_title(&path);
    let panel = d.panel_mut(SlotId::Center as usize);
    if let Some(p) = panel {
        p.close_active();
    }
    state.broker.close(&abs_key);
    let _ = state.broker.open(&abs_key, SlotId::Center, 0);
    let view: Box<dyn View> = if to_structured {
        try_open_structured(&path).unwrap_or_else(|| open_editor_view(&path, state))
    } else {
        open_editor_view(&path, state)
    };
    try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
