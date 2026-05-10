//! Variable and command substitution.

use crate::error::TclError;
use crate::value::TclValue;

use super::Interpreter;

/// Check if the entire input is a single substitution ($var or [cmd]).
/// If so, return the value directly to preserve type information.
fn try_direct_subst(chars: &[char], interp: &mut Interpreter) -> Result<Option<TclValue>, TclError> {
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
pub(super) fn subst_variable(
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
            TclValue::Dict(d) => TclValue::List(d.iter().map(|(k, _)| TclValue::Str(k.clone())).collect()),
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
            Err(TclError::new(format!("key \"{key}\" not known in dictionary")))
        }
        _ => Err(TclError::new(format!(
            "can't use \"{key}\" as dict key on {}",
            val.type_name()
        ))),
    }
}

/// Substitute a command `[...]`.
pub(super) fn subst_command(
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
                let val = super::eval::eval_script_catching_return(interp, &script)?;
                return Ok((val, i));
            }
        }
        script.push(chars[i]);
        i += 1;
    }
    Err(TclError::new("unmatched '['"))
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
