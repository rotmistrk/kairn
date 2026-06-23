//! Git command handlers — stage, unstage, untrack, commit.

use std::path::PathBuf;

use txv_core::message::Message;
use txv_core::program::CommandContext;

use crate::commands::*;
use crate::git_ops::{git_commit, git_stage, git_unstage, git_untrack};
use crate::handler::AppState;

pub fn handle_git_stage(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match git_stage(state.root_dir(), rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Staged: {rel}"));
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_unstage(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match git_unstage(state.root_dir(), rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Unstaged: {rel}"));
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_untrack(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(rel) = boxed.downcast_ref::<String>() else {
        return;
    };
    match git_untrack(state.root_dir(), rel) {
        Ok(()) => {
            let msg = Message::info("git", format!("Untracked: {rel}"));
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        Err(e) => {
            let msg = Message::error("git", e);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }
}

pub fn handle_git_commit(ctx: &mut CommandContext, state: &AppState) {
    let Some(boxed) = ctx.data().as_ref() else {
        return;
    };
    let Some(msg) = boxed.downcast_ref::<String>() else {
        return;
    };
    if msg.is_empty() {
        return;
    }
    match git_commit(state.root_dir(), msg) {
        Ok(()) => {
            let m = Message::info("git", format!("Committed: {msg}"));
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
        Err(e) => {
            let m = Message::error("git", e);
            ctx.sink()
                .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(m)));
        }
    }
}

pub fn handle_git_commit_prompt(ctx: &mut CommandContext, _state: &AppState) {
    let prefill = "git-commit ".to_string();
    ctx.sink().push_command(CM_COMMAND_PREFILL, Some(Box::new(prefill)));
}

pub fn handle_git_set_base(ctx: &mut CommandContext, state: &mut AppState) {
    let payload = ctx
        .data()
        .as_ref()
        .and_then(|d| d.downcast_ref::<Option<(PathBuf, String)>>())
        .cloned()
        .unwrap_or(None);

    let msg = match &payload {
        Some((root, hash)) => {
            state.diff_base.insert(root.clone(), hash.clone());
            Message::info("git", format!("Diff base set to {hash}"))
        }
        None => {
            state.diff_base.clear();
            Message::info("git", "Diff base reset to HEAD".to_string())
        }
    };
    ctx.sink()
        .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    // Notify views via a DIFFERENT command (non-reentrant)
    ctx.sink()
        .push_broadcast(CM_GIT_BASE_CHANGED, Some(Box::new(state.diff_base.clone())));
}
