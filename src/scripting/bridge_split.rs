//! Split namespace — split pane manipulation.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("split", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "vsplit" | "vertical" => {
                let file = super::arg_opt(args, 1);
                push(&cmds, ScriptCommand::SplitVertical { file });
                Ok(TclValue::Str(String::new()))
            }
            "hsplit" | "horizontal" => {
                let file = super::arg_opt(args, 1);
                push(&cmds, ScriptCommand::SplitHorizontal { file });
                Ok(TclValue::Str(String::new()))
            }
            "close" | "only" => {
                push(&cmds, ScriptCommand::SplitClose);
                Ok(TclValue::Str(String::new()))
            }
            "focus" => {
                push(&cmds, ScriptCommand::SplitFocus);
                Ok(TclValue::Str(String::new()))
            }
            "open" => {
                let path = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::SplitOpen { path });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("split: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
