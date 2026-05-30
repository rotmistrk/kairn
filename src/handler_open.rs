//! File-open command handlers — CM_OPEN_FILE, :edit.

use std::fs::File;
use std::path::Path;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::OpenResult;
use crate::commands::*;
use crate::desktop::{active_tab_title, close_tab_by_title, focus_tab_by_title, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::views::results::{ResultEntry, ResultsView};

/// Compute tab title: relative path within project, or full path for external files.
fn tab_title(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string()
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
    let title = tab_title(path, &state.root_dir);
    log::info!("Open file: {title} (broker check)");

    match state.broker.open(&title, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => handle_already_open(ctx, state, req, &title, focus_center),
        OpenResult::Opened => handle_fresh_open(ctx, state, req, &title, focus_center),
    }
}

fn handle_already_open(
    ctx: &mut CommandContext,
    state: &mut AppState,
    req: &OpenFileRequest,
    title: &str,
    focus_center: bool,
) {
    log::info!("Already open: {title}");
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        if !focus_tab_by_title(desktop, SlotId::Center, title) {
            state.broker.close(title);
            let path = &req.path;
            let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
            try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title.to_string(), view);
            if focus_center {
                desktop.focus_panel(SlotId::Center as usize);
            }
            ctx.sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::info("editor", format!("Opened: {title}")))),
            );
            return;
        }
        if let (Some(line), Some(col)) = (req.line, req.col) {
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
    title: &str,
    focus_center: bool,
) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        close_tab_by_title(desktop, SlotId::Center, "Welcome");
        let path = &req.path;
        let view: Box<dyn View> = try_open_structured(path).unwrap_or_else(|| open_editor(path, state, req));
        try_insert_tab(desktop, state, ctx.sink, SlotId::Center, title.to_string(), view);
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
    let title = tab_title(&path, &state.root_dir);
    match state.broker.open(&title, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { slot, .. } => {
            if let Some(d) = downcast_desktop(desktop) {
                focus_tab_by_title(d, slot, &title);
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
    let Some(title) = active_tab_title(d, SlotId::Center).map(String::from) else {
        return;
    };
    let path = state.root_dir.join(&title);
    if !path.is_file() {
        return;
    }
    close_tab_by_title(d, SlotId::Center, &title);
    state.broker.close(&title);
    let _ = state.broker.open(&title, SlotId::Center, 0);
    let view: Box<dyn View> = if to_structured {
        try_open_structured(&path).unwrap_or_else(|| open_editor_view(&path, state))
    } else {
        open_editor_view(&path, state)
    };
    try_insert_tab(d, state, sink, SlotId::Center, title, view);
}
