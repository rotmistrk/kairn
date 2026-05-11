//! Git command handlers — stage, unstage, untrack, commit.

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
            let msg = format!("Staged: {rel}");
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(e)));
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
            let msg = format!("Unstaged: {rel}");
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(e)));
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
            let msg = format!("Untracked: {rel}");
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(e)));
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
            let status = format!("Committed: {msg}");
            ctx.queue
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(status)));
        }
        Err(e) => {
            ctx.queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(e)));
        }
    }
}

pub fn handle_git_commit_prompt(ctx: &mut CommandContext, _state: &AppState) {
    // Emit prefill command — CommandItem activates with this prefix
    let prefill = "git-commit ".to_string();
    ctx.queue.put_command(CM_COMMAND_PREFILL, Some(Box::new(prefill)));
}
