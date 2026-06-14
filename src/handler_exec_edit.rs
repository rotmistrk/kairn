//! Edit/action handler functions for M-x dispatch.

use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_core::run::Waker;

use crate::commands::*;
use crate::desktop::{focus_tab_by_title, next_tab_name, SlotId};
use crate::grep::{grep_async, grep_async_roots};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::handler_log::open_git_log;
use crate::handler_lsp_cmd::handle_lsp_command as handle_lsp_cmd;
use crate::handler_open::{handle_edit_file, open_as_csv, toggle_view_mode};
use crate::handler_script_util::fire_hooks_for_event;
use crate::lsp::config_commands::format_lsp_status;
use crate::scripting::hooks::HookEvent;
use crate::views::help::HelpView;
use crate::views::problems::ProblemsView;
use crate::views::results::ResultsView;
use crate::views::terminal::new_shell_terminal;

pub(crate) fn cmd_blame(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_BLAME, None);
}

pub(crate) fn cmd_build(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_BUILD, None);
}

pub(crate) fn cmd_close(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_TAB_CLOSE, None);
}

pub(crate) fn cmd_code_action(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_CODE_ACTION, None);
}

pub(crate) fn cmd_diff(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_DIFF, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_edit(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let sink = ctx.sink().clone();
    handle_edit_file(ctx.desktop_mut(), &sink, state, arg);
}

pub(crate) fn cmd_git_commit(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_GIT_COMMIT, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_git_stage(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_GIT_STAGE, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_git_unstage(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_GIT_UNSTAGE, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_git_untrack(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_GIT_UNTRACK, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_grep(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let sink = ctx.sink().clone();
    if arg.is_empty() {
        push_status(ctx, Message::warn("grep", "Usage: :grep [-a] <pattern>"));
        return;
    }
    let (all_roots, pattern) = if let Some(rest) = arg.strip_prefix("-a ") {
        (true, rest.trim_start())
    } else {
        (false, arg)
    };
    let root = state.root_dir.clone();
    let waker = state.waker.clone().unwrap_or_else(Waker::noop);
    let grep_state = if all_roots && state.roots().len() > 1 {
        let roots: Vec<_> = state.roots().all().iter().map(|r| r.path.clone()).collect();
        grep_async_roots(pattern, &roots, waker)
    } else {
        grep_async(pattern, &root, waker)
    };
    state.grep_pending = Some((format!("grep:{pattern}"), grep_state, root.clone()));
    let title = format!("grep:{pattern}");
    let view = ResultsView::searching(&title, &root);
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        try_insert_tab(desktop, state, &sink, SlotId::Tools, title, Box::new(view));
        desktop.focus_panel(SlotId::Tools as usize);
    }
}

pub(crate) fn cmd_replace(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    use crate::handler_replace;
    handler_replace::cmd_replace(ctx, state, arg);
}

pub(crate) fn cmd_help(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        if !focus_tab_by_title(desktop, SlotId::Center, "Help") {
            try_insert_tab(
                desktop,
                state,
                &sink,
                SlotId::Center,
                "Help".into(),
                Box::new(HelpView::new()),
            );
        }
    }
}

pub(crate) fn cmd_log(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    open_git_log(ctx, state, arg);
}

pub(crate) fn cmd_lsp(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let msg = handle_lsp_cmd(arg, state);
    push_status(ctx, Message::info("lsp", msg));
}

pub(crate) fn cmd_lsp_rename(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    ctx.sink().push_command(CM_LSP_RENAME, Some(Box::new(arg.to_string())));
}

pub(crate) fn cmd_lsp_status(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let status = format_lsp_status(&state.lsp);
    ctx.sink().push_command(CM_SHELL_OUTPUT, Some(Box::new(status)));
}

pub(crate) fn cmd_messages(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_SHOW_MESSAGES, None);
}

pub(crate) fn cmd_clipboard(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    use crate::desktop::SlotId;
    use crate::handler::downcast_desktop;
    use crate::handler_evict::try_insert_tab;
    use crate::views::clipboard_viewer::ClipboardViewer;
    if let Some(d) = downcast_desktop(ctx.desktop_mut()) {
        let view = ClipboardViewer::new(state.clipboard.clone());
        try_insert_tab(d, state, &sink, SlotId::Tools, "Clipboard".into(), Box::new(view));
        d.focus_panel(SlotId::Tools as usize);
    }
}

pub(crate) fn cmd_problems(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        if !focus_tab_by_title(desktop, SlotId::Tools, "Problems") {
            let problems = ProblemsView::new(&state.root_dir);
            try_insert_tab(
                desktop,
                state,
                &sink,
                SlotId::Tools,
                "Problems".into(),
                Box::new(problems),
            );
        }
        desktop.set_hidden(SlotId::Tools as usize, false);
        desktop.focus_panel(SlotId::Tools as usize);
    }
}

pub(crate) fn cmd_next_error(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_NEXT_ERROR, None);
}

pub(crate) fn cmd_noblame(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_NOBLAME, None);
}

pub(crate) fn cmd_paste(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    if let Ok(mut ring) = state.clipboard.lock() {
        if let Some(text) = ring.paste() {
            ctx.sink().push_command(CM_CLIPBOARD_PASTE, Some(Box::new(text)));
        }
    }
}

pub(crate) fn cmd_prev_error(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_PREV_ERROR, None);
}

pub(crate) fn cmd_quit(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_APP_QUIT, None);
}

pub(crate) fn cmd_run(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_RUN, None);
}

pub(crate) fn cmd_save(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    fire_hooks_for_event(state, &HookEvent::PreSave, "", ctx);
    ctx.sink().push_command(CM_SAVE, None);
    ctx.sink().push_broadcast(CM_FS_CHANGED, None);
    fire_hooks_for_event(state, &HookEvent::FileSave, "", ctx);
}

pub(crate) fn cmd_shell(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    if let Some(desktop) = downcast_desktop(ctx.desktop_mut()) {
        let name = next_tab_name(desktop, SlotId::Tools, "Shell");
        try_insert_tab(desktop, state, &sink, SlotId::Tools, name.clone(), new_shell_terminal());
        sink.push_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("shell", format!("Started: {name}")))),
        );
    }
}

pub(crate) fn cmd_split(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let req = SplitRequest {
        vertical: false,
        file: if arg.is_empty() {
            None
        } else {
            Some(arg.to_string())
        },
    };
    ctx.sink().push_command(CM_SPLIT, Some(Box::new(req)));
}

pub(crate) fn cmd_structured(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    toggle_view_mode(ctx.desktop_mut(), &sink, state, true);
}

pub(crate) fn cmd_tab(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    open_as_csv(ctx.desktop_mut(), &sink, state);
}

pub(crate) fn cmd_test(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_TEST, None);
}

pub(crate) fn cmd_test_at_cursor(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_TEST_AT_CURSOR, None);
}

pub(crate) fn cmd_test_file(ctx: &mut CommandContext, _state: &mut AppState, _arg: &str) {
    ctx.sink().push_command(CM_TEST_FILE, None);
}

pub(crate) fn cmd_text(ctx: &mut CommandContext, state: &mut AppState, _arg: &str) {
    let sink = ctx.sink().clone();
    toggle_view_mode(ctx.desktop_mut(), &sink, state, false);
}

pub(crate) fn push_status(ctx: &mut CommandContext, msg: Message) {
    ctx.sink()
        .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
}
