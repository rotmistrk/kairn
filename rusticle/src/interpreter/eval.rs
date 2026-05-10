//! Script evaluation: parse, substitute, and dispatch commands.

use crate::error::{ErrorCode, TclError};
use crate::parser::Parser;
use crate::parser::Word;
use crate::value::TclValue;

use super::{Interpreter, Proc};
use super::eval_literals::{eval_dict_literal, eval_list_literal};

pub use super::subst::substitute;

/// Evaluate a script string in the interpreter.
pub fn eval_script(interp: &mut Interpreter, script: &str) -> Result<TclValue, TclError> {
    let parsed = Parser::parse(script)?;
    let mut result = TclValue::Str(String::new());
    for cmd in &parsed.commands {
        if cmd.words.is_empty() {
            continue;
        }
        let args = resolve_words(interp, &cmd.words)?;
        if args.is_empty() {
            continue;
        }
        // Handle null coalescing: `value ?? default`
        if args.len() == 3 && args[1].as_str() == "??" {
            let val = &args[0];
            result = if val.is_empty() {
                args[2].clone()
            } else {
                val.clone()
            };
            continue;
        }
        result = dispatch(interp, &args)?;
    }
    Ok(result)
}

/// Evaluate a script, catching top-level `return` and extracting the value.
/// Used by command substitution `[...]` and the public `eval()` API.
pub fn eval_script_catching_return(interp: &mut Interpreter, script: &str) -> Result<TclValue, TclError> {
    match eval_script(interp, script) {
        Ok(v) => Ok(v),
        Err(e) if matches!(e.code, ErrorCode::Return(_)) => {
            if let ErrorCode::Return(v) = e.code {
                Ok(v)
            } else {
                Ok(TclValue::Str(String::new()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Resolve all words in a command to TclValues.
fn resolve_words(interp: &mut Interpreter, words: &[Word]) -> Result<Vec<TclValue>, TclError> {
    let mut result = Vec::with_capacity(words.len());
    for word in words {
        result.push(resolve_word(interp, word)?);
    }
    Ok(result)
}

/// Resolve a single word to a TclValue.
fn resolve_word(interp: &mut Interpreter, word: &Word) -> Result<TclValue, TclError> {
    match word {
        Word::Literal(s) => Ok(TclValue::Str(s.clone())),
        Word::Braced(s) => Ok(TclValue::Str(s.clone())),
        Word::Quoted(s) => substitute(interp, s),
        Word::Bare(s) => substitute(interp, s),
        Word::DictLiteral(s) => eval_dict_literal(interp, s),
        Word::ListLiteral(s) => eval_list_literal(interp, s),
        Word::Heredoc(s) => substitute(interp, s),
        Word::HeredocRaw(s) => Ok(TclValue::Str(s.clone())),
    }
}

/// Dispatch a resolved command.
pub fn dispatch(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    let name = args[0].as_str().to_string();
    let cmd_args = &args[1..];

    // Check for proc first
    if let Some(proc_def) = interp.procs.get(&name).cloned() {
        return call_proc(interp, &proc_def, cmd_args);
    }

    // Check registered commands — clone the Rc so the command stays in the map
    // during execution (allows recursive/nested calls to the same command)
    if let Some(cmd) = interp.commands.get(&name).cloned() {
        return cmd.call(interp, cmd_args);
    }

    // If it's a single word (no arguments) and not a known command,
    // return it as a value. This supports pipe chains like `$x(0) | cmd`.
    if cmd_args.is_empty() {
        return Ok(args[0].clone());
    }

    Err(TclError::new(format!("invalid command name \"{name}\"")))
}

/// Call a procedure.
fn call_proc(interp: &mut Interpreter, proc_def: &Proc, args: &[TclValue]) -> Result<TclValue, TclError> {
    // Check arity
    let min_args = proc_def.params.iter().filter(|p| p.default.is_none()).count();
    let max_args = proc_def.params.len();
    if args.len() < min_args || args.len() > max_args {
        return Err(TclError::new(format!(
            "wrong # args: expected {min_args}..{max_args}, got {}",
            args.len()
        )));
    }
    interp.push_scope_linked(proc_def.defining_scope);
    // Bind parameters
    for (i, param) in proc_def.params.iter().enumerate() {
        let val = if i < args.len() {
            args[i].clone()
        } else if let Some(default) = &param.default {
            default.clone()
        } else {
            TclValue::Str(String::new())
        };
        interp.set_var(&param.name, val);
    }
    let result = eval_script(interp, &proc_def.body);
    interp.pop_scope();
    match result {
        Ok(v) => Ok(v),
        Err(e) if matches!(e.code, ErrorCode::Return(_)) => {
            if let ErrorCode::Return(v) = e.code {
                Ok(v)
            } else {
                Ok(TclValue::Str(String::new()))
            }
        }
        Err(e) => Err(e),
    }
}
