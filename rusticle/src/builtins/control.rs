//! Control flow commands: if, while, foreach, for, break, continue, return, proc, switch, match, try.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::{Interpreter, Proc, ProcParam};
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
    interp.register_fn("proc", cmd_proc);
    interp.register_fn("switch", cmd_switch);
    interp.register_fn("match", cmd_match);
    interp.register_fn("try", cmd_try);
    interp.register_fn("catch", cmd_catch);
    interp.register_fn("error", cmd_error);
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
        return Err(TclError::new(
            "wrong # args: should be \"while test command\"",
        ));
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
        return Err(TclError::new(
            "wrong # args: should be \"foreach varName list body\"",
        ));
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
            let val = list
                .get(i + j)
                .cloned()
                .unwrap_or(TclValue::Str(String::new()));
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
        return Err(TclError::new(
            "wrong # args: should be \"for start test next command\"",
        ));
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
    let val = args
        .first()
        .cloned()
        .unwrap_or(TclValue::Str(String::new()));
    Err(TclError::with_code("return", ErrorCode::Return(val)))
}

/// `proc name args body`
fn cmd_proc(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 3 {
        return Err(TclError::new(
            "wrong # args: should be \"proc name args body\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let params = parse_proc_params(&args[1].as_str())?;
    let body = args[2].as_str().to_string();
    let defining_scope = interp.scopes.len() - 1;
    interp.define_proc(
        name,
        Proc {
            params,
            body,
            defining_scope,
        },
    );
    Ok(TclValue::Str(String::new()))
}

/// Parse proc parameter list.
fn parse_proc_params(s: &str) -> Result<Vec<ProcParam>, TclError> {
    let mut params = Vec::new();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(params);
    }
    // Parse as a Tcl list — each element is either "name" or "{name default}"
    let elements = parse_param_list(trimmed);
    for elem in elements {
        let elem = elem.trim().to_string();
        if elem.starts_with('{') && elem.ends_with('}') {
            let inner = &elem[1..elem.len() - 1];
            let parts: Vec<&str> = inner.splitn(2, ' ').collect();
            let name = strip_type_annotation(parts[0]);
            let default = parts.get(1).map(|s| TclValue::Str(s.trim().to_string()));
            params.push(ProcParam { name, default });
        } else {
            let name = strip_type_annotation(&elem);
            params.push(ProcParam {
                name,
                default: None,
            });
        }
    }
    Ok(params)
}

/// Strip type annotation from parameter name (e.g., "line:int" → "line").
fn strip_type_annotation(s: &str) -> String {
    s.split(':').next().unwrap_or(s).to_string()
}

/// Parse a simple parameter list (space-separated, respecting braces).
fn parse_param_list(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        if chars[i] == '{' {
            let start = i;
            let mut depth = 1;
            i += 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                }
                i += 1;
            }
            result.push(chars[start..i].iter().collect());
        } else {
            let start = i;
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            result.push(chars[start..i].iter().collect());
        }
    }
    result
}

/// `switch value {pattern body ...}`
fn cmd_switch(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 2 {
        return Err(TclError::new(
            "wrong # args: should be \"switch value {pattern body ...}\"",
        ));
    }
    let value = args[0].as_str().to_string();
    let body = args[1].as_str().to_string();
    let cases = parse_switch_cases(&body)?;
    for (pattern, script) in &cases {
        if pattern == &value || pattern == "default" || pattern == "_" {
            return interp.eval(script);
        }
    }
    Ok(TclValue::Str(String::new()))
}

/// `match value { pattern ?var? body ... }`
fn cmd_match(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 2 {
        return Err(TclError::new(
            "wrong # args: should be \"match value {cases}\"",
        ));
    }
    let value = args[0].clone();
    let body = args[1].as_str().to_string();
    let cases = parse_match_cases(&body)?;
    for case in &cases {
        if match_pattern(interp, &value, case)? {
            return interp.eval(&case.body);
        }
    }
    Ok(TclValue::Str(String::new()))
}

/// A match case.
struct MatchCase {
    pattern: String,
    binding: Option<String>,
    body: String,
}

/// Parse match cases from the body.
fn parse_match_cases(body: &str) -> Result<Vec<MatchCase>, TclError> {
    let words = parse_param_list(body);
    let mut cases = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let pattern = words[i].trim_matches(|c| c == '{' || c == '}').to_string();
        i += 1;
        if i >= words.len() {
            return Err(TclError::new("missing body in match"));
        }
        // Check if next word is a body (starts with {) or a binding variable
        let next = &words[i];
        if next.starts_with('{') {
            let body_str = next[1..next.len().saturating_sub(1)].to_string();
            cases.push(MatchCase {
                pattern,
                binding: None,
                body: body_str,
            });
            i += 1;
        } else {
            // It's a binding variable
            let binding = Some(next.clone());
            i += 1;
            if i >= words.len() {
                return Err(TclError::new("missing body in match"));
            }
            let body_str = words[i]
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            cases.push(MatchCase {
                pattern,
                binding,
                body: body_str,
            });
            i += 1;
        }
    }
    Ok(cases)
}

/// Check if a value matches a pattern and bind variables.
fn match_pattern(
    interp: &mut Interpreter,
    value: &TclValue,
    case: &MatchCase,
) -> Result<bool, TclError> {
    let pat = &case.pattern;
    // Wildcard
    if pat == "_" {
        return Ok(true);
    }
    // Type patterns
    let type_match = match pat.as_str() {
        "int" => matches!(value, TclValue::Int(_)) || value.as_int().is_ok(),
        "string" => matches!(value, TclValue::Str(_)),
        "float" => matches!(value, TclValue::Float(_)),
        "bool" => matches!(value, TclValue::Bool(_)),
        "list" => matches!(value, TclValue::List(_)),
        "dict" => matches!(value, TclValue::Dict(_)),
        _ => false,
    };
    if type_match {
        if let Some(binding) = &case.binding {
            interp.set_var(binding, value.clone());
        }
        return Ok(true);
    }
    // Literal string match
    let pat_str = pat.trim_matches('"');
    if value.as_str() == pat_str {
        if let Some(binding) = &case.binding {
            interp.set_var(binding, value.clone());
        }
        return Ok(true);
    }
    Ok(false)
}

/// Parse switch cases from body text.
fn parse_switch_cases(body: &str) -> Result<Vec<(String, String)>, TclError> {
    let words = parse_param_list(body);
    let mut cases = Vec::new();
    let mut i = 0;
    while i + 1 < words.len() {
        let pattern = words[i].trim_matches('"').to_string();
        let script = words[i + 1]
            .trim_start_matches('{')
            .trim_end_matches('}')
            .to_string();
        cases.push((pattern, script));
        i += 2;
    }
    Ok(cases)
}

/// `try body ?on error {var} body? ?finally body?`
fn cmd_try(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"try body ...\""));
    }
    let body = args[0].as_str().to_string();
    let result = interp.eval(&body);

    let mut finally_body: Option<String> = None;
    let mut handled = false;
    let mut final_result = result;

    let mut i = 1;
    while i < args.len() {
        let keyword = args[i].as_str().to_string();
        i += 1;
        if keyword == "on" {
            // on error {var} body
            if i + 2 >= args.len() {
                return Err(TclError::new("wrong # args in try/on"));
            }
            let _error_type = args[i].as_str().to_string();
            i += 1;
            let var_spec = args[i].as_str().to_string();
            i += 1;
            let handler = args[i].as_str().to_string();
            i += 1;
            if let Err(ref e) = final_result {
                if e.code == ErrorCode::Error && !handled {
                    let var = var_spec.trim_matches(|c| c == '{' || c == '}').to_string();
                    interp.set_var(&var, TclValue::Str(e.message.clone()));
                    final_result = interp.eval(&handler);
                    handled = true;
                }
            }
        } else if keyword == "finally" && i < args.len() {
            finally_body = Some(args[i].as_str().to_string());
            i += 1;
        }
    }

    if let Some(fb) = finally_body {
        interp.eval(&fb)?;
    }

    final_result
}

/// `catch script ?resultVar?`
fn cmd_catch(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"catch script ?resultVar?\"",
        ));
    }
    let script = args[0].as_str().to_string();
    let result = interp.eval(&script);
    let (code, value) = match result {
        Ok(v) => (0, v),
        Err(e) => (1, TclValue::Str(e.message)),
    };
    if args.len() > 1 {
        let var = args[1].as_str().to_string();
        interp.set_var(&var, value);
    }
    Ok(TclValue::Int(code))
}

/// `error message`
fn cmd_error(_interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"error message\""));
    }
    Err(TclError::new(args[0].as_str().to_string()))
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
        interp
            .eval("if {0} {set result yes} else {set result no}")
            .unwrap();
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
        interp
            .eval("for {set i 0} {$i < 5} {incr i} {incr sum $i}")
            .unwrap();
        assert_eq!(interp.eval("set sum").unwrap().as_str(), "10");
    }

    #[test]
    fn proc_basic() {
        let mut interp = Interpreter::new();
        interp
            .eval("proc double {x} {return [expr {$x * 2}]}")
            .unwrap();
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
    fn break_in_while() {
        let mut interp = Interpreter::new();
        interp.eval("set x 0").unwrap();
        interp
            .eval("while {1} {incr x; if {$x == 3} {break}}")
            .unwrap();
        assert_eq!(interp.eval("set x").unwrap().as_str(), "3");
    }

    #[test]
    fn return_value() {
        let mut interp = Interpreter::new();
        interp.eval("proc foo {} {return 42}").unwrap();
        assert_eq!(interp.eval("foo").unwrap().as_str(), "42");
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
        assert_eq!(
            interp.eval("set log").unwrap().as_str(),
            "caught:boom,cleaned"
        );
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
