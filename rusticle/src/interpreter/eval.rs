//! Script evaluation: parse, substitute, and dispatch commands.

use crate::error::{ErrorCode, TclError};
use crate::parser::Parser;
use crate::parser::Word;
use crate::value::TclValue;

use super::{Interpreter, Proc};

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
pub fn eval_script_catching_return(
    interp: &mut Interpreter,
    script: &str,
) -> Result<TclValue, TclError> {
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

/// Check if the entire input is a single substitution ($var or [cmd]).
/// If so, return the value directly to preserve type information.
fn try_direct_subst(
    chars: &[char],
    interp: &mut Interpreter,
) -> Result<Option<TclValue>, TclError> {
    if chars.is_empty() {
        return Ok(None);
    }
    if chars[0] == '$' {
        let (val, end) = subst_variable(interp, chars, 0)?;
        if end == chars.len() {
            return Ok(Some(val));
        }
    } else if chars[0] == '[' {
        let (val, end) = subst_command(interp, chars, 0)?;
        if end == chars.len() {
            return Ok(Some(val));
        }
    }
    Ok(None)
}

/// Perform variable and command substitution on a string.
pub fn substitute(interp: &mut Interpreter, input: &str) -> Result<TclValue, TclError> {
    let chars: Vec<char> = input.chars().collect();
    // Optimization: if the entire input is a single $var, return the value directly
    // to preserve type information (Dict, List, etc.)
    if let Some(direct) = try_direct_subst(&chars, interp)? {
        return Ok(direct);
    }
    let mut result = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            let (val, next) = subst_variable(interp, &chars, i)?;
            result.push_str(&val.as_str());
            i = next;
        } else if chars[i] == '[' {
            let (val, next) = subst_command(interp, &chars, i)?;
            result.push_str(&val.as_str());
            i = next;
        } else if chars[i] == '\\' && i + 1 < chars.len() {
            result.push(unescape_char(chars[i + 1]));
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Ok(try_parse_value(&result))
}

/// Substitute a variable reference starting at `$`.
fn subst_variable(
    interp: &mut Interpreter,
    chars: &[char],
    start: usize,
) -> Result<(TclValue, usize), TclError> {
    let mut i = start + 1; // skip $
    if i >= chars.len() {
        return Ok((TclValue::Str("$".into()), i));
    }
    // ${varname} form
    if chars[i] == '{' {
        i += 1;
        let mut name = String::new();
        while i < chars.len() && chars[i] != '}' {
            name.push(chars[i]);
            i += 1;
        }
        if i < chars.len() {
            i += 1; // skip }
        }
        let val = lookup_var(interp, &name)?;
        return Ok((val, i));
    }
    // Regular $varname with optional accessors
    let mut name = String::new();
    while i < chars.len() && is_var_char(chars[i]) {
        name.push(chars[i]);
        i += 1;
    }
    if name.is_empty() {
        return Ok((TclValue::Str("$".into()), i));
    }
    let mut val = lookup_var(interp, &name)?;
    // Handle accessor chains: .prop, (index), ?, ??
    loop {
        if i >= chars.len() {
            break;
        }
        if chars[i] == '.' {
            let (new_val, next) = apply_property(interp, &val, chars, i)?;
            val = new_val;
            i = next;
        } else if chars[i] == '(' {
            let (new_val, next) = apply_index(interp, &val, chars, i)?;
            val = new_val;
            i = next;
        } else {
            break;
        }
    }
    // Optional chaining: ? after accessor
    if i < chars.len() && chars[i] == '?' {
        i += 1;
        // Null coalescing: ??
        if i < chars.len() && chars[i] == '?' {
            i += 1;
            // Skip whitespace
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            // Read default value (bare word)
            let mut default = String::new();
            while i < chars.len() && chars[i] != ' ' && chars[i] != ']' && chars[i] != '"' {
                default.push(chars[i]);
                i += 1;
            }
            if val.is_empty() {
                val = TclValue::Str(default);
            }
        }
    }
    Ok((val, i))
}

/// Look up a variable by name, supporting `::` context access.
fn lookup_var(interp: &Interpreter, name: &str) -> Result<TclValue, TclError> {
    interp
        .get_var(name)
        .cloned()
        .ok_or_else(|| TclError::new(format!("can't read \"{name}\": no such variable")))
}

/// Apply a property accessor `.prop`.
fn apply_property(
    interp: &mut Interpreter,
    val: &TclValue,
    chars: &[char],
    start: usize,
) -> Result<(TclValue, usize), TclError> {
    let mut i = start + 1; // skip .
    let mut prop = String::new();
    while i < chars.len() && is_var_char(chars[i]) {
        prop.push(chars[i]);
        i += 1;
    }
    let result = match prop.as_str() {
        "len" => match val {
            TclValue::List(l) => TclValue::Int(l.len() as i64),
            TclValue::Dict(d) => TclValue::Int(d.len() as i64),
            TclValue::Str(s) => TclValue::Int(s.len() as i64),
            _ => TclValue::Int(val.as_str().len() as i64),
        },
        "keys" => match val {
            TclValue::Dict(d) => {
                TclValue::List(d.iter().map(|(k, _)| TclValue::Str(k.clone())).collect())
            }
            _ => return Err(TclError::new(".keys requires a dict")),
        },
        "values" => match val {
            TclValue::Dict(d) => TclValue::List(d.iter().map(|(_, v)| v.clone()).collect()),
            _ => return Err(TclError::new(".values requires a dict")),
        },
        "type" => TclValue::Str(val.type_name().into()),
        _ => {
            // Try as dict key access
            dict_get(val, &prop, interp)?
        }
    };
    Ok((result, i))
}

/// Apply an index accessor `(index)` or `(key)`.
fn apply_index(
    interp: &mut Interpreter,
    val: &TclValue,
    chars: &[char],
    start: usize,
) -> Result<(TclValue, usize), TclError> {
    let mut i = start + 1; // skip (
    let mut key_str = String::new();
    let mut depth = 1;
    while i < chars.len() && depth > 0 {
        if chars[i] == '(' {
            depth += 1;
        } else if chars[i] == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        key_str.push(chars[i]);
        i += 1;
    }
    if i < chars.len() {
        i += 1; // skip )
    }
    // Handle optional chaining: )?
    let optional = i < chars.len() && chars[i] == '?';
    if optional {
        i += 1;
    }
    // Resolve the key (may contain $var references)
    let key = if key_str.starts_with('"') && key_str.ends_with('"') {
        key_str[1..key_str.len() - 1].to_string()
    } else if key_str.starts_with('$') {
        let resolved = substitute(interp, &key_str)?;
        resolved.as_str().to_string()
    } else {
        key_str.clone()
    };
    // Check for range: a..b
    if let Some(range_result) = try_range_access(val, &key)? {
        return Ok((range_result, i));
    }
    let result = index_value(val, &key);
    match result {
        Ok(v) => Ok((v, i)),
        Err(_e) if optional => Ok((TclValue::Str(String::new()), i)),
        Err(e) => Err(e),
    }
}

/// Try to interpret the key as a range (e.g., "2..5", "3..", "..5").
fn try_range_access(val: &TclValue, key: &str) -> Result<Option<TclValue>, TclError> {
    if !key.contains("..") {
        return Ok(None);
    }
    let parts: Vec<&str> = key.splitn(2, "..").collect();
    if parts.len() != 2 {
        return Ok(None);
    }
    let list = val.as_list()?;
    let len = list.len();
    let start = if parts[0].is_empty() {
        0
    } else {
        parts[0]
            .parse::<usize>()
            .map_err(|_| TclError::new(format!("invalid range start: {}", parts[0])))?
    };
    let end = if parts[1].is_empty() {
        len
    } else {
        parts[1]
            .parse::<usize>()
            .map_err(|_| TclError::new(format!("invalid range end: {}", parts[1])))?
    };
    let end = end.min(len);
    let start = start.min(len);
    Ok(Some(TclValue::List(list[start..end].to_vec())))
}

/// Index into a value (list by integer, dict by key).
fn index_value(val: &TclValue, key: &str) -> Result<TclValue, TclError> {
    // Try as integer index for lists
    if let Ok(idx) = key.parse::<i64>() {
        let list = val.as_list()?;
        let idx = if idx < 0 {
            (list.len() as i64 + idx) as usize
        } else {
            idx as usize
        };
        return list
            .get(idx)
            .cloned()
            .ok_or_else(|| TclError::new(format!("list index {idx} out of range")));
    }
    // Dict key access
    dict_get(val, key, &Interpreter::new())
}

/// Get a value from a dict by key.
fn dict_get(val: &TclValue, key: &str, _interp: &Interpreter) -> Result<TclValue, TclError> {
    match val {
        TclValue::Dict(pairs) => {
            for (k, v) in pairs {
                if k == key {
                    return Ok(v.clone());
                }
            }
            Err(TclError::new(format!(
                "key \"{key}\" not known in dictionary"
            )))
        }
        _ => Err(TclError::new(format!(
            "can't use \"{key}\" as dict key on {}",
            val.type_name()
        ))),
    }
}

/// Substitute a command `[...]`.
fn subst_command(
    interp: &mut Interpreter,
    chars: &[char],
    start: usize,
) -> Result<(TclValue, usize), TclError> {
    let mut i = start + 1; // skip [
    let mut depth = 1;
    let mut script = String::new();
    while i < chars.len() && depth > 0 {
        if chars[i] == '[' {
            depth += 1;
        } else if chars[i] == ']' {
            depth -= 1;
            if depth == 0 {
                i += 1;
                let val = eval_script_catching_return(interp, &script)?;
                return Ok((val, i));
            }
        }
        script.push(chars[i]);
        i += 1;
    }
    Err(TclError::new("unmatched '['"))
}

/// Evaluate a `%{ ... }` dict literal.
fn eval_dict_literal(interp: &mut Interpreter, content: &str) -> Result<TclValue, TclError> {
    let mut pairs = Vec::new();
    let entries = split_literal_entries(content);
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (key, val_str) = split_kv(entry)?;
        let val = eval_literal_value(interp, val_str.trim())?;
        pairs.push((key, val));
    }
    Ok(TclValue::Dict(pairs))
}

/// Evaluate a `%[ ... ]` list literal.
fn eval_list_literal(interp: &mut Interpreter, content: &str) -> Result<TclValue, TclError> {
    let mut items = Vec::new();
    let entries = split_literal_entries(content);
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        items.push(eval_literal_value(interp, entry)?);
    }
    Ok(TclValue::List(items))
}

/// Split literal entries by commas or newlines (respecting nesting).
fn split_literal_entries(content: &str) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut i = 0;
    let mut depth = 0;
    let mut in_quotes = false;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() {
            current.push(ch);
            current.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if ch == '"' && depth == 0 {
            in_quotes = !in_quotes;
            current.push(ch);
            i += 1;
            continue;
        }
        if !in_quotes {
            if ch == '{' || ch == '[' {
                depth += 1;
            } else if (ch == '}' || ch == ']') && depth > 0 {
                depth -= 1;
            }
            if depth == 0 && (ch == ',' || ch == '\n') {
                entries.push(current.trim().to_string());
                current = String::new();
                i += 1;
                continue;
            }
        }
        current.push(ch);
        i += 1;
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        entries.push(trimmed);
    }
    entries
}

/// Split a `key: value` pair.
fn split_kv(entry: &str) -> Result<(String, &str), TclError> {
    let colon_pos = entry.find(':').ok_or_else(|| {
        TclError::new(format!(
            "expected 'key: value' in dict literal, got: {entry}"
        ))
    })?;
    let key = entry[..colon_pos].trim();
    let key = if key.starts_with('"') && key.ends_with('"') {
        key[1..key.len() - 1].to_string()
    } else {
        key.to_string()
    };
    let val = entry[colon_pos + 1..].trim();
    Ok((key, val))
}

/// Evaluate a single value in a structured literal.
fn eval_literal_value(interp: &mut Interpreter, s: &str) -> Result<TclValue, TclError> {
    if s == "true" {
        return Ok(TclValue::Bool(true));
    }
    if s == "false" {
        return Ok(TclValue::Bool(false));
    }
    if s.starts_with('"') && s.ends_with('"') {
        let inner = &s[1..s.len() - 1];
        return substitute(interp, inner);
    }
    if s.starts_with("%{") {
        let inner = &s[2..s.len() - 1];
        return eval_dict_literal(interp, inner);
    }
    if s.starts_with("%[") {
        let inner = &s[2..s.len() - 1];
        return eval_list_literal(interp, inner);
    }
    if s.starts_with('$') {
        return substitute(interp, s);
    }
    if let Ok(n) = s.parse::<i64>() {
        return Ok(TclValue::Int(n));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(TclValue::Float(f));
    }
    Ok(TclValue::Str(s.to_string()))
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
fn call_proc(
    interp: &mut Interpreter,
    proc_def: &Proc,
    args: &[TclValue],
) -> Result<TclValue, TclError> {
    // Check arity
    let min_args = proc_def
        .params
        .iter()
        .filter(|p| p.default.is_none())
        .count();
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

/// Check if a character is valid in a variable name.
fn is_var_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == ':'
}

/// Unescape a backslash sequence.
fn unescape_char(c: char) -> char {
    match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '\\' => '\\',
        '"' => '"',
        '$' => '$',
        '[' => '[',
        ']' => ']',
        '{' => '{',
        '}' => '}',
        _ => c,
    }
}

/// Try to parse a string as a typed value.
fn try_parse_value(s: &str) -> TclValue {
    TclValue::Str(s.to_string())
}
