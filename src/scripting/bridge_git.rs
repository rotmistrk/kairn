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
        match sub.as_str() {
            "stage" => {
                let file = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::GitStage { file });
                Ok(TclValue::Str(String::new()))
            }
            "unstage" => {
                let file = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::GitUnstage { file });
                Ok(TclValue::Str(String::new()))
            }
            "commit" => {
                let message = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::GitCommit { message });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("git: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
