//! Keymap namespace — bind/unbind keys.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("keymap", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "bind" => {
                let key = super::arg_str(args, 1)?;
                let command = super::arg_str(args, 2)?;
                push(&cmds, ScriptCommand::SetKeyBinding { key, command });
                Ok(TclValue::Str(String::new()))
            }
            "unbind" => {
                let key = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::UnbindKey { key });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("keymap: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
