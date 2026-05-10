//! Control flow commands: if, while, foreach, for, break, continue, return.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register control flow commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("if", cmd_if);
    interp.register_fn("while", cmd_while);
    interp.register_fn("foreach", cmd_foreach);
    interp.register_fn("for", cmd_for);
    interp.register_fn("break", cmd_break);
    interp.register_fn("continue", cmd_continue);
    interp.register_fn("return", cmd_return);
}

/// `if {cond} {body} ?elseif {cond} {body}? ?else {body}?`
fn cmd_if(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    let mut i = 0;
    while i < args.len() {
        if i > 0 {
            let keyword = args[i].as_str();
            if keyword == "else" {
                i += 1;
                if i < args.len() {
                    return interp.eval(&args[i].as_str());
                }
                return Ok(TclValue::Str(String::new()));
            }
            if keyword == "elseif" {
                i += 1;
            } else {
                return Err(TclError::new(format!(
                    "expected \"elseif\" or \"else\", got \"{keyword}\""
                )));
            }
        }
        if i >= args.len() {
            break;
        }
        let cond = eval_condition(interp, &args[i].as_str())?;
        i += 1;
        if cond {
            if i < args.len() {
                return interp.eval(&args[i].as_str());
            }
            return Ok(TclValue::Str(String::new()));
        }
        i += 1; // skip body
    }
    Ok(TclValue::Str(String::new()))
}

/// `while {cond} {body}`
fn cmd_while(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 2 {
        return Err(TclError::new("wrong # args: should be \"while test command\""));
    }
    let cond_str = args[0].as_str().to_string();
    let body = args[1].as_str().to_string();
    loop {
        if !eval_condition(interp, &cond_str)? {
            break;
        }
        match interp.eval(&body) {
            Ok(_) => {}
            Err(e) if e.code == ErrorCode::Break => break,
            Err(e) if e.code == ErrorCode::Continue => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(TclValue::Str(String::new()))
}

/// `foreach var list body`
fn cmd_foreach(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 3 {
        return Err(TclError::new("wrong # args: should be \"foreach varName list body\""));
    }
    let var_pattern = args[0].as_str().to_string();
    let list = args[1].as_list()?;
    let body = args[2].as_str().to_string();

    // Check for destructuring: {key, value}
    let vars = parse_foreach_vars(&var_pattern);
    let step = vars.len();

    let mut i = 0;
    while i < list.len() {
        for (j, var) in vars.iter().enumerate() {
            let val = list.get(i + j).cloned().unwrap_or(TclValue::Str(String::new()));
            interp.set_var(var, val);
        }
        i += step;
        match interp.eval(&body) {
            Ok(_) => {}
            Err(e) if e.code == ErrorCode::Break => break,
            Err(e) if e.code == ErrorCode::Continue => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(TclValue::Str(String::new()))
}

/// Parse foreach variable pattern (single var or {a, b} destructuring).
fn parse_foreach_vars(pattern: &str) -> Vec<String> {
    let trimmed = pattern.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        inner.split(',').map(|s| s.trim().to_string()).collect()
    } else if trimmed.contains(',') {
        trimmed.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Could be a Tcl list of var names
        trimmed.split_whitespace().map(|s| s.to_string()).collect()
    }
}

/// `for {init} {cond} {step} {body}`
fn cmd_for(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 4 {
        return Err(TclError::new("wrong # args: should be \"for start test next command\""));
    }
    let init = args[0].as_str().to_string();
    let cond = args[1].as_str().to_string();
    let step = args[2].as_str().to_string();
    let body = args[3].as_str().to_string();

    interp.eval(&init)?;
    loop {
        if !eval_condition(interp, &cond)? {
            break;
        }
        match interp.eval(&body) {
            Ok(_) => {}
            Err(e) if e.code == ErrorCode::Break => break,
            Err(e) if e.code == ErrorCode::Continue => {}
            Err(e) => return Err(e),
        }
        interp.eval(&step)?;
    }
    Ok(TclValue::Str(String::new()))
}

/// `break`
fn cmd_break(_interp: &mut Interpreter, _args: &[TclValue]) -> Result<TclValue, TclError> {
    Err(TclError::with_code("break", ErrorCode::Break))
}

/// `continue`
fn cmd_continue(_interp: &mut Interpreter, _args: &[TclValue]) -> Result<TclValue, TclError> {
    Err(TclError::with_code("continue", ErrorCode::Continue))
}

/// `return ?value?`
fn cmd_return(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    let val = args.first().cloned().unwrap_or(TclValue::Str(String::new()));
    Err(TclError::with_code("return", ErrorCode::Return(val)))
}

/// Evaluate a condition string as a boolean.
fn eval_condition(interp: &mut Interpreter, cond: &str) -> Result<bool, TclError> {
    let result = interp.eval(&format!("expr {{{cond}}}"))?;
    result.as_bool()
}
