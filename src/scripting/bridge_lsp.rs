//! LSP namespace — hover, definition, references, rename, format.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::ScriptCommand;

pub fn register(interp: &mut Interpreter, commands: Arc<Mutex<Vec<ScriptCommand>>>) {
    let cmds = commands;
    interp.register_fn("lsp", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "hover" => {
                push(&cmds, ScriptCommand::LspHover);
                Ok(TclValue::Str(String::new()))
            }
            "definition" => {
                push(&cmds, ScriptCommand::LspDefinition);
                Ok(TclValue::Str(String::new()))
            }
            "references" => {
                push(&cmds, ScriptCommand::LspReferences);
                Ok(TclValue::Str(String::new()))
            }
            "rename" => {
                let new_name = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::LspRename { new_name });
                Ok(TclValue::Str(String::new()))
            }
            "format" => {
                push(&cmds, ScriptCommand::LspFormat);
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("lsp: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
