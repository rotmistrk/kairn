//! Variable and command substitution.
use super::subst_access::{
    apply_index, apply_property, is_var_char, lookup_var, subst_command, try_parse_value, unescape_char,
};

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
