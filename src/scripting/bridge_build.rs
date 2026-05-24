//! Build namespace — run, test, errors.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands.clone();
    interp.register_fn("build", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "run" => {
                let command = super::arg_opt(args, 1);
                push(&cmds, ScriptCommand::RunBuild { command });
                Ok(TclValue::Str(String::new()))
            }
            "test" => {
                let command = super::arg_opt(args, 1);
                push(&cmds, ScriptCommand::RunTest { command });
                Ok(TclValue::Str(String::new()))
            }
            "test-file" => {
                push(&cmds, ScriptCommand::TestFile);
                Ok(TclValue::Str(String::new()))
            }
            "test-at-cursor" => {
                push(&cmds, ScriptCommand::TestAtCursor);
                Ok(TclValue::Str(String::new()))
            }
            "next-error" => {
                push(&cmds, ScriptCommand::NextError);
                Ok(TclValue::Str(String::new()))
            }
            "prev-error" => {
                push(&cmds, ScriptCommand::PrevError);
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("build: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}

pub fn register_grep(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("grep", move |_interp, args| {
        let pattern = super::arg_str(args, 0)?;
        push(&cmds, ScriptCommand::Grep { pattern });
        Ok(TclValue::Str(String::new()))
    });
}
