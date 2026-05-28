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
        handle_view_cmd(&cmds, args, &sub)
    });
}

fn handle_view_cmd(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "focus" => {
            let slot = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::FocusSlot { slot });
        }
        "message" => {
            let level = super::arg_str(args, 1)?;
            let origin = super::arg_str(args, 2)?;
            let text = super::arg_str(args, 3)?;
            push(cmds, ScriptCommand::ShowMessage { level, origin, text });
        }
        "status" => {
            let text = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::StatusFlash { text });
        }
        "theme" => {
            let mode = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::ViewTheme { mode });
        }
        "zoom" => push(cmds, ScriptCommand::ViewZoom),
        "toggle-tree" => push(cmds, ScriptCommand::ViewToggleTree),
        "toggle-tools" => push(cmds, ScriptCommand::ViewToggleTools),
        "layout" => push(cmds, ScriptCommand::ViewLayout),
        other => return Err(TclError::new(format!("view: unknown subcommand '{other}'"))),
    }
    Ok(TclValue::Str(String::new()))
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
