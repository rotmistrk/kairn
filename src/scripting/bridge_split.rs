//! Split namespace — split pane manipulation.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::{ScriptCommand, StateSnapshot};

pub fn register(
    interp: &mut Interpreter,
    commands: Arc<Mutex<Vec<ScriptCommand>>>,
    snapshot: Arc<Mutex<StateSnapshot>>,
) {
    let cmds = commands;
    let snap = snapshot;
    interp.register_fn("split", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        handle_split_cmd(&cmds, &snap, args, &sub)
    });
}

fn handle_split_cmd(
    cmds: &Arc<Mutex<Vec<ScriptCommand>>>,
    snap: &Arc<Mutex<StateSnapshot>>,
    args: &[TclValue],
    sub: &str,
) -> Result<TclValue, TclError> {
    match sub {
        "vsplit" | "vertical" => {
            let file = super::arg_opt(args, 1);
            push(cmds, ScriptCommand::SplitVertical { file });
        }
        "hsplit" | "horizontal" => {
            let file = super::arg_opt(args, 1);
            push(cmds, ScriptCommand::SplitHorizontal { file });
        }
        "close" | "only" => push(cmds, ScriptCommand::SplitClose),
        "focus" => push(cmds, ScriptCommand::SplitFocus),
        "open" => {
            let path = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::SplitOpen { path });
        }
        "direction" => {
            let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
            return Ok(TclValue::Str(s.split_direction.clone()));
        }
        "linked" => {
            if let Some(val) = super::arg_opt(args, 1) {
                let on = val == "true" || val == "1";
                push(cmds, ScriptCommand::SplitLinked { on });
            } else {
                let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
                return Ok(TclValue::Str(s.split_linked.to_string()));
            }
        }
        other => return Err(TclError::new(format!("split: unknown subcommand '{other}'"))),
    }
    Ok(TclValue::Str(String::new()))
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
