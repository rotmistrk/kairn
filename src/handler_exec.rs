//! M-x command dispatch — handles CM_EXECUTE_COMMAND.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::views::help::HelpView;
use crate::views::terminal::{new_kiro_terminal, new_shell_terminal};

/// Handle the M-x command dispatch.
pub fn handle_execute_command(ctx: &mut CommandContext, state: &mut AppState) {
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
        "edit" | "e" if !arg.is_empty() => crate::handler::handle_edit_file(ctx.desktop, state, arg),
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
        "lsp-status" => {
            let status = crate::lsp::config_commands::format_lsp_status(&state.lsp);
            ctx.queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(status)));
        }
        "build" => ctx.queue.put_command(CM_BUILD, None),
        "run" => ctx.queue.put_command(CM_RUN, None),
        "test" => ctx.queue.put_command(CM_TEST, None),
        "test-file" => ctx.queue.put_command(CM_TEST_FILE, None),
        "test-at-cursor" => ctx.queue.put_command(CM_TEST_AT_CURSOR, None),
        "paste" => {
            if let Some(text) = crate::clipboard::paste_from_clipboard() {
                ctx.queue.put_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
            }
        }
        _ => {}
    }
}
