//! Background task drain — polls grep and build tasks for results.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::buffer_store::CommandStore;
use crate::build::ErrorLocation;
use crate::commands::CM_TODO_NOTE_SAVE;
use crate::completer::refresh_commands;
use crate::desktop::{find_view_mut, focus_view_mut, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::views::results::ResultsView;
use crate::views::todo_tree::model::{get_item_mut, load_todo_file, save_todo_file};
use crate::views::todo_tree::TodoTreeView;

/// Drain grep results from background thread into the ResultsView.
pub fn drain_grep(ctx: &mut CommandContext, state: &mut AppState) {
    let Some((title, gs, root)) = state.grep_pending.take() else {
        return;
    };
    if let Some(err) = gs.take_error() {
        let msg = Message::new(MsgLevel::Error, "grep", err);
        ctx.sink
            .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = gs.take_entries();
    let done = gs.is_done();
    if !entries.is_empty() || done {
        append_to_active_results(ctx, entries, done);
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
        let msg = Message::new(MsgLevel::Error, "build", err);
        ctx.sink
            .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
    let entries = task.take_entries();
    let done = task.is_done();
    if !entries.is_empty() || done {
        collect_build_errors(&entries, &root, state);
        append_to_results_view(ctx, entries, done);
    }
    if !done {
        state.build_pending = Some((title, task, root));
    }
}

fn collect_build_errors(entries: &[crate::views::results::ResultEntry], root: &std::path::Path, state: &mut AppState) {
    for e in entries {
        if !e.path.as_os_str().is_empty() {
            state.build_errors.push(ErrorLocation {
                file: e
                    .path
                    .strip_prefix(root)
                    .unwrap_or(&e.path)
                    .to_string_lossy()
                    .to_string(),
                line: e.line + 1,
                col: e.col + 1,
                message: e.text.clone(),
            });
        }
    }
}

fn append_to_active_results(ctx: &mut CommandContext, entries: Vec<crate::views::results::ResultEntry>, done: bool) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let rv = desktop
        .panel_mut(SlotId::Tools as usize)
        .and_then(|p| p.active_view_mut())
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<ResultsView>());
    if let Some(rv) = rv {
        rv.append(entries, done);
    }
}

fn append_to_results_view(ctx: &mut CommandContext, entries: Vec<crate::views::results::ResultEntry>, done: bool) {
    append_to_active_results(ctx, entries, done);
}

/// Refresh plugins: scan dirs, reload changed, unload removed.
pub fn refresh_plugins(ctx: &mut CommandContext, state: &mut AppState) {
    let warnings = state.plugins.refresh(&mut state.script);
    if !warnings.is_empty() {
        refresh_commands(&state.command_list, &state.script);
        for w in warnings {
            let msg = Message::warn("plugin", w);
            ctx.sink
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
    if let Some(nv) = find_view_mut::<NotesView>(desktop, SlotId::Center) {
        // Tab exists — update content and bring to front
        nv.replace_content(note);
        focus_view_mut::<NotesView>(desktop, SlotId::Center);
        if *focus {
            desktop.focus_panel(SlotId::Center as usize);
        } else {
            desktop.focus_panel(SlotId::Left as usize);
        }
    } else {
        // Create new Notes tab
        let mut nv = NotesView::new(note);
        nv.editor
            .editor_mut()
            .set_shared_register(state.shared_register.clone());
        let store = CommandStore::new(CM_TODO_NOTE_SAVE, ctx.sink.clone());
        nv.set_store(Box::new(store));
        let view: Box<dyn View> = Box::new(nv);
        try_insert_tab(desktop, state, ctx.sink, SlotId::Center, "Notes".to_string(), view);
        focus_view_mut::<NotesView>(desktop, SlotId::Center);
        if *focus {
            desktop.focus_panel(SlotId::Center as usize);
        } else {
            desktop.focus_panel(SlotId::Left as usize);
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
    if let Some(nv) = find_view_mut::<NotesView>(desktop, SlotId::Center) {
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
    let mut file = load_todo_file(&todo_path);
    if let Some(item) = get_item_mut(&mut file, path) {
        item.note.clone_from(content);
        save_todo_file(&todo_path, &file);
    }
}

/// Handle CM_TODO_ACTION — dispatch MCP todo actions.
pub fn handle_todo_action(ctx: &mut CommandContext, _state: &mut AppState) {
    use crate::mcp::commands::McpAction;
    let Some(action) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<McpAction>()) else {
        return;
    };
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let todo_view = desktop
        .panel_mut(SlotId::Left as usize)
        .and_then(|p| p.view_at_mut(2))
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<TodoTreeView>());
    if let Some(tv) = todo_view {
        if let Err(e) = tv.mcp_action(action) {
            let msg = Message::new(MsgLevel::Error, "todo", e);
            ctx.sink
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}
