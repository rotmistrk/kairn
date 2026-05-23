//! Background task drain — polls grep and build tasks for results.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};

/// Drain grep results from background thread into the ResultsView.
pub fn drain_grep(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, gs, root)) = state.grep_pending.take() else {
        return;
    };
    if let Some(err) = gs.take_error() {
        let msg = txv_core::message::Message::new(txv_core::message::MsgLevel::Error, "grep", err);
        ctx.sink
            .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
        ctx.sink
            .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
            ctx.sink
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
    let Some((tree_path, note, focus)) = boxed.downcast_ref::<(Vec<usize>, String, bool)>() else {
        return;
    };
    state.todo_note_path = Some(tree_path.clone());
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    use crate::views::notes::NotesView;
    if let Some(nv) = desktop.find_view_mut::<NotesView>(SlotId::Center) {
        // Tab exists — update content and bring to front
        nv.replace_content(note);
        desktop.focus_view_mut::<NotesView>(SlotId::Center);
        if *focus {
            desktop.focus_slot(SlotId::Center);
        } else {
            desktop.focus_slot(SlotId::Left);
        }
    } else {
        // Create new Notes tab
        let mut nv = NotesView::new(note);
        let store = crate::buffer_store::CommandStore::new(crate::commands::CM_TODO_NOTE_SAVE, ctx.sink.clone());
        nv.set_store(Box::new(store));
        let view: Box<dyn View> = Box::new(nv);
        crate::handler_evict::try_insert_tab(desktop, state, ctx.sink, SlotId::Center, "Notes".to_string(), view);
        desktop.focus_view_mut::<NotesView>(SlotId::Center);
        if *focus {
            desktop.focus_slot(SlotId::Center);
        } else {
            desktop.focus_slot(SlotId::Left);
        }
    }
}

/// Handle CM_TODO_NOTE_UPDATE — update Notes content if tab exists, don't create.
pub fn update_todo_note(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some((tree_path, note)) = boxed.downcast_ref::<(Vec<usize>, String)>() else {
        return;
    };
    state.todo_note_path = Some(tree_path.clone());
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    use crate::views::notes::NotesView;
    if let Some(nv) = desktop.find_view_mut::<NotesView>(SlotId::Center) {
        nv.replace_content(note);
    }
}

/// Handle CM_TODO_NOTE_SAVE — content arrives via command data from CommandStore.
pub fn save_todo_note(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(ref path) = state.todo_note_path else {
        return;
    };
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(content) = boxed.downcast_ref::<String>() else {
        return;
    };
    let todo_path = state.root_dir.join(".kairn.todo");
    let mut file = crate::views::todo_tree::model::load_todo_file(&todo_path);
    if let Some(item) = crate::views::todo_tree::model::get_item_mut(&mut file, path) {
        item.note.clone_from(content);
        crate::views::todo_tree::model::save_todo_file(&todo_path, &file);
    }
}
