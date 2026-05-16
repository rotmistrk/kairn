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
            "direction" => {
                let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Str(s.split_direction.clone()))
            }
            "linked" => {
                if let Some(val) = super::arg_opt(args, 1) {
                    let on = val == "true" || val == "1";
                    push(&cmds, ScriptCommand::SplitLinked { on });
                    Ok(TclValue::Str(String::new()))
                } else {
                    let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
                    Ok(TclValue::Str(s.split_linked.to_string()))
                }
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
