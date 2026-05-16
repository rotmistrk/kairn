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
            "start" => {
                let pattern = super::arg_str(args, 1).unwrap_or_else(|_| "*".to_string());
                push(&cmds, ScriptCommand::LspStart { pattern });
                Ok(TclValue::Str(String::new()))
            }
            "restart" => {
                let pattern = super::arg_str(args, 1).unwrap_or_else(|_| "*".to_string());
                push(&cmds, ScriptCommand::LspRestart { pattern });
                Ok(TclValue::Str(String::new()))
            }
            "stop" => {
                let pattern = super::arg_str(args, 1).unwrap_or_else(|_| "*".to_string());
                push(&cmds, ScriptCommand::LspStop { pattern });
                Ok(TclValue::Str(String::new()))
            }
            "timeout" => {
                let pattern = super::arg_str(args, 1)?;
                let secs = super::arg_str(args, 2).ok().and_then(|s| s.parse::<u64>().ok());
                push(&cmds, ScriptCommand::LspTimeout { pattern, secs });
                Ok(TclValue::Str(String::new()))
            }
            "args" => {
                let pattern = super::arg_str(args, 1)?;
                let command = super::arg_str(args, 2)?;
                push(&cmds, ScriptCommand::LspArgs { pattern, command });
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
