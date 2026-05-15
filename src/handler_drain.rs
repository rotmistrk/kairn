//! Background task drain — polls grep and build tasks for results.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

/// Drain grep results from background thread into the ResultsView.
pub fn drain_grep(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, gs, root)) = state.grep_pending.take() else {
        return;
    };
    if let Some(err) = gs.take_error() {
        let msg = txv_core::message::Message::new(txv_core::message::MsgLevel::Error, "grep", err);
        ctx.queue
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = gs.take_entries();
    let done = gs.is_done();
    if !entries.is_empty() || done {
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            if let Some(view) = desktop.active_view_mut(SlotId::Right) {
                if let Some(rv) = view
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<crate::views::results::ResultsView>())
                {
                    rv.append(entries, done);
                }
            }
        }
    }
    if !done {
        state.grep_pending = Some((title, gs, root));
    }
}

/// Drain build/test results from background thread into the ResultsView.
pub fn drain_build(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, task, root)) = state.build_pending.take() else {
        return;
    };
    if let Some(err) = task.take_error() {
        let msg = txv_core::message::Message::new(txv_core::message::MsgLevel::Error, "build", err);
        ctx.queue
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = task.take_entries();
    let done = task.is_done();
    if !entries.is_empty() || done {
        for e in &entries {
            if !e.path.as_os_str().is_empty() {
                state.build_errors.push(crate::build::ErrorLocation {
                    file: e
                        .path
                        .strip_prefix(&root)
                        .unwrap_or(&e.path)
                        .to_string_lossy()
                        .to_string(),
                    line: e.line + 1,
                    col: e.col + 1,
                    message: e.text.clone(),
                });
            }
        }
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            if let Some(view) = desktop.active_view_mut(SlotId::Right) {
                if let Some(rv) = view
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<crate::views::results::ResultsView>())
                {
                    rv.append(entries, done);
                }
            }
        }
    }
    if !done {
        state.build_pending = Some((title, task, root));
    }
}

/// Refresh plugins: scan dirs, reload changed, unload removed.
pub fn refresh_plugins(ctx: &mut CommandContext, state: &mut AppState) {
    let warnings = state.plugins.refresh(&mut state.script);
    if !warnings.is_empty() {
        crate::completer::refresh_commands(&state.command_list, &state.script);
        for w in warnings {
            let msg = txv_core::message::Message::warn("plugin", w);
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

/// Update PTY activity badges and auto-close exited terminals.
pub fn update_pty_badges(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let idle_secs = state.settings.terminal_idle_timeout;
        let exited = desktop.update_badges(idle_secs);
        if state.settings.terminal_auto_close {
            for title in exited {
                for slot in [SlotId::Right, SlotId::Bottom] {
                    desktop.close_tab_by_title(slot, &title);
                }
            }
        }
    }
}

/// Open (or focus) the Notes tab for a todo item.
pub fn open_todo_note(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((tree_path, note)) = boxed.downcast_ref::<(Vec<usize>, String)>() else {
        return;
    };
    let kairn_dir = state.root_dir.join(".kairn");
    let _ = std::fs::create_dir_all(&kairn_dir);
    let note_file = kairn_dir.join("note.md");
    if let Err(e) = std::fs::write(&note_file, note) {
        log::warn!("todo note: failed to write note.md: {e}");
        return;
    }
    state.todo_note_path = Some(tree_path.clone());
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let title = "Notes";
    if desktop.focus_tab_by_title(SlotId::Center, title) {
        // Tab exists — reload content by replacing the editor
        desktop.close_tab_by_title(SlotId::Center, title);
    }
    let defaults = &state.settings.editor_defaults;
    let theme = state.current_syntax_theme();
    let view: Box<dyn View> = match crate::views::editor::EditorView::open_with_theme(&note_file, defaults, theme) {
        Ok(editor) => Box::new(editor),
        Err(_) => Box::new(crate::views::editor::EditorView::new_file(&note_file, defaults)),
    };
    crate::handler_evict::try_insert_tab(desktop, state, ctx.queue, SlotId::Center, title.to_string(), view);
    desktop.focus_tab_by_title(SlotId::Center, title);
    desktop.focus_slot(SlotId::Center);
}

/// Sync the Notes tab content back to the todo item on save.
pub fn sync_todo_note(state: &mut AppState) {
    let Some(ref path) = state.todo_note_path else {
        return;
    };
    let note_file = state.root_dir.join(".kairn").join("note.md");
    let Ok(content) = std::fs::read_to_string(&note_file) else {
        return;
    };
    let todo_path = state.root_dir.join(".kairn.todo");
    let mut file = crate::views::todo_tree::model::load_todo_file(&todo_path);
    if let Some(item) = crate::views::todo_tree::model::get_item_mut(&mut file, path) {
        item.note = content;
        crate::views::todo_tree::model::save_todo_file(&todo_path, &file);
    }
}
