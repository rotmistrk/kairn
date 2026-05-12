//! M-x command dispatch — handles CM_EXECUTE_COMMAND.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;
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
                if !desktop.focus_tab_by_title(SlotId::Center, "Help") {
                    desktop.insert_tab(SlotId::Center, "Help", Box::new(HelpView::new()));
                }
            }
        }
        "quit" => ctx.queue.put_command(CM_QUIT, None),
        "edit" | "e" if !arg.is_empty() => crate::handler_open::handle_edit_file(ctx.desktop, state, arg),
        "save" => ctx.queue.put_command(CM_SAVE, None),
        "close" => ctx.queue.put_command(CM_TAB_CLOSE, None),
        "tab-rename" if !arg.is_empty() => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let slot = desktop.focused_slot();
                let old_title = desktop.active_tab_title(slot).map(String::from);
                desktop.rename_focused_tab(arg);
                if let Some(old) = old_title {
                    if state.kiro_registry.contains(&old) {
                        let new_title = desktop.active_tab_title(slot).map(String::from);
                        if let Some(new) = new_title {
                            state.kiro_registry.rename(&old, &new);
                        }
                    }
                }
            }
        }
        "lsp-rename" if !arg.is_empty() => {
            ctx.queue.put_command(CM_LSP_RENAME, Some(Box::new(arg.to_string())));
        }
        "shell" => {
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                let name = desktop.next_tab_name(SlotId::Right, "Shell");
                desktop.insert_tab(SlotId::Right, &name, new_shell_terminal());
                ctx.queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(txv_core::message::Message::info(
                        "shell",
                        format!("Started: {name}"),
                    ))),
                );
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
                    Some("kairn")
                };
                let term = new_kiro_terminal(agent_arg, &state.root_dir);
                desktop.insert_tab(SlotId::Right, &name, term);
                state.kiro_registry.register(&name);
                ctx.queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(txv_core::message::Message::info(
                        "kiro",
                        format!("Started: {name}"),
                    ))),
                );
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
        "grep" if !arg.is_empty() => {
            let pattern = arg.to_string();
            let root = state.root_dir.clone();
            let rx = crate::grep::grep_stream(&pattern, &root);
            state.pending_grep = Some((format!("grep: {arg}"), rx, Vec::new()));
            ctx.queue.put_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(txv_core::message::Message::info("grep", "Searching..."))),
            );
        }
        "test-file" => ctx.queue.put_command(CM_TEST_FILE, None),
        "test-at-cursor" => ctx.queue.put_command(CM_TEST_AT_CURSOR, None),
        "next-error" => ctx.queue.put_command(CM_NEXT_ERROR, None),
        "prev-error" => ctx.queue.put_command(CM_PREV_ERROR, None),
        "code-action" => ctx.queue.put_command(CM_CODE_ACTION, None),
        "grow" => ctx.queue.put_command(CM_PANEL_GROW, None),
        "shrink" => ctx.queue.put_command(CM_PANEL_SHRINK, None),
        "grow-v" => ctx.queue.put_command(CM_PANEL_GROW_V, None),
        "shrink-v" => ctx.queue.put_command(CM_PANEL_SHRINK_V, None),
        "paste" => {
            if let Some(text) = crate::clipboard::paste_from_clipboard() {
                ctx.queue.put_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
            }
        }
        "git-stage" if !arg.is_empty() => {
            ctx.queue.put_command(CM_GIT_STAGE, Some(Box::new(arg.to_string())));
        }
        "git-unstage" if !arg.is_empty() => {
            ctx.queue.put_command(CM_GIT_UNSTAGE, Some(Box::new(arg.to_string())));
        }
        "git-untrack" if !arg.is_empty() => {
            ctx.queue.put_command(CM_GIT_UNTRACK, Some(Box::new(arg.to_string())));
        }
        "git-commit" if !arg.is_empty() => {
            ctx.queue.put_command(CM_GIT_COMMIT, Some(Box::new(arg.to_string())));
        }
        "diff" => {
            ctx.queue.put_command(CM_DIFF, Some(Box::new(arg.to_string())));
        }
        _ => {
            let msg = txv_core::message::Message::warn("handler", format!("Unknown command: {cmd}"));
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}
