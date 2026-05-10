//! Procedure definition, pattern matching, and error handling commands.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::{Interpreter, Proc, ProcParam};
use crate::value::TclValue;

/// Register proc/match/error-handling commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("proc", cmd_proc);
    interp.register_fn("switch", cmd_switch);
    interp.register_fn("match", cmd_match);
    interp.register_fn("try", cmd_try);
    interp.register_fn("catch", cmd_catch);
    interp.register_fn("error", cmd_error);
}

/// `proc name args body`
fn cmd_proc(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() != 3 {
        return Err(TclError::new("wrong # args: should be \"proc name args body\""));
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
            params.push(ProcParam { name, default: None });
        }
    }
    Ok(params)
}

/// Strip type annotation from parameter name (e.g., "line:int" → "line").
fn strip_type_annotation(s: &str) -> String {
    s.split(':').next().unwrap_or(s).to_string()
}

/// Parse a simple parameter list (space-separated, respecting braces).
pub(super) fn parse_param_list(s: &str) -> Vec<String> {
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
        return Err(TclError::new("wrong # args: should be \"match value {cases}\""));
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
            let binding = Some(next.clone());
            i += 1;
            if i >= words.len() {
                return Err(TclError::new("missing body in match"));
            }
            let body_str = words[i].trim_start_matches('{').trim_end_matches('}').to_string();
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
fn match_pattern(interp: &mut Interpreter, value: &TclValue, case: &MatchCase) -> Result<bool, TclError> {
    let pat = &case.pattern;
    if pat == "_" {
        return Ok(true);
    }
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
        let script = words[i + 1].trim_start_matches('{').trim_end_matches('}').to_string();
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
        return Err(TclError::new("wrong # args: should be \"catch script ?resultVar?\""));
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
