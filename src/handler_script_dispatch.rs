//! Script command dispatch — git, todo, split, lsp, grep variants.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::commands::*;
use crate::handler_script_util::lsp_cmd;
use crate::scripting::ScriptCommand;

/// Dispatch git/todo/split/lsp/grep script commands.
/// Returns true if the command was handled, false otherwise.
pub(crate) fn dispatch_extended(cmd: ScriptCommand, ctx: &mut CommandContext, state: &mut AppState) -> bool {
    match cmd {
        ScriptCommand::GitStage { file } => {
            ctx.sink.push_command(CM_GIT_STAGE, Some(Box::new(file)));
        }
        ScriptCommand::GitUnstage { file } => {
            ctx.sink.push_command(CM_GIT_UNSTAGE, Some(Box::new(file)));
        }
        ScriptCommand::GitCommit { message } => {
            ctx.sink.push_command(CM_GIT_COMMIT, Some(Box::new(message)));
        }
        ScriptCommand::GitBlame => {
            ctx.sink.push_command(CM_BLAME, None);
        }
        ScriptCommand::GitNoBlame => {
            ctx.sink.push_command(CM_NOBLAME, None);
        }
        ScriptCommand::GitUntrack { file } => {
            ctx.sink.push_command(CM_GIT_UNTRACK, Some(Box::new(file)));
        }
        ScriptCommand::GitLog => {
            ctx.sink.push_command(CM_GIT_LOG, None);
        }
        ScriptCommand::GitDiff => {
            ctx.sink.push_command(CM_DIFF, None);
        }
        ScriptCommand::TodoAdd { text, parent } => {
            let path = parse_todo_path(&parent.unwrap_or_default());
            let action = crate::mcp::commands::McpAction::TodoAdd { path, title: text };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoRemove { path } => {
            let action = crate::mcp::commands::McpAction::TodoRemove {
                path: parse_todo_path(&path),
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoComplete { path } => {
            let action = crate::mcp::commands::McpAction::TodoToggle {
                path: parse_todo_path(&path),
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoToggleImportant { path } => {
            let action = crate::mcp::commands::McpAction::TodoToggleImportant {
                path: parse_todo_path(&path),
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoEdit { path, text } => {
            let action = crate::mcp::commands::McpAction::TodoEdit {
                path: parse_todo_path(&path),
                title: text,
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoSwap { path, direction } => {
            let p = parse_todo_path(&path);
            let action = if direction == "up" {
                crate::mcp::commands::McpAction::TodoMoveUp { path: p }
            } else {
                crate::mcp::commands::McpAction::TodoMoveDown { path: p }
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoPromote { path } => {
            let action = crate::mcp::commands::McpAction::TodoPromote {
                path: parse_todo_path(&path),
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoDemote { path } => {
            let action = crate::mcp::commands::McpAction::TodoDemote {
                path: parse_todo_path(&path),
            };
            ctx.sink.push_command(CM_TODO_ACTION, Some(Box::new(action)));
        }
        ScriptCommand::TodoList => {}
        ScriptCommand::SplitVertical { file } => {
            let req = crate::commands::SplitRequest { vertical: true, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitHorizontal { file } => {
            let req = crate::commands::SplitRequest { vertical: false, file };
            ctx.sink.push_command(CM_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitClose => {
            ctx.sink.push_command(CM_SPLIT_CLOSE, None);
        }
        ScriptCommand::SplitFocus => {
            ctx.sink.push_command(CM_SPLIT_FOCUS, None);
        }
        ScriptCommand::SplitOpen { path } => {
            let req = crate::commands::OpenFileRequest {
                path: std::path::PathBuf::from(path),
                line: None,
                col: None,
                diff: false,
            };
            ctx.sink.push_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
        }
        ScriptCommand::SplitLinked { on } => {
            ctx.sink.push_command(CM_SPLIT_LINKED, Some(Box::new(on)));
        }
        ScriptCommand::DiffRevert => {
            ctx.sink.push_command(CM_DIFF_REVERT, None);
        }
        ScriptCommand::Grep { pattern } => {
            let root = state.root_dir.clone();
            let waker = state.waker.clone().unwrap_or_else(txv_core::run::Waker::noop);
            let grep_state = crate::grep::grep_async(&pattern, &root, waker);
            state.grep_pending = Some((format!("grep:{pattern}"), grep_state, root));
        }
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

fn parse_todo_path(s: &str) -> Vec<usize> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split('.').filter_map(|p| p.parse().ok()).collect()
}
