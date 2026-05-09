//! Shared command handler — used by both main.rs and test harness.
//!
//! NOTE: App command handlers call each other directly (not via queue).
//! This is correct — the queue is for cross-view communication.
//! When the App already knows what to do, it does it immediately.
//! Same pattern as TXV's Program::handle.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::broker::{FileBroker, OpenResult};
use crate::commands::*;
use crate::desktop::{SlotId, SlottedDesktop};
use crate::views::editor::EditorView;
use crate::views::help::HelpView;
use crate::views::terminal::TerminalView;
use crate::views::tree::FileTreeView;

/// Application state shared across command handler invocations.
pub struct AppState {
    pub broker: FileBroker,
    pub root_dir: PathBuf,
}

impl AppState {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            broker: FileBroker::new(),
            root_dir,
        }
    }
}

/// Handle a command from the Program event loop.
/// This is the single source of truth for command handling.
pub fn handle_command(ctx: &mut CommandContext, state: &mut AppState) {
    match ctx.command {
        CM_OPEN_FILE => handle_open_file(ctx, state),
        CM_EXECUTE_COMMAND => handle_execute_command(ctx, state),
        CM_SHOW_HELP => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let help = HelpView::new();
                desktop.insert_tab(SlotId::Center, "Help", Box::new(help));
            }
        }
        CM_NEW_SHELL => {
            let term = TerminalView::new("Shell");
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
            }
        }
        _ => {
            log::debug!("Unhandled command: {}", ctx.command);
        }
    }
}

fn handle_open_file(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(path) = boxed.downcast_ref::<PathBuf>() else {
        return;
    };
    let path_str = path.to_string_lossy().to_string();

    match state.broker.open(&path_str, SlotId::Center, 0) {
        OpenResult::AlreadyOpen { .. } => {}
        OpenResult::Opened => {
            let editor = EditorView::open(path).unwrap_or_else(|_| EditorView::new_file(path));
            let title = editor.title().to_string();
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.insert_tab(SlotId::Center, title, Box::new(editor));
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
                let help = HelpView::new();
                desktop.insert_tab(SlotId::Center, "Help", Box::new(help));
            }
        }
        "quit" => ctx.queue.put_command(CM_QUIT, None),
        "edit" | "e" if !arg.is_empty() => {
            let path = state.root_dir.join(arg);
            // NOTE: App command handlers call each other directly (not via queue).
            // This is correct — the queue is for cross-view communication.
            // When the App already knows what to do, it does it immediately.
            // Same pattern as TXV's Program::handle.
            let path_str = path.to_string_lossy().to_string();
            match state.broker.open(&path_str, SlotId::Center, 0) {
                OpenResult::AlreadyOpen { .. } => {}
                OpenResult::Opened => {
                    let editor = EditorView::open(&path).unwrap_or_else(|_| EditorView::new_file(&path));
                    let title = editor.title().to_string();
                    if let Some(d) = downcast_desktop(ctx.desktop) {
                        d.insert_tab(SlotId::Center, title, Box::new(editor));
                    }
                }
            }
        }
        "save" => ctx.queue.put_command(CM_SAVE, None),
        "close" => ctx.queue.put_command(CM_TAB_CLOSE, None),
        "shell" => {
            let term = TerminalView::new("Shell");
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
            }
        }
        _ => {}
    }
}

/// Downcast the desktop View to SlottedDesktop.
pub fn downcast_desktop(view: &mut dyn View) -> Option<&mut SlottedDesktop> {
    let ptr = view as *mut dyn View;
    // SAFETY: we know the desktop is a SlottedDesktop (we created it).
    unsafe { (ptr as *mut SlottedDesktop).as_mut() }
}

/// Build the standard kairn desktop with tree and terminal.
pub fn build_desktop(root_dir: &Path) -> SlottedDesktop {
    let mut desktop = SlottedDesktop::new();
    let tree = FileTreeView::new(root_dir.to_path_buf());
    desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));
    let term = TerminalView::new("Shell");
    desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
    desktop
}
