//! Script command dispatch — git, todo, split, lsp, grep variants.

use std::path::PathBuf;

use txv_core::program::CommandContext;
use txv_core::run::Waker;

use crate::app_state::AppState;
use crate::commands::*;
use crate::grep::grep_async;
use crate::handler_script_util::lsp_cmd;
use crate::mcp::commands::McpAction;
use crate::scripting::ScriptCommand;

/// Dispatch git/todo/split/lsp/grep script commands.
/// Returns true if the command was handled, false otherwise.
pub(crate) fn dispatch_extended(cmd: ScriptCommand, ctx: &mut CommandContext, state: &mut AppState) -> bool {
    match cmd {
        ScriptCommand::GitStage { .. }
        | ScriptCommand::GitUnstage { .. }
        | ScriptCommand::GitCommit { .. }
        | ScriptCommand::GitBlame
        | ScriptCommand::GitNoBlame
        | ScriptCommand::GitUntrack { .. }
        | ScriptCommand::GitLog
        | ScriptCommand::GitDiff => dispatch_git(cmd, ctx),
        ScriptCommand::TodoAdd { .. }
        | ScriptCommand::TodoRemove { .. }
        | ScriptCommand::TodoComplete { .. }
        | ScriptCommand::TodoToggleImportant { .. }
        | ScriptCommand::TodoEdit { .. }
        | ScriptCommand::TodoSwap { .. }
        | ScriptCommand::TodoPromote { .. }
        | ScriptCommand::TodoDemote { .. }
        | ScriptCommand::TodoList => dispatch_todo(cmd, ctx),
        ScriptCommand::SplitVertical { .. }
        | ScriptCommand::SplitHorizontal { .. }
        | ScriptCommand::SplitClose
        | ScriptCommand::SplitFocus
        | ScriptCommand::SplitOpen { .. }
        | ScriptCommand::SplitLinked { .. }
        | ScriptCommand::DiffRevert => dispatch_split(cmd, ctx),
        ScriptCommand::Grep { pattern } => {
            let root = state.root_dir.clone();
            let waker = state.waker.clone().unwrap_or_else(Waker::noop);
            let grep_state = grep_async(&pattern, &root, waker);
            state.grep_pending = Some((format!("grep:{pattern}"), grep_state, root));
            true
        }
        ScriptCommand::LspStart { .. }
        | ScriptCommand::LspRestart { .. }
        | ScriptCommand::LspStop { .. }
        | ScriptCommand::LspTimeout { .. }
        | ScriptCommand::LspArgs { .. }
        | ScriptCommand::LspEnv { .. } => dispatch_lsp_control(cmd, ctx, state),
        _ => false,
    }
}

fn dispatch_lsp_control(cmd: ScriptCommand, ctx: &mut CommandContext, state: &mut AppState) -> bool {
    match cmd {
        ScriptCommand::LspStart { pattern } => lsp_cmd(ctx, state, &format!("start {pattern}")),
        ScriptCommand::LspRestart { pattern } => lsp_cmd(ctx, state, &format!("restart {pattern}")),
        ScriptCommand::LspStop { pattern } => lsp_cmd(ctx, state, &format!("stop {pattern}")),
        ScriptCommand::LspTimeout { pattern, secs } => {
            let arg = match secs {
                Some(s) => format!("timeout {pattern} {s}"),
                None => format!("timeout {pattern}"),
            };
            lsp_cmd(ctx, state, &arg);
        }
        ScriptCommand::LspArgs { pattern, command } => {
            lsp_cmd(ctx, state, &format!("args {pattern} {command}"));
        }
        ScriptCommand::LspEnv { pattern, key, value } => {
            for lang in state.lsp.matching_languages(&pattern) {
                state.lsp.set_env(&lang, key.clone(), value.clone());
            }
        }
        _ => return false,
    }
    true
}

fn dispatch_git(cmd: ScriptCommand, ctx: &mut CommandContext) -> bool {
    match cmd {
        ScriptCommand::GitStage { file } => ctx.sink.push_command(CM_GIT_STAGE, Some(Box::new(file))),
        ScriptCommand::GitUnstage { file } => ctx.sink.push_command(CM_GIT_UNSTAGE, Some(Box::new(file))),
        ScriptCommand::GitCommit { message } => ctx.sink.push_command(CM_GIT_COMMIT, Some(Box::new(message))),
        ScriptCommand::GitBlame => ctx.sink.push_command(CM_BLAME, None),
        ScriptCommand::GitNoBlame => ctx.sink.push_command(CM_NOBLAME, None),
        ScriptCommand::GitUntrack { file } => ctx.sink.push_command(CM_GIT_UNTRACK, Some(Box::new(file))),
        ScriptCommand::GitLog => ctx.sink.push_command(CM_GIT_LOG, None),
        ScriptCommand::GitDiff => ctx.sink.push_command(CM_DIFF, None),
        _ => {}
    }
    true
}

fn dispatch_todo(cmd: ScriptCommand, ctx: &mut CommandContext) -> bool {
    let action = match cmd {
        ScriptCommand::TodoAdd { text, parent } => {
            let path = parse_todo_path(&parent.unwrap_or_default());
            McpAction::TodoAdd { path, title: text }
        }
        ScriptCommand::TodoRemove { path } => McpAction::TodoRemove {
            path: parse_todo_path(&path),
        },
        ScriptCommand::TodoComplete { path } => McpAction::TodoToggle {
            path: parse_todo_path(&path),
        },
        ScriptCommand::TodoToggleImportant { path } => McpAction::TodoToggleImportant {
            path: parse_todo_path(&path),
        },
        ScriptCommand::TodoEdit { path, text } => McpAction::TodoEdit {
            path: parse_todo_path(&path),
            title: text,
        },
        ScriptCommand::TodoSwap { path, direction } => {
            let p = parse_todo_path(&path);
            if direction == "up" {
                McpAction::TodoMoveUp { path: p }
            } else {
                McpAction::TodoMoveDown { path: p }
            }
        }
        ScriptCommand::TodoPromote { path } => McpAction::TodoPromote {
            path: parse_todo_path(&path),
        },
        ScriptCommand::TodoDemote { path } => McpAction::TodoDemote {
            path: parse_todo_path(&path),
        },
        ScriptCommand::TodoList => return true,
        _ => return false,
    };
    ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
    true
}

fn dispatch_split(cmd: ScriptCommand, ctx: &mut CommandContext) -> bool {
    match cmd {
        ScriptCommand::SplitVertical { file } => {
            let req = SplitRequest { vertical: true, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitHorizontal { file } => {
            let req = SplitRequest { vertical: false, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitClose => ctx.sink.push_command(CM_SPLIT_CLOSE, None),
        ScriptCommand::SplitFocus => ctx.sink.push_command(CM_SPLIT_FOCUS, None),
        ScriptCommand::SplitOpen { path } => {
            let req = OpenFileRequest {
                path: PathBuf::from(path),
                line: None,
                col: None,
                diff: false,
            };
            ctx.sink.push_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitLinked { on } => ctx.sink.push_command(CM_SPLIT_LINKED, Some(Box::new(on))),
        ScriptCommand::DiffRevert => ctx.sink.push_command(CM_DIFF_REVERT, None),
        _ => return false,
    }
    true
}

fn parse_todo_path(s: &str) -> Vec<usize> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split('.').filter_map(|p| p.parse().ok()).collect()
}
