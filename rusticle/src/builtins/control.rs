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

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn if_true() {
        let mut interp = Interpreter::new();
        interp.eval("set result no").unwrap();
        interp.eval("if {1} {set result yes}").unwrap();
        assert_eq!(interp.eval("set result").unwrap().as_str(), "yes");
    }

    #[test]
    fn if_false_else() {
        let mut interp = Interpreter::new();
        interp.eval("set result no").unwrap();
        interp.eval("if {0} {set result yes} else {set result no}").unwrap();
        assert_eq!(interp.eval("set result").unwrap().as_str(), "no");
    }

    #[test]
    fn while_loop() {
        let mut interp = Interpreter::new();
        interp.eval("set x 0").unwrap();
        interp.eval("while {$x < 3} {incr x}").unwrap();
        assert_eq!(interp.eval("set x").unwrap().as_str(), "3");
    }

    #[test]
    fn foreach_loop() {
        let mut interp = Interpreter::new();
        interp.eval("set sum 0").unwrap();
        interp.eval("foreach i [list 1 2 3] {incr sum $i}").unwrap();
        assert_eq!(interp.eval("set sum").unwrap().as_str(), "6");
    }

    #[test]
    fn for_loop() {
        let mut interp = Interpreter::new();
        interp.eval("set sum 0").unwrap();
        interp.eval("for {set i 0} {$i < 5} {incr i} {incr sum $i}").unwrap();
        assert_eq!(interp.eval("set sum").unwrap().as_str(), "10");
    }

    #[test]
    fn break_in_while() {
        let mut interp = Interpreter::new();
        interp.eval("set x 0").unwrap();
        interp.eval("while {1} {incr x; if {$x == 3} {break}}").unwrap();
        assert_eq!(interp.eval("set x").unwrap().as_str(), "3");
    }

    #[test]
    fn return_value() {
        let mut interp = Interpreter::new();
        interp.eval("proc foo {} {return 42}").unwrap();
        assert_eq!(interp.eval("foo").unwrap().as_str(), "42");
    }

    #[test]
    fn proc_basic() {
        let mut interp = Interpreter::new();
        interp.eval("proc double {x} {return [expr {$x * 2}]}").unwrap();
        let result = interp.eval("double 5").unwrap();
        assert_eq!(result.as_str(), "10");
    }

    #[test]
    fn proc_default_arg() {
        let mut interp = Interpreter::new();
        interp
            .eval("proc greet {{name world}} {return \"hello $name\"}")
            .unwrap();
        assert_eq!(interp.eval("greet").unwrap().as_str(), "hello world");
        assert_eq!(interp.eval("greet rust").unwrap().as_str(), "hello rust");
    }

    #[test]
    fn switch_basic() {
        let mut interp = Interpreter::new();
        interp.eval("set result none").unwrap();
        interp
            .eval("switch hello {hello {set result hi} world {set result bye}}")
            .unwrap();
        assert_eq!(interp.eval("set result").unwrap().as_str(), "hi");
    }

    #[test]
    fn match_literal() {
        let mut interp = Interpreter::new();
        interp.eval("set result unknown").unwrap();
        interp
            .eval(r#"match ok {"ok" {set result success} _ {set result unknown}}"#)
            .unwrap();
        assert_eq!(interp.eval("set result").unwrap().as_str(), "success");
    }

    #[test]
    fn try_catch_finally() {
        let mut interp = Interpreter::new();
        interp
            .eval(
                r#"set log ""
try {
    error "boom"
} on error {msg} {
    append log "caught:$msg"
} finally {
    append log ",cleaned"
}"#,
            )
            .unwrap();
        assert_eq!(interp.eval("set log").unwrap().as_str(), "caught:boom,cleaned");
    }

    #[test]
    fn catch_returns_code() {
        let mut interp = Interpreter::new();
        let result = interp.eval("catch {error oops} msg").unwrap();
        assert_eq!(result.as_str(), "1");
        assert_eq!(interp.eval("set msg").unwrap().as_str(), "oops");
    }

    #[test]
    fn lexical_scoping() {
        let mut interp = Interpreter::new();
        interp.eval("set x 10").unwrap();
        interp.eval("proc foo {} { return $x }").unwrap();
        let result = interp.eval("foo").unwrap();
        assert_eq!(result.as_str(), "10");
    }
}
