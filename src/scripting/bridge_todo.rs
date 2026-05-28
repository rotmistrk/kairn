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
        handle_todo_cmd(&cmds, args, &sub)
    });
}

fn handle_todo_cmd(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "add" => {
            let text = super::arg_str(args, 1)?;
            let parent = parse_flag_str(args, "-parent");
            push(cmds, ScriptCommand::TodoAdd { text, parent });
        }
        "remove" | "complete" | "toggle-important" | "promote" | "demote" => {
            handle_path_cmd(cmds, args, sub)?;
        }
        "edit" => {
            let path = super::arg_str(args, 1)?;
            let text = super::arg_str(args, 2)?;
            push(cmds, ScriptCommand::TodoEdit { path, text });
        }
        "swap" => {
            let path = super::arg_str(args, 1)?;
            let direction = super::arg_str(args, 2)?;
            push(cmds, ScriptCommand::TodoSwap { path, direction });
        }
        "list" => push(cmds, ScriptCommand::TodoList),
        other => return Err(TclError::new(format!("todo: unknown subcommand '{other}'"))),
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_path_cmd(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<(), TclError> {
    let path = super::arg_str(args, 1)?;
    match sub {
        "remove" => push(cmds, ScriptCommand::TodoRemove { path }),
        "complete" => push(cmds, ScriptCommand::TodoComplete { path }),
        "toggle-important" => push(cmds, ScriptCommand::TodoToggleImportant { path }),
        "promote" => push(cmds, ScriptCommand::TodoPromote { path }),
        "demote" => push(cmds, ScriptCommand::TodoDemote { path }),
        _ => {}
    }
    Ok(())
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
