//! Todo namespace — add, remove, complete.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("todo", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "add" => {
                let text = super::arg_str(args, 1)?;
                let parent = parse_flag_str(args, "-parent");
                push(&cmds, ScriptCommand::TodoAdd { text, parent });
                Ok(TclValue::Str(String::new()))
            }
            "remove" => {
                let path = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::TodoRemove { path });
                Ok(TclValue::Str(String::new()))
            }
            "complete" => {
                let path = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::TodoComplete { path });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("todo: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}

fn parse_flag_str(args: &[TclValue], flag: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a.as_str() == flag {
            return args.get(i + 1).map(|v| v.as_str().into_owned());
        }
    }
    None
}
