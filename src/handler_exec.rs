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
                    crate::handler_evict::try_insert_tab(
                        desktop,
                        state,
                        SlotId::Center,
                        "Help".into(),
                        Box::new(HelpView::new()),
                    );
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
                crate::handler_evict::try_insert_tab(desktop, state, SlotId::Right, name.clone(), new_shell_terminal());
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
                crate::handler_evict::try_insert_tab(desktop, state, SlotId::Right, name.clone(), term);
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
        "theme" => {
            if arg == "dark" || arg == "light" || arg == "toggle" || arg.is_empty() {
                ctx.queue.put_command(CM_TOGGLE_THEME, Some(Box::new(arg.to_string())));
            }
        }
        "lsp-status" => {
            let status = crate::lsp::config_commands::format_lsp_status(&state.lsp);
            ctx.queue.put_command(CM_SHELL_OUTPUT, Some(Box::new(status)));
        }
        "build" => ctx.queue.put_command(CM_BUILD, None),
        "run" => ctx.queue.put_command(CM_RUN, None),
        "test" => ctx.queue.put_command(CM_TEST, None),
        "grep" if !arg.is_empty() => {
            let root = state.root_dir.clone();
            let waker = state.waker.clone().unwrap_or_else(txv_core::run::Waker::noop);
            let grep_state = crate::grep::grep_async(arg, &root, waker);
            state.grep_pending = Some((format!("grep:{arg}"), grep_state, root.clone()));
            let title = format!("grep:{arg}");
            let view = crate::views::results::ResultsView::searching(&title, &root);
            if let Some(desktop) = downcast_desktop(ctx.desktop) {
                crate::handler_evict::try_insert_tab(desktop, state, SlotId::Right, title, Box::new(view));
                desktop.focus_slot(SlotId::Right);
            }
        }
        "grep" => {
            let msg = txv_core::message::Message::warn("grep", "Usage: :grep <pattern>");
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
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
        "paste" => match crate::clipboard::paste_from_clipboard() {
            Ok(text) => {
                ctx.queue.put_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
            }
            Err(e) => {
                let msg = txv_core::message::Message::error("clipboard", e);
                ctx.queue
                    .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
        },
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
        "struct" | "structured" => {
            crate::handler_open::toggle_view_mode(ctx.desktop, state, true);
        }
        "text" => {
            crate::handler_open::toggle_view_mode(ctx.desktop, state, false);
        }
        _ => {
            let msg = txv_core::message::Message::warn("handler", format!("Unknown command: {cmd}"));
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}
