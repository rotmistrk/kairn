//! Git command handlers — stage, unstage, untrack, commit.

use txv_core::message::Message;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::handler::AppState;

pub fn handle_git_stage(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match crate::git_ops::git_stage(&state.root_dir, rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Staged: {rel}"));
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_unstage(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match crate::git_ops::git_unstage(&state.root_dir, rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Unstaged: {rel}"));
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_untrack(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match crate::git_ops::git_untrack(&state.root_dir, rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Untracked: {rel}"));
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_commit(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(msg) = boxed.downcast_ref::<String>() else {
        return;
    };
    if msg.is_empty() {
        return;
    }
    match crate::git_ops::git_commit(&state.root_dir, msg) {
        Ok(()) => {
            let m = Message::info("git", format!("Committed: {msg}"));
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
        Err(e) => {
            let m = Message::error("git", e);
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
    }
}

pub fn handle_git_commit_prompt(ctx: &mut CommandContext, _state: &AppState) {
    let prefill = "git-commit ".to_string();
    ctx.queue.put_command(CM_COMMAND_PREFILL, Some(Box::new(prefill)));
}
