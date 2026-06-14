//! Shared command handler — used by both main.rs and test harness.
//! App handlers call each other directly (queue is for cross-view communication).

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::commands::{CM_TW_SPLIT_H, CM_TW_SPLIT_V, CM_TW_TAB_CLOSE};
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use crate::app_state::AppState;
use crate::commands::*;
use crate::handler_badges::{
    auto_close_exited_terminals, sync_dirty_badges, sync_pty_badges, sync_root_badges, sync_tab_titles,
};
use crate::handler_build::{
    handle_build, handle_next_error, handle_prev_error, handle_run, handle_test, handle_test_at_cursor,
    handle_test_file,
};
use crate::handler_clipboard::{handle_clipboard_commands, update_problems_view};
use crate::handler_close::{handle_app_quit, handle_file_closed, handle_save_all, handle_tab_close};
use crate::handler_confirm::{handle_confirm_response, handle_set_confirm_context};
use crate::handler_context::{broadcast_context, handle_cursor_moved, update_window_title};
use crate::handler_diff_view::{handle_diff_exit, handle_diff_open_view, handle_diff_revert};
use crate::handler_drain::{
    drain_build, drain_grep, handle_todo_action, open_todo_note, refresh_plugins, save_todo_note, update_todo_note,
};
use crate::handler_exec::handle_execute_command;
use crate::handler_exec_nav::handle_file_finder_open;
use crate::handler_git::{
    handle_git_commit, handle_git_commit_prompt, handle_git_stage, handle_git_unstage, handle_git_untrack,
};
use crate::handler_log::open_git_log;
use crate::handler_mcp::drain_mcp;
use crate::handler_open::{handle_open_file, handle_shell_output, handle_show_results};
use crate::handler_script::handle_script_command;
use crate::handler_set::handle_set_global;
use crate::handler_split::{
    handle_split, handle_split_close, handle_split_focus, handle_split_h, handle_split_linked, handle_split_v,
};
use crate::handler_split_nav::{handle_diff_split, handle_open_in_split};
use crate::handler_tabs;
use crate::handler_theme::{handle_set_glyphs, handle_set_syntax_theme, handle_toggle_theme};
use crate::lsp::handler::{handle_lsp_command, poll_lsp};
use crate::mcp::collect::{collect_messages, collect_snapshot, collect_terminal_content};
use crate::suspend::{peek_screen, suspend_to_shell};

/// Handle a command from the Program event loop.
/// This is the single source of truth for command handling.
pub fn handle_command(ctx: &mut CommandContext, state: &mut AppState) {
    if intercept_status_message(ctx, state) {
        return;
    }
    run_background_tasks(ctx, state);
    update_mcp_snapshot(ctx, state);
    dispatch_command(ctx, state);
}

fn intercept_status_message(ctx: &mut CommandContext, state: &mut AppState) -> bool {
    if ctx.command() != txv_widgets::CM_STATUS_MESSAGE {
        return false;
    }
    if let Some(boxed) = ctx.data().as_ref() {
        if let Some(msg) = boxed.downcast_ref::<Message>() {
            if let Ok(mut ring) = state.messages.lock() {
                ring.push(msg.clone());
            } else {
                log::error!("Message ring mutex poisoned");
            }
        }
    }
    true
}

fn run_background_tasks(ctx: &mut CommandContext, state: &mut AppState) {
    handle_lsp_command(ctx, state);
    {
        let sink = ctx.sink().clone();
        poll_lsp(state, &sink);
    }
    drain_grep(ctx, state);
    drain_build(ctx, state);
    drain_mcp(ctx, state);
    auto_close_exited_terminals(ctx, state);
    sync_dirty_badges(ctx);
    sync_pty_badges(ctx, state);
    sync_tab_titles(ctx, state);
    sync_root_badges(ctx, state);
}

fn update_mcp_snapshot(ctx: &mut CommandContext, state: &mut AppState) {
    state.mcp.tick = state.mcp.tick.wrapping_add(1);
    if state.mcp.snapshot.is_some() && state.mcp.tick.is_multiple_of(20) {
        if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
            let mut snap = collect_snapshot(desktop);
            snap.terminals = collect_terminal_content(desktop);
            snap.messages = collect_messages(&state.messages);
            let Some(ref arc) = state.mcp.snapshot else {
                return;
            };
            match arc.lock() {
                Ok(mut locked) => *locked = snap,
                Err(_) => log::error!("MCP snapshot mutex poisoned"),
            }
        }
    }
    if state.mcp.tick.is_multiple_of(100) {
        refresh_plugins(ctx, state);
    }
}

fn dispatch_command(ctx: &mut CommandContext, state: &mut AppState) {
    if !dispatch_core(ctx, state) {
        dispatch_extended_cmd(ctx, state);
    }
}

fn dispatch_core(ctx: &mut CommandContext, state: &mut AppState) -> bool {
    match ctx.command() {
        CM_TICK => handle_tick(ctx, state),
        CM_APP_QUIT => handle_app_quit(ctx, state),
        CM_TW_TAB_CLOSE | CM_TAB_CLOSE => handle_tab_close(ctx, state),
        CM_SAVE_ALL => handle_save_all(ctx),
        CM_OPEN_FILE => handle_open_file(ctx, state, false),
        CM_OPEN_FILE_FOCUS => handle_open_file(ctx, state, true),
        CM_EXECUTE_COMMAND => handle_execute_command(ctx, state),
        CM_FILE_FINDER_OPEN => handle_file_finder_open(ctx, state),
        CM_SHOW_HELP => handle_show_help(ctx, state),
        CM_SHOW_MESSAGES => handle_show_messages(ctx, state),
        CM_NEW_SHELL => handle_new_shell(ctx, state),
        CM_FILE_CLOSED => handle_file_closed(ctx, state),
        CM_SHELL_OUTPUT => handle_shell_output(ctx, state),
        CM_SHOW_RESULTS => handle_show_results(ctx, state),
        CM_BUILD => handle_build(ctx, state),
        CM_RUN => handle_run(ctx, state),
        CM_TEST => handle_test(ctx, state),
        CM_TEST_FILE => handle_test_file(ctx, state),
        CM_TEST_AT_CURSOR => handle_test_at_cursor(ctx, state),
        CM_NEXT_ERROR => handle_next_error(ctx, state),
        CM_PREV_ERROR => handle_prev_error(ctx, state),
        CM_SET_GLOBAL => handle_set_global(ctx, state),
        CM_SUSPEND => handle_suspend(ctx),
        CM_PEEK => peek_screen(),
        CM_GIT_STAGE => handle_git_stage(ctx, state),
        CM_GIT_UNSTAGE => handle_git_unstage(ctx, state),
        CM_GIT_UNTRACK => handle_git_untrack(ctx, state),
        CM_GIT_COMMIT => handle_git_commit(ctx, state),
        CM_GIT_COMMIT_PROMPT => handle_git_commit_prompt(ctx, state),
        CM_GIT_LOG => open_git_log(ctx, state, ""),
        _ => return false,
    }
    true
}

fn handle_tick(ctx: &mut CommandContext, state: &mut AppState) {
    if state.show_messages_on_start {
        state.show_messages_on_start = false;
        handle_show_messages(ctx, state);
    }
    broadcast_context(ctx, state);
    update_window_title(state, ctx.sink());
}

fn handle_suspend(ctx: &mut CommandContext) {
    if let Err(e) = suspend_to_shell() {
        ctx.sink().push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::error("suspend", e))),
        );
    }
}

fn dispatch_extended_cmd(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command() {
        CM_SPLIT => handle_split(ctx, state),
        CM_SPLIT_CLOSE => handle_split_close(ctx, state),
        CM_OPEN_IN_SPLIT => handle_open_in_split(ctx, state),
        CM_SPLIT_FOCUS => handle_split_focus(ctx),
        CM_SPLIT_LINKED => handle_split_linked(ctx, state),
        CM_DIFF_SPLIT => handle_diff_split(ctx, state),
        CM_DIFF_OPEN_VIEW => handle_diff_open_view(ctx, state),
        CM_DIFF_EXIT => handle_diff_exit(ctx),
        CM_DIFF_REVERT => handle_diff_revert(ctx),
        CM_TW_SPLIT_H => handle_split_h(ctx, state),
        CM_TW_SPLIT_V => handle_split_v(ctx, state),
        CM_TOGGLE_THEME => handle_toggle_theme(ctx, state),
        CM_SET_SYNTAX_THEME => handle_set_syntax_theme(ctx, state),
        CM_SET_GLYPHS => handle_set_glyphs(ctx, state),
        CM_CURSOR_MOVED => handle_cursor_moved(ctx, state),
        CM_SET_CONFIRM_CONTEXT => handle_set_confirm_context(ctx, state),
        CM_CONFIRM_RESPONSE => handle_confirm_response(ctx, state),
        CM_EDITOR_REPLACE_SELECTION
        | CM_EDITOR_DELETE_LINE
        | CM_EDITOR_REPLACE_WORD
        | CM_EDITOR_SEARCH
        | CM_EDITOR_CLEAR_HIGHLIGHT
        | CM_EDITOR_SET
        | CM_CHAR_INSERTED
        | CM_WORD_COMPLETED => handle_script_command(ctx, state),
        CM_TODO_NOTE_SAVE => save_todo_note(ctx, state),
        CM_TODO_NOTE_OPEN => open_todo_note(ctx, state),
        CM_TODO_NOTE_UPDATE => update_todo_note(ctx, state),
        CM_TODO_ACTION => handle_todo_action(ctx, state),
        CM_DIAGNOSTIC => update_problems_view(ctx),
        _ => handle_clipboard_commands(ctx, state),
    }
}

fn handle_show_help(ctx: &mut CommandContext, state: &mut AppState) {
    handler_tabs::handle_show_help(ctx, state);
}

fn handle_show_messages(ctx: &mut CommandContext, state: &mut AppState) {
    handler_tabs::handle_show_messages(ctx, state);
}

fn handle_new_shell(ctx: &mut CommandContext, state: &mut AppState) {
    handler_tabs::handle_new_shell(ctx, state);
}

/// Downcast the desktop View to TiledWorkspace.
pub fn downcast_workspace(view: &mut dyn View) -> Option<&mut TiledWorkspace> {
    let any = view.as_any_mut()?;
    any.downcast_mut::<TiledWorkspace>()
}

/// Deprecated alias.
pub fn downcast_desktop(view: &mut dyn View) -> Option<&mut TiledWorkspace> {
    downcast_workspace(view)
}
