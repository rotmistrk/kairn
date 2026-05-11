//! Shared command handler — used by both main.rs and test harness.
//!
//! NOTE: App command handlers call each other directly (not via queue).
//! This is correct — the queue is for cross-view communication.
//! When the App already knows what to do, it does it immediately.
//! Same pattern as TXV's Program::handle.

use std::path::PathBuf;

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::{FileBroker, OpenResult};
use crate::commands::*;
use crate::layout_group::LayoutGroup;
use crate::layout_group::SlotId;
use crate::lsp::registry::LspRegistry;
use crate::settings::AppSettings;
use crate::views::editor::EditorView;
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
        }
    }
}

/// Handle a command from the Program event loop.
/// This is the single source of truth for command handling.
pub fn handle_command(ctx: &mut CommandContext, state: &mut AppState) {
    // LSP: send didOpen on file open
    crate::lsp::handler::handle_lsp_command(ctx, state);
    // LSP: poll servers for notifications
    crate::lsp::handler::poll_lsp(state, ctx.queue);

    match ctx.command {
        CM_OPEN_FILE => handle_open_file(ctx, state, false),
        CM_OPEN_FILE_FOCUS => handle_open_file(ctx, state, true),
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
                    let messages = MessagesView::new();
                    desktop.insert_tab(SlotId::Right, "Messages", Box::new(messages));
                    desktop.focus_slot(SlotId::Right);
                }
            }
        }
        CM_NEW_SHELL => {
            let term = new_shell_terminal();
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Shell");
                desktop.insert_tab(SlotId::Right, name, term);
            }
        }
        CM_FILE_CLOSED => {
            if let Some(boxed) = ctx.data.as_ref() {
                if let Some(path) = boxed.downcast_ref::<String>() {
                    state.broker.close(path);
                }
            }
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if desktop.tab_count(SlotId::Center) == 0 {
                    desktop.insert_tab(SlotId::Center, "Welcome", Box::new(WelcomeView::new()));
                }
            }
        }
        CM_SHELL_OUTPUT => handle_shell_output(ctx),
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
        _ => {
            log::debug!("Unhandled command: {}", ctx.command);
        }
    }
}

fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState, focus_center: bool) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(req) = boxed.downcast_ref::<crate::commands::OpenFileRequest>() else {
        return;
    };
    let path = &req.path;
    let path_str = path.to_string_lossy().to_string();

    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {
            if focus_center {
                if let Some(desktop) = downcast_desktop(ctx.desktop) {
                    desktop.focus_slot(SlotId::Center);
                }
            }
        }
        OpenResult::Opened => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.close_tab_by_title(SlotId::Center, "Welcome");
                let defaults = &state.settings.editor_defaults;
                let mut editor =
                    EditorView::open(path, defaults).unwrap_or_else(|_| EditorView::new_file(path, defaults));
                editor.set_root_dir(state.root_dir.clone());
                if let (Some(line), Some(col)) = (req.line, req.col) {
                    editor.goto(line, col);
                }
                let title = path
                    .strip_prefix(&state.root_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                desktop.insert_tab(SlotId::Center, title, Box::new(editor));
                if focus_center {
                    desktop.focus_slot(SlotId::Center);
                }
            }
        }
    }
}

pub(crate) fn handle_edit_file(desktop: &mut dyn View, state: &mut AppState, arg: &str) {
    let path = state.root_dir.join(arg);
    let path_str = path.to_string_lossy().to_string();
    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {}
        OpenResult::Opened => {
            let defaults = &state.settings.editor_defaults;
            let mut editor =
                EditorView::open(&path, defaults).unwrap_or_else(|_| EditorView::new_file(&path, defaults));
            editor.set_root_dir(state.root_dir.clone());
            let title = path
                .strip_prefix(&state.root_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if let Some(d) = downcast_desktop(desktop) {
                d.insert_tab(SlotId::Center, title, Box::new(editor));
            }
        }
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

fn handle_shell_output(ctx: &mut CommandContext) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(output) = boxed.downcast_ref::<String>() else {
        return;
    };
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let view = EditorView::from_text(output);
        desktop.insert_tab(SlotId::Center, "[cmd output]", Box::new(view));
    }
}

use crate::views::welcome::WelcomeView;
