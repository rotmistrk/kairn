//! View namespace — focus, messages, status.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("view", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "focus" => {
                let slot = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::FocusSlot { slot });
                Ok(TclValue::Str(String::new()))
            }
            "message" => {
                let level = super::arg_str(args, 1)?;
                let origin = super::arg_str(args, 2)?;
                let text = super::arg_str(args, 3)?;
                push(&cmds, ScriptCommand::ShowMessage { level, origin, text });
                Ok(TclValue::Str(String::new()))
            }
            "status" => {
                let text = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::StatusFlash { text });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("view: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
