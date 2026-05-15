//! Shared command handler — used by both main.rs and test harness.
//!
//! NOTE: App command handlers call each other directly (not via queue).
//! This is correct — the queue is for cross-view communication.
//! When the App already knows what to do, it does it immediately.
//! Same pattern as TXV's Program::handle.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

pub use crate::app_state::AppState;
use crate::commands::*;
use crate::layout_group::LayoutGroup;
use crate::layout_group::SlotId;
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
    crate::lsp::handler::poll_lsp(state, ctx.queue);

    // Drain background tasks (grep, build)
    crate::handler_drain::drain_grep(ctx, state);
    crate::handler_drain::drain_build(ctx, state);

    // Drain MCP write commands
    crate::handler_drain::drain_mcp(ctx, state);

    // MCP: update snapshot every 20 commands (~1s at 50ms tick)
    if state.mcp_snapshot.is_some() {
        state.mcp_tick = state.mcp_tick.wrapping_add(1);
        if state.mcp_tick.is_multiple_of(20) {
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
    }

    match ctx.command {
        CM_TICK => crate::handler_context::broadcast_context(ctx, state),
        CM_OPEN_FILE => crate::handler_open::handle_open_file(ctx, state, false),
        CM_OPEN_FILE_FOCUS => crate::handler_open::handle_open_file(ctx, state, true),
        CM_EXECUTE_COMMAND => crate::handler_exec::handle_execute_command(ctx, state),
        CM_SHOW_HELP => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if !desktop.focus_tab_by_title(SlotId::Center, "Help") {
                    let help = HelpView::new();
                    crate::handler_evict::try_insert_tab(
                        desktop,
                        state,
                        ctx.queue,
                        SlotId::Center,
                        "Help".into(),
                        Box::new(help),
                    );
                }
            }
        }
        CM_SHOW_MESSAGES => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if desktop.focus_tab_by_title(SlotId::Right, "Messages") {
                    desktop.focus_slot(SlotId::Right);
                } else {
                    let messages = MessagesView::new(state.messages.clone());
                    crate::handler_evict::try_insert_tab(
                        desktop,
                        state,
                        ctx.queue,
                        SlotId::Right,
                        "Messages".into(),
                        Box::new(messages),
                    );
                    desktop.focus_slot(SlotId::Right);
                }
            }
        }
        CM_NEW_SHELL => {
            let term = new_shell_terminal();
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Shell");
                crate::handler_evict::try_insert_tab(desktop, state, ctx.queue, SlotId::Right, name.clone(), term);
                ctx.queue.put_command(
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
                }
            }
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                crate::handler_evict::complete_pending_insert(desktop, state);
                if desktop.tab_count(SlotId::Center) == 0 {
                    desktop.insert_tab(
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
        CM_SET_GLOBAL => handle_set_global(ctx, state),
        CM_SUSPEND => crate::suspend::suspend_to_shell(),
        CM_PEEK => crate::suspend::peek_screen(),
        CM_GIT_STAGE => crate::handler_git::handle_git_stage(ctx, state),
        CM_GIT_UNSTAGE => crate::handler_git::handle_git_unstage(ctx, state),
        CM_GIT_UNTRACK => crate::handler_git::handle_git_untrack(ctx, state),
        CM_GIT_COMMIT => crate::handler_git::handle_git_commit(ctx, state),
        CM_GIT_COMMIT_PROMPT => crate::handler_git::handle_git_commit_prompt(ctx, state),
        CM_DIFF => {} // Handled by the focused editor view directly
        CM_TOGGLE_THEME => {
            let arg = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned();
            if let Some(ref ts) = state.theme_state {
                let mut ts = ts.borrow_mut();
                match arg.as_deref() {
                    Some("dark") => {
                        ts.mode = txv_core::palette::ThemeMode::Dark;
                        ts.active = ts.dark.clone();
                        ts.apply();
                    }
                    Some("light") => {
                        ts.mode = txv_core::palette::ThemeMode::Light;
                        ts.active = ts.light.clone();
                        ts.apply();
                    }
                    Some("auto") => {
                        let detected = txv_core::palette::detect_system_theme();
                        ts.mode = detected.clone();
                        ts.active = match detected {
                            txv_core::palette::ThemeMode::Light => ts.light.clone(),
                            _ => ts.dark.clone(),
                        };
                        ts.apply();
                    }
                    _ => ts.toggle(),
                }
            }
        }
        CM_SET_SYNTAX_THEME => {
            if let Some(name) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                // Store in settings for the current mode
                let is_light = state
                    .theme_state
                    .as_ref()
                    .map(|ts| ts.borrow().mode == txv_core::palette::ThemeMode::Light)
                    .unwrap_or(false);
                if is_light {
                    state.settings.theme_syntax_light = name.clone();
                } else {
                    state.settings.theme_syntax_dark = name.clone();
                }
                // Apply to all open editors
                if let Some(desktop) = downcast_desktop(ctx.desktop) {
                    for slot in [SlotId::Center, SlotId::Right] {
                        let panel = desktop.panel_mut(slot);
                        for i in 0..panel.tab_count() {
                            if let Some(view) = panel.view_at_mut(i) {
                                if let Some(any) = view.as_any_mut() {
                                    if let Some(editor) = any.downcast_mut::<crate::views::editor::EditorView>() {
                                        editor.set_syntax_theme(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
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
                    state.cursor_pos = (pos.line.saturating_sub(1), pos.col.saturating_sub(1));
                }
            }
        }
        CM_SET_CONFIRM_CONTEXT => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(context) = boxed.downcast_ref::<crate::commands::ConfirmContext>() {
                    state.confirm_context = Some(context.clone());
                }
            }
        }
        CM_CONFIRM_RESPONSE => {
            crate::handler_confirm::handle_confirm_response(ctx, state);
        }
        _ => {}
    }
}

fn handle_set_global(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(opt) = boxed.downcast_ref::<String>() else {
        return;
    };
    let defaults = &mut state.settings.editor_defaults;
    match opt.as_str() {
        "wrap" => defaults.wrap = true,
        "nowrap" => defaults.wrap = false,
        "list" | "li" => defaults.list = true,
        "nolist" | "noli" => defaults.list = false,
        "number" | "nu" => defaults.number = true,
        "nonumber" | "nonu" => defaults.number = false,
        _ => {}
    }
}

/// Downcast the desktop View to LayoutGroup.
pub fn downcast_desktop(view: &mut dyn View) -> Option<&mut LayoutGroup> {
    view.as_any_mut()?.downcast_mut::<LayoutGroup>()
}
