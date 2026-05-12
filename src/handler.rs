//! Shared command handler — used by both main.rs and test harness.
//!
//! NOTE: App command handlers call each other directly (not via queue).
//! This is correct — the queue is for cross-view communication.
//! When the App already knows what to do, it does it immediately.
//! Same pattern as TXV's Program::handle.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::FileBroker;
use crate::commands::*;
use crate::kiro_registry::KiroTabRegistry;
use crate::layout_group::LayoutGroup;
use crate::layout_group::SlotId;
use crate::lsp::registry::LspRegistry;
use crate::message_ring::MessageRing;
use crate::settings::AppSettings;
use crate::views::help::HelpView;
use crate::views::messages::MessagesView;
use crate::views::terminal::new_shell_terminal;

/// Application state shared across command handler invocations.
pub struct AppState {
    pub broker: FileBroker,
    pub root_dir: PathBuf,
    pub settings: AppSettings,
    pub lsp: LspRegistry,
    pub(crate) lsp_pending: crate::lsp::handler::PendingRequests,
    pub build_errors: Vec<crate::build::ErrorLocation>,
    pub build_error_idx: usize,
    /// Last known cursor position (0-indexed line, col) from the editor.
    pub cursor_pos: (u32, u32),
    /// Shared message ring buffer.
    pub messages: Arc<Mutex<MessageRing>>,
    /// Registry of active kiro tabs for session persistence.
    pub kiro_registry: KiroTabRegistry,
    /// LSP document version counters (keyed by file path string).
    pub doc_versions: std::collections::HashMap<String, i64>,
    /// MCP snapshot (updated periodically for MCP server reads).
    pub mcp_snapshot: Option<Arc<Mutex<crate::mcp::snapshot::McpSnapshot>>>,
    mcp_tick: u16,
    pub waker: Option<txv_core::run::Waker>,
    pub grep_pending: Option<(String, std::sync::Arc<crate::grep::GrepState>, std::path::PathBuf)>,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            broker: FileBroker::new(),
            root_dir,
            settings: AppSettings::default(),
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
            build_errors: Vec::new(),
            build_error_idx: 0,
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            doc_versions: std::collections::HashMap::new(),
            mcp_snapshot: None,
            mcp_tick: 0,
            waker: None,
            grep_pending: None,
        }
    }

    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        Self {
            broker: FileBroker::new(),
            root_dir,
            settings,
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
            build_errors: Vec::new(),
            build_error_idx: 0,
            cursor_pos: (0, 0),
            messages: Arc::new(Mutex::new(MessageRing::new())),
            kiro_registry: KiroTabRegistry::default(),
            doc_versions: std::collections::HashMap::new(),
            mcp_snapshot: None,
            mcp_tick: 0,
            waker: None,
            grep_pending: None,
        }
    }
}

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

    // Grep: drain results from background thread into the results view
    if let Some((title, gs, root)) = state.grep_pending.take() {
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

    // MCP: update snapshot every 20 commands (~1s at 50ms tick)
    if state.mcp_snapshot.is_some() {
        state.mcp_tick = state.mcp_tick.wrapping_add(1);
        if state.mcp_tick.is_multiple_of(20) {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let mut snap = crate::mcp::collect::collect_snapshot(desktop);
                snap.terminals = crate::mcp::collect::collect_terminal_content(desktop);
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
        CM_OPEN_FILE => crate::handler_open::handle_open_file(ctx, state, false),
        CM_OPEN_FILE_FOCUS => crate::handler_open::handle_open_file(ctx, state, true),
        CM_EXECUTE_COMMAND => crate::handler_exec::handle_execute_command(ctx, state),
        CM_SHOW_HELP => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if !desktop.focus_tab_by_title(SlotId::Center, "Help") {
                    let help = HelpView::new();
                    desktop.insert_tab(SlotId::Center, "Help", Box::new(help));
                }
            }
        }
        CM_SHOW_MESSAGES => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if desktop.focus_tab_by_title(SlotId::Right, "Messages") {
                    desktop.focus_slot(SlotId::Right);
                } else {
                    let messages = MessagesView::new(state.messages.clone());
                    desktop.insert_tab(SlotId::Right, "Messages", Box::new(messages));
                    desktop.focus_slot(SlotId::Right);
                }
            }
        }
        CM_NEW_SHELL => {
            let term = new_shell_terminal();
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Shell");
                desktop.insert_tab(SlotId::Right, &name, term);
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
                if desktop.tab_count(SlotId::Center) == 0 {
                    desktop.insert_tab(SlotId::Center, "Welcome", Box::new(WelcomeView::new()));
                }
            }
        }
        CM_SHELL_OUTPUT => crate::handler_open::handle_shell_output(ctx),
        CM_SHOW_RESULTS => crate::handler_open::handle_show_results(ctx),
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
        CM_CURSOR_MOVED => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(pos) = boxed.downcast_ref::<txv_widgets::CursorPos>() {
                    // CursorPos is 1-indexed; LSP uses 0-indexed
                    state.cursor_pos = (pos.line.saturating_sub(1), pos.col.saturating_sub(1));
                }
            }
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

use crate::views::welcome::WelcomeView;
