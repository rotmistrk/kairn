//! Structured literal evaluation: `%{ ... }` dict and `%[ ... ]` list literals.

use crate::error::TclError;
use crate::value::TclValue;

use super::subst::substitute;
use super::Interpreter;

/// Evaluate a `%{ ... }` dict literal.
pub fn eval_dict_literal(interp: &mut Interpreter, content: &str) -> Result<TclValue, TclError> {
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
pub fn eval_list_literal(interp: &mut Interpreter, content: &str) -> Result<TclValue, TclError> {
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
    let colon_pos = entry
        .find(':')
        .ok_or_else(|| TclError::new(format!("expected 'key: value' in dict literal, got: {entry}")))?;
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
