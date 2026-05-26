//! Shared command handler — used by both main.rs and test harness.
//! App handlers call each other directly (queue is for cross-view communication).

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::commands::{CM_TW_SPLIT_H, CM_TW_SPLIT_V, CM_TW_TAB_CLOSE};
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use crate::app_state::AppState;
use crate::commands::*;
use crate::slots::{focus_tab_by_title, insert_tab, next_tab_name, SlotId};
use crate::views::help::HelpView;
use crate::views::messages::MessagesView;
use crate::views::terminal::new_shell_terminal;
use crate::views::welcome::WelcomeView;

/// Handle a command from the Program event loop.
/// This is the single source of truth for command handling.
pub fn handle_command(ctx: &mut CommandContext, state: &mut AppState) {
    // Intercept status messages and append to ring buffer
    if ctx.command == txv_widgets::CM_STATUS_MESSAGE {
        if let Some(boxed) = ctx.data.as_ref() {
            if let Some(msg) = boxed.downcast_ref::<Message>() {
                if let Ok(mut ring) = state.messages.lock() {
                    ring.push(msg.clone());
                } else {
                    log::error!("Message ring mutex poisoned");
                }
            }
        }
        return;
    }

    // LSP: send didOpen on file open
    crate::lsp::handler::handle_lsp_command(ctx, state);
    // LSP: poll servers for notifications
    crate::lsp::handler::poll_lsp(state, ctx.sink);

    // Drain background tasks (grep, build)
    crate::handler_drain::drain_grep(ctx, state);
    crate::handler_drain::drain_build(ctx, state);

    // Drain MCP write commands
    crate::handler_mcp::drain_mcp(ctx, state);

    // Auto-close exited terminals
    crate::handler_badges::auto_close_exited_terminals(ctx, state);

    // Sync dirty badges on tab bar
    crate::handler_badges::sync_dirty_badges(ctx);

    // Sync PTY activity badges on terminal tabs
    crate::handler_badges::sync_pty_badges(ctx, state);

    // MCP: update snapshot every 20 commands (~1s at 50ms tick)
    state.mcp_tick = state.mcp_tick.wrapping_add(1);
    if state.mcp_snapshot.is_some() && state.mcp_tick.is_multiple_of(20) {
        if let Some(desktop) = downcast_desktop(ctx.desktop) {
            let mut snap = crate::mcp::collect::collect_snapshot(desktop);
            snap.terminals = crate::mcp::collect::collect_terminal_content(desktop);
            snap.messages = crate::mcp::collect::collect_messages(&state.messages);
            if let Some(ref arc) = state.mcp_snapshot {
                if let Ok(mut locked) = arc.lock() {
                    *locked = snap;
                } else {
                    log::error!("MCP snapshot mutex poisoned");
                }
            }
        }
    }

    // Plugin hot-reload: scan every ~5s (100 ticks at 50ms)
    if state.mcp_tick.is_multiple_of(100) {
        crate::handler_drain::refresh_plugins(ctx, state);
    }

    match ctx.command {
        CM_TICK => crate::handler_context::broadcast_context(ctx, state),
        CM_APP_QUIT => crate::handler_close::handle_app_quit(ctx, state),
        CM_TW_TAB_CLOSE | CM_TAB_CLOSE => crate::handler_close::handle_tab_close(ctx, state),
        CM_SAVE_ALL => crate::handler_close::handle_save_all(ctx),
        CM_OPEN_FILE => crate::handler_open::handle_open_file(ctx, state, false),
        CM_OPEN_FILE_FOCUS => crate::handler_open::handle_open_file(ctx, state, true),
        CM_EXECUTE_COMMAND => crate::handler_exec::handle_execute_command(ctx, state),
        CM_SHOW_HELP => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if !focus_tab_by_title(desktop, SlotId::Center, "Help") {
                    let help = HelpView::new();
                    crate::handler_evict::try_insert_tab(
                        desktop,
                        state,
                        ctx.sink,
                        SlotId::Center,
                        "Help".into(),
                        Box::new(help),
                    );
                }
            }
        }
        CM_SHOW_MESSAGES => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if focus_tab_by_title(desktop, SlotId::Tools, "Messages") {
                    desktop.focus_panel(SlotId::Tools as usize);
                } else {
                    let messages = MessagesView::new(state.messages.clone());
                    crate::handler_evict::try_insert_tab(
                        desktop,
                        state,
                        ctx.sink,
                        SlotId::Tools,
                        "Messages".into(),
                        Box::new(messages),
                    );
                    desktop.focus_panel(SlotId::Tools as usize);
                }
            }
        }
        CM_NEW_SHELL => {
            let term = new_shell_terminal();
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = next_tab_name(desktop, SlotId::Tools, "Shell");
                crate::handler_evict::try_insert_tab(desktop, state, ctx.sink, SlotId::Tools, name.clone(), term);
                ctx.sink.push_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::info("shell", format!("Started: {name}")))),
                );
            }
        }
        CM_FILE_CLOSED => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(path) = boxed.downcast_ref::<String>() {
                    state.broker.close(path);
                    state.kiro_registry.remove(path);
                    let full = state.root_dir.join(path);
                    if let Some(id) = state.buffers.find_by_path(&full.canonicalize().unwrap_or(full)) {
                        state.buffers.release(id);
                    }
                }
            }
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                crate::handler_evict::complete_pending_insert(desktop, state);
                let empty = desktop
                    .panel(SlotId::Center as usize)
                    .is_none_or(|p| p.tab_count() == 0);
                if empty {
                    insert_tab(
                        desktop,
                        SlotId::Center,
                        "Welcome",
                        Box::new(WelcomeView::new(state.root_dir.clone())),
                    );
                }
            }
        }
        CM_SHELL_OUTPUT => crate::handler_open::handle_shell_output(ctx, state),
        CM_SHOW_RESULTS => crate::handler_open::handle_show_results(ctx, state),
        CM_BUILD => crate::handler_build::handle_build(ctx, state),
        CM_RUN => crate::handler_build::handle_run(ctx, state),
        CM_TEST => crate::handler_build::handle_test(ctx, state),
        CM_TEST_FILE => crate::handler_build::handle_test_file(ctx, state),
        CM_TEST_AT_CURSOR => crate::handler_build::handle_test_at_cursor(ctx, state),
        CM_NEXT_ERROR => crate::handler_build::handle_next_error(ctx, state),
        CM_PREV_ERROR => crate::handler_build::handle_prev_error(ctx, state),
        CM_SET_GLOBAL => crate::handler_set::handle_set_global(ctx, state),
        CM_SUSPEND => crate::suspend::suspend_to_shell(),
        CM_PEEK => crate::suspend::peek_screen(),
        CM_GIT_STAGE => crate::handler_git::handle_git_stage(ctx, state),
        CM_GIT_UNSTAGE => crate::handler_git::handle_git_unstage(ctx, state),
        CM_GIT_UNTRACK => crate::handler_git::handle_git_untrack(ctx, state),
        CM_GIT_COMMIT => crate::handler_git::handle_git_commit(ctx, state),
        CM_GIT_COMMIT_PROMPT => crate::handler_git::handle_git_commit_prompt(ctx, state),
        CM_GIT_LOG => crate::handler_log::open_git_log(ctx, state, ""),
        CM_SPLIT => crate::handler_split::handle_split(ctx, state),
        CM_SPLIT_CLOSE => crate::handler_split::handle_split_close(ctx, state),
        CM_OPEN_IN_SPLIT => crate::handler_split_nav::handle_open_in_split(ctx, state),
        CM_SPLIT_FOCUS => crate::handler_split::handle_split_focus(ctx),
        CM_SPLIT_LINKED => crate::handler_split::handle_split_linked(ctx, state),
        CM_DIFF_SPLIT => crate::handler_split_nav::handle_diff_split(ctx, state),
        CM_TW_SPLIT_H => crate::handler_split::handle_split_h(ctx, state),
        CM_TW_SPLIT_V => crate::handler_split::handle_split_v(ctx, state),
        CM_TOGGLE_THEME => crate::handler_theme::handle_toggle_theme(ctx, state),
        CM_SET_SYNTAX_THEME => crate::handler_theme::handle_set_syntax_theme(ctx, state),
        CM_SET_GLYPHS => {
            if let Some(g) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                let tier = match g.as_str() {
                    "ascii" => txv_core::glyphs::GlyphTier::Ascii,
                    "utf" => txv_core::glyphs::GlyphTier::Unicode,
                    "nerd" => txv_core::glyphs::GlyphTier::Nerd,
                    _ => return,
                };
                txv_core::glyphs::set_glyphs(txv_core::glyphs::GlyphSet::from_tier(tier));
                state.settings.theme_glyphs = g.clone();
            }
        }
        CM_CURSOR_MOVED => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(pos) = boxed.downcast_ref::<txv_widgets::CursorPos>() {
                    // CursorPos is 1-indexed; LSP uses 0-indexed
                    state.cursor_pos = (pos.line().saturating_sub(1), pos.col().saturating_sub(1));
                }
            }
        }
        CM_SET_CONFIRM_CONTEXT => {
            crate::handler_confirm::handle_set_confirm_context(ctx, state);
        }
        CM_CONFIRM_RESPONSE => {
            crate::handler_confirm::handle_confirm_response(ctx, state);
        }
        CM_EDITOR_REPLACE_SELECTION
        | CM_EDITOR_DELETE_LINE
        | CM_EDITOR_REPLACE_WORD
        | CM_EDITOR_SEARCH
        | CM_EDITOR_CLEAR_HIGHLIGHT
        | CM_CHAR_INSERTED
        | CM_WORD_COMPLETED => {
            crate::handler_script::handle_script_command(ctx, state);
        }
        CM_TODO_NOTE_SAVE => crate::handler_drain::save_todo_note(ctx, state),
        CM_TODO_NOTE_OPEN => crate::handler_drain::open_todo_note(ctx, state),
        CM_TODO_NOTE_UPDATE => crate::handler_drain::update_todo_note(ctx, state),
        CM_TODO_ACTION => {
            if let Some(action) = ctx
                .data
                .as_ref()
                .and_then(|d| d.downcast_ref::<crate::mcp::commands::McpAction>())
            {
                if let Some(desktop) = downcast_desktop(ctx.desktop) {
                    if let Some(panel) = desktop.panel_mut(SlotId::Left as usize) {
                        let todo_view = panel
                            .view_at_mut(2)
                            .and_then(|v| v.as_any_mut())
                            .and_then(|a| a.downcast_mut::<crate::views::todo_tree::TodoTreeView>());
                        if let Some(tv) = todo_view {
                            if let Err(e) = tv.mcp_action(action) {
                                let msg = txv_core::message::Message::error("todo", e);
                                ctx.sink
                                    .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Downcast the desktop View to TiledWorkspace.
pub fn downcast_workspace(view: &mut dyn View) -> Option<&mut TiledWorkspace> {
    view.as_any_mut()?.downcast_mut::<TiledWorkspace>()
}

/// Deprecated alias.
pub fn downcast_desktop(view: &mut dyn View) -> Option<&mut TiledWorkspace> {
    downcast_workspace(view)
}
