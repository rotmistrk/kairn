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
use crate::desktop::{SlotId, SlottedDesktop};
use crate::lsp::registry::LspRegistry;
use crate::settings::AppSettings;
use crate::views::editor::EditorView;
use crate::views::help::HelpView;
use crate::views::messages::MessagesView;
use crate::views::terminal::{new_kiro_terminal, new_shell_terminal};

/// Application state shared across command handler invocations.
pub struct AppState {
    pub broker: FileBroker,
    pub root_dir: PathBuf,
    pub settings: AppSettings,
    pub lsp: LspRegistry,
    pub(crate) lsp_pending: crate::lsp::handler::PendingRequests,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            broker: FileBroker::new(),
            root_dir,
            settings: AppSettings::default(),
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
        }
    }

    pub fn with_settings(root_dir: PathBuf, settings: AppSettings) -> Self {
        Self {
            broker: FileBroker::new(),
            root_dir,
            settings,
            lsp: LspRegistry::new(),
            lsp_pending: Default::default(),
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
        CM_EXECUTE_COMMAND => handle_execute_command(ctx, state),
        CM_SHOW_HELP => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let help = HelpView::new();
                desktop.insert_tab(SlotId::Center, "Help", Box::new(help));
            }
        }
        CM_SHOW_MESSAGES => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let messages = MessagesView::new();
                desktop.insert_tab(SlotId::Bottom, "Messages", Box::new(messages));
                desktop.focus_slot(SlotId::Bottom);
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
                if let Some(title) = boxed.downcast_ref::<String>() {
                    let full_path = state.root_dir.join(title).to_string_lossy().to_string();
                    state.broker.close(&full_path);
                }
            }
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                if desktop.tab_count(SlotId::Center) == 0 {
                    desktop.insert_tab(SlotId::Center, "Welcome", Box::new(WelcomeView::new()));
                }
            }
        }
        CM_SHELL_OUTPUT => handle_shell_output(ctx),
        CM_SET_GLOBAL => handle_set_global(ctx, state),
        CM_SUSPEND => crate::suspend::suspend_to_shell(),
        CM_PEEK => crate::suspend::peek_screen(),
        _ => {
            log::debug!("Unhandled command: {}", ctx.command);
        }
    }
}

fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState, focus_center: bool) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(path) = boxed.downcast_ref::<PathBuf>() else {
        return;
    };
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

fn handle_execute_command(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(text) = boxed.downcast_ref::<String>() else {
        return;
    };
    log::debug!("execute_command: {:?}", text);

    let parts: Vec<&str> = text.trim().splitn(2, ' ').collect();
    let cmd = parts.first().copied().unwrap_or("");
    let arg = parts.get(1).copied().unwrap_or("");

    match cmd {
        "help" => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.insert_tab(SlotId::Center, "Help", Box::new(HelpView::new()));
            }
        }
        "quit" => ctx.queue.put_command(CM_QUIT, None),
        "edit" | "e" if !arg.is_empty() => handle_edit_file(ctx.desktop, state, arg),
        "save" => ctx.queue.put_command(CM_SAVE, None),
        "close" => ctx.queue.put_command(CM_TAB_CLOSE, None),
        "rename" if !arg.is_empty() => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.rename_focused_tab(arg);
            }
        }
        "shell" => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Shell");
                desktop.insert_tab(SlotId::Right, name, new_shell_terminal());
            }
        }
        "kiro" => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Kiro");
                let agent_arg = if arg.starts_with("--agent=") {
                    Some(arg.trim_start_matches("--agent="))
                } else if !arg.is_empty() {
                    Some(arg)
                } else {
                    None
                };
                let term = new_kiro_terminal(agent_arg);
                desktop.insert_tab(SlotId::Right, name, term);
            }
        }
        "messages" => ctx.queue.put_command(CM_SHOW_MESSAGES, None),
        "paste" => {
            if let Some(text) = crate::clipboard::paste_from_clipboard() {
                ctx.queue.put_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
            }
        }
        _ => {}
    }
}

fn handle_edit_file(desktop: &mut dyn View, state: &mut AppState, arg: &str) {
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

/// Downcast the desktop View to SlottedDesktop.
pub fn downcast_desktop(view: &mut dyn View) -> Option<&mut SlottedDesktop> {
    let ptr = view as *mut dyn View;
    // SAFETY: we know the desktop is a SlottedDesktop (we created it).
    unsafe { (ptr as *mut SlottedDesktop).as_mut() }
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
