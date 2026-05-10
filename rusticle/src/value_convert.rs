//! TclValue conversions: parsing, formatting, Display, PartialEq, From impls.

use std::fmt;

use crate::error::TclError;
use crate::value::TclValue;

/// Format a float, ensuring it always has a decimal point.
pub(super) fn format_float(f: f64) -> String {
    let s = f.to_string();
    if s.contains('.') {
        s
    } else {
        format!("{s}.0")
    }
}

/// Convert a list of values to a Tcl list string.
pub(super) fn list_to_string(items: &[TclValue]) -> String {
    items
        .iter()
        .map(|v: &TclValue| quote_list_element(&v.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert dict pairs to a Tcl list string.
pub(super) fn dict_to_string(pairs: &[(String, TclValue)]) -> String {
    pairs
        .iter()
        .flat_map(|(k, v): &(String, TclValue)| vec![quote_list_element(k), quote_list_element(&v.as_str())])
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quote a string for inclusion in a Tcl list.
fn quote_list_element(s: &str) -> String {
    if s.is_empty() {
        return "{}".to_string();
    }
    let needs_quoting = s.contains(|c: char| c.is_whitespace() || c == '{' || c == '}' || c == '"' || c == '\\');
    if needs_quoting {
        format!("{{{s}}}")
    } else {
        s.to_string()
    }
}

/// Parse a Tcl list string into values.
pub(super) fn parse_list(s: &str) -> Vec<TclValue> {
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let (word, next) = parse_list_word(&chars, i);
        result.push(TclValue::Str(word));
        i = next;
    }
    result
}

/// Parse one word from a list string starting at position `i`.
fn parse_list_word(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start;
    if i < chars.len() && chars[i] == '{' {
        return parse_braced_word(chars, i);
    }
    if i < chars.len() && chars[i] == '"' {
        return parse_quoted_word(chars, i);
    }
    let mut word = String::new();
    while i < chars.len() && !chars[i].is_whitespace() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            word.push(chars[i + 1]);
            i += 2;
        } else {
            word.push(chars[i]);
            i += 1;
        }
    }
    (word, i)
}

/// Parse a brace-quoted word `{...}`.
fn parse_braced_word(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1; // skip opening {
    let mut depth = 1;
    let mut word = String::new();
    while i < chars.len() && depth > 0 {
        if chars[i] == '{' {
            depth += 1;
            word.push('{');
        } else if chars[i] == '}' {
            depth -= 1;
            if depth > 0 {
                word.push('}');
            }
        } else {
            word.push(chars[i]);
        }
        i += 1;
    }
    (word, i)
}

/// Parse a double-quoted word `"..."`.
fn parse_quoted_word(chars: &[char], start: usize) -> (String, usize) {
    let mut i = start + 1; // skip opening "
    let mut word = String::new();
    while i < chars.len() && chars[i] != '"' {
        if chars[i] == '\\' && i + 1 < chars.len() {
            word.push(chars[i + 1]);
            i += 2;
        } else {
            word.push(chars[i]);
            i += 1;
        }
    }
    if i < chars.len() {
        i += 1; // skip closing "
    }
    (word, i)
}

/// Parse a string as a boolean (Tcl-compatible).
pub(super) fn str_to_bool(s: &str) -> Result<bool, TclError> {
    match s.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(TclError::new(format!("expected boolean but got \"{s}\""))),
    }
}

impl fmt::Display for TclValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl PartialEq for TclValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Int(a), Self::Float(b)) | (Self::Float(b), Self::Int(a)) => (*a as f64) == *b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Dict(a), Self::Dict(b)) => a == b,
            _ => self.as_str() == other.as_str(),
        }
    }
}

impl From<String> for TclValue {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<&str> for TclValue {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

impl From<i64> for TclValue {
    fn from(n: i64) -> Self {
        Self::Int(n)
    }
}

impl From<f64> for TclValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for TclValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Vec<TclValue>> for TclValue {
    fn from(v: Vec<TclValue>) -> Self {
        Self::List(v)
    }
}

impl From<i32> for TclValue {
    fn from(n: i32) -> Self {
        Self::Int(i64::from(n))
    }
}
