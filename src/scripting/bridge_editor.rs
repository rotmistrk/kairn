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
            "open" | "save" | "save-all" | "close" => handle_file_ops(&cmds, args, &sub),
            "goto" | "insert" => handle_cursor_ops(&cmds, args, &sub),
            "undo" | "redo" => handle_undo_ops(&cmds, &sub),
            "search" | "clear-highlight" => handle_search_ops(&cmds, args, &sub),
            "mark" | "jump-mark" => handle_mark_ops(&cmds, args, &sub),
            "get-selection" | "replace-selection" => handle_selection_ops(&cmds, &snapshot, args, &sub),
            "get-line" | "delete-line" | "replace-word" | "diff-revert" => {
                handle_line_ops(&cmds, &snapshot, args, &sub)
            }
            "current-file" | "current-line" | "current-col" | "modified?" | "filetype" => {
                handle_query_ops(&snapshot, &sub)
            }
            "set" => handle_set_op(&cmds, args),
            other => Err(TclError::new(format!("editor: unknown subcommand '{other}'"))),
        }
    });
}

fn handle_file_ops(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "open" => {
            let path = super::arg_str(args, 1)?;
            let line = parse_flag_u32(args, "-line");
            let col = parse_flag_u32(args, "-col");
            push(cmds, ScriptCommand::OpenFile { path, line, col });
        }
        "save" => push(cmds, ScriptCommand::Save),
        "save-all" => push(cmds, ScriptCommand::SaveAll),
        "close" => push(cmds, ScriptCommand::Close),
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_cursor_ops(
    cmds: &Arc<Mutex<Vec<ScriptCommand>>>,
    args: &[TclValue],
    sub: &str,
) -> Result<TclValue, TclError> {
    match sub {
        "goto" => {
            let line = super::arg_str(args, 1)?
                .parse::<u32>()
                .map_err(|e| TclError::new(e.to_string()))?;
            let col = super::arg_opt(args, 2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            push(cmds, ScriptCommand::Goto { line, col });
        }
        "insert" => {
            let text = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::Insert { text });
        }
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_undo_ops(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "undo" => push(cmds, ScriptCommand::Undo),
        "redo" => push(cmds, ScriptCommand::Redo),
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_search_ops(
    cmds: &Arc<Mutex<Vec<ScriptCommand>>>,
    args: &[TclValue],
    sub: &str,
) -> Result<TclValue, TclError> {
    match sub {
        "search" => {
            let pattern = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::Search { pattern: Some(pattern) });
        }
        "clear-highlight" => push(cmds, ScriptCommand::Search { pattern: None }),
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_selection_ops(
    cmds: &Arc<Mutex<Vec<ScriptCommand>>>,
    snapshot: &Arc<Mutex<StateSnapshot>>,
    args: &[TclValue],
    sub: &str,
) -> Result<TclValue, TclError> {
    match sub {
        "get-selection" => {
            let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
            return Ok(TclValue::Str(snap.selection_text.clone()));
        }
        "replace-selection" => {
            let text = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::ReplaceSelection { text });
        }
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_line_ops(
    cmds: &Arc<Mutex<Vec<ScriptCommand>>>,
    snapshot: &Arc<Mutex<StateSnapshot>>,
    args: &[TclValue],
    sub: &str,
) -> Result<TclValue, TclError> {
    match sub {
        "get-line" => {
            let line = super::arg_opt(args, 1).and_then(|s| s.parse::<u32>().ok());
            if line.is_some() {
                push(cmds, ScriptCommand::GetLine { line });
            }
            let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
            return Ok(TclValue::Str(snap.current_line_text.clone()));
        }
        "delete-line" => {
            let line = super::arg_opt(args, 1).and_then(|s| s.parse::<u32>().ok());
            push(cmds, ScriptCommand::DeleteLine { line });
        }
        "replace-word" => {
            let text = super::arg_str(args, 1)?;
            push(cmds, ScriptCommand::ReplaceWord { text });
        }
        "diff-revert" => push(cmds, ScriptCommand::DiffRevert),
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn handle_query_ops(snapshot: &Arc<Mutex<StateSnapshot>>, sub: &str) -> Result<TclValue, TclError> {
    let snap = snapshot.lock().map_err(|e| TclError::new(e.to_string()))?;
    match sub {
        "current-file" => Ok(TclValue::Str(snap.context.file.clone().unwrap_or_default())),
        "current-line" => Ok(TclValue::Int(snap.context.line as i64)),
        "current-col" => Ok(TclValue::Int(snap.context.col as i64)),
        "modified?" => Ok(TclValue::Bool(snap.context.modified)),
        "filetype" => Ok(TclValue::Str(snap.context.language.clone())),
        _ => Ok(TclValue::Str(String::new())),
    }
}

fn handle_mark_ops(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    let name = super::arg_str(args, 1)?;
    let ch = name.chars().next().ok_or_else(|| TclError::new("mark name required"))?;
    if !ch.is_ascii_alphabetic() {
        return Err(TclError::new("mark name must be a-z or A-Z"));
    }
    match sub {
        "mark" => push(cmds, ScriptCommand::SetMark { name: ch }),
        "jump-mark" => push(cmds, ScriptCommand::JumpMark { name: ch }),
        _ => {}
    }
    Ok(TclValue::Str(String::new()))
}

fn push(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, cmd: ScriptCommand) {
    if let Ok(mut v) = cmds.lock() {
        v.push(cmd);
    }
}

fn handle_set_op(cmds: &Arc<Mutex<Vec<ScriptCommand>>>, args: &[TclValue]) -> Result<TclValue, TclError> {
    let option = super::arg_str(args, 1)?;
    push(cmds, ScriptCommand::EditorSet { option });
    Ok(TclValue::Str(String::new()))
}

fn parse_flag_u32(args: &[TclValue], flag: &str) -> Option<u32> {
    for (i, a) in args.iter().enumerate() {
        if a.as_str() == flag {
            return args.get(i + 1).and_then(|v| v.as_str().parse().ok());
        }
    }
    None
}
