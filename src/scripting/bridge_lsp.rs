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
        handle_lsp_cmd(&cmds, args, &sub)
    });
}

fn handle_lsp_cmd(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "hover" => push(cmds, ScriptCommand::LspHover),
        "definition" => push(cmds, ScriptCommand::LspDefinition),
        "references" => push(cmds, ScriptCommand::LspReferences),
        "rename" => {
            let new_name = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::LspRename { new_name });
        }
        "format" => push(cmds, ScriptCommand::LspFormat),
        "start" | "restart" | "stop" => handle_lifecycle(cmds, args, sub)?,
        "timeout" | "args" | "env" => handle_config(cmds, args, sub)?,
        other => {
            // Treat unknown subcommand as "lsp-server <lang> <cmd> [args...]"
            // e.g. "lsp clangd" → configure current filetype with clangd
            // e.g. "lsp cpp clangd --flag" → configure cpp with clangd
            let mut cmd_args = vec![other.to_string()];
            cmd_args.extend(args.iter().skip(1).map(|a| a.as_str().to_string()));
            push(cmds, ScriptCommand::LspServerConfig { args: cmd_args });
        }
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_lifecycle(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<(), TclError> {
    let pattern = super::arg_str(args, 1).unwrap_or_else(|_| "*".to_string());
    match sub {
        "start" => push(cmds, ScriptCommand::LspStart { pattern }),
        "restart" => push(cmds, ScriptCommand::LspRestart { pattern }),
        "stop" => push(cmds, ScriptCommand::LspStop { pattern }),
        _ => {}
    }
    Ok(())
}

fn handle_config(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<(), TclError> {
    match sub {
        "timeout" => {
            let pattern = super::arg_str(args, 1)?;
            let secs = super::arg_str(args, 2).ok().and_then(|s| s.parse::<u64>().ok());
            push(cmds, ScriptCommand::LspTimeout { pattern, secs });
        }
        "args" => {
            let pattern = super::arg_str(args, 1)?;
            let command = super::arg_str(args, 2)?;
            push(cmds, ScriptCommand::LspArgs { pattern, command });
        }
        "env" => {
            let pattern = super::arg_str(args, 1)?;
            let key = super::arg_str(args, 2)?;
            let value = super::arg_str(args, 3)?;
            push(cmds, ScriptCommand::LspEnv { pattern, key, value });
        }
        _ => {}
    }
    Ok(())
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}
