//! Editor namespace — file/cursor/buffer operations.

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
    let cmds = commands.clone();
    interp.register_fn("editor", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "open" => {
                let path = super::arg_str(args, 1)?;
                let line = parse_flag_u32(args, "-line");
                let col = parse_flag_u32(args, "-col");
                push(&cmds, ScriptCommand::OpenFile { path, line, col });
                Ok(TclValue::Str(String::new()))
            }
            "save" => {
                push(&cmds, ScriptCommand::Save);
                Ok(TclValue::Str(String::new()))
            }
            "save-all" => {
                push(&cmds, ScriptCommand::SaveAll);
                Ok(TclValue::Str(String::new()))
            }
            "close" => {
                push(&cmds, ScriptCommand::Close);
                Ok(TclValue::Str(String::new()))
            }
            "goto" => {
                let line = super::arg_str(args, 1)?
                    .parse::<u32>()
                    .map_err(|e| TclError::new(e.to_string()))?;
                let col = super::arg_opt(args, 2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
                push(&cmds, ScriptCommand::Goto { line, col });
                Ok(TclValue::Str(String::new()))
            }
            "insert" => {
                let text = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::Insert { text });
                Ok(TclValue::Str(String::new()))
            }
            "undo" => {
                push(&cmds, ScriptCommand::Undo);
                Ok(TclValue::Str(String::new()))
            }
            "redo" => {
                push(&cmds, ScriptCommand::Redo);
                Ok(TclValue::Str(String::new()))
            }
            "current-file" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                let f = snap.context.file.clone().unwrap_or_default();
                Ok(TclValue::Str(f))
            }
            "current-line" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Int(snap.context.line as i64))
            }
            "current-col" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Int(snap.context.col as i64))
            }
            "modified?" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Bool(snap.context.modified))
            }
            "filetype" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Str(snap.context.language.clone()))
            }
            "get-selection" => {
                let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Str(snap.selection_text.clone()))
            }
            "replace-selection" => {
                let text = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::ReplaceSelection { text });
                Ok(TclValue::Str(String::new()))
            }
            "get-line" => {
                let line = super::arg_opt(args, 1).and_then(|s| s.parse::<u32>().ok());
                if line.is_some() {
                    push(&cmds, ScriptCommand::GetLine { line });
                    // For explicit line, we'd need synchronous access; return from snapshot
                    let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                    Ok(TclValue::Str(snap.current_line_text.clone()))
                } else {
                    let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
                    Ok(TclValue::Str(snap.current_line_text.clone()))
                }
            }
            "delete-line" => {
                let line = super::arg_opt(args, 1).and_then(|s| s.parse::<u32>().ok());
                push(&cmds, ScriptCommand::DeleteLine { line });
                Ok(TclValue::Str(String::new()))
            }
            "replace-word" => {
                let text = super::arg_str(args, 1)?;
                push(&cmds, ScriptCommand::ReplaceWord { text });
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("editor: unknown subcommand '{other}'"))),
        }
    });
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}

fn parse_flag_u32(args: &[TclValue], flag: &str) -> Option<u32> {
    for (i, a) in args.iter().enumerate() {
        if a.as_str() == flag {
            return args.get(i + 1).and_then(|v| v.as_str().parse().ok());
        }
    }
    None
}
