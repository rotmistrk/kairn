//! Git namespace — branch, stage, unstage, commit.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("git", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        handle_git_cmd(&cmds, args, &sub)
    });
}

fn handle_git_cmd(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "stage" => {
            let file = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::GitStage { file });
        }
        "unstage" => {
            let file = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::GitUnstage { file });
        }
        "commit" => {
            let message = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::GitCommit { message });
        }
        "blame" => push(cmds, ScriptCommand::GitBlame),
        "noblame" => push(cmds, ScriptCommand::GitNoBlame),
        "untrack" => {
            let file = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::GitUntrack { file });
        }
        "log" => push(cmds, ScriptCommand::GitLog),
        "diff" => push(cmds, ScriptCommand::GitDiff),
        "base" => {
            let base = args.get(2).map(|v| v.to_string()).filter(|s| !s.is_empty());
            let root = args.get(3).map(|v| v.to_string()).filter(|s| !s.is_empty());
            push(cmds, ScriptCommand::GitSetBase { base, root });
        }
        other => return Err(TclError::new(format!("git: unknown subcommand '{other}'"))),
    }
    Ok(TclValue::Str(String::new()))
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
