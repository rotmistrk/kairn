//! TclValue — dual-representation values for the rusticle interpreter.

use std::borrow::Cow;
use std::fmt;

use crate::error::TclError;

/// The core value type. Every value has a string representation
/// and may carry typed internal data.
#[derive(Clone, Debug)]
pub enum TclValue {
    /// A string value.
    Str(String),
    /// A 64-bit integer.
    Int(i64),
    /// A 64-bit float.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// An ordered list of values.
    List(Vec<TclValue>),
    /// An ordered dict of key-value pairs.
    Dict(Vec<(String, TclValue)>),
}

impl TclValue {
    /// Return the string representation.
    pub fn as_str(&self) -> Cow<'_, str> {
        match self {
            Self::Str(s) => Cow::Borrowed(s),
            Self::Int(n) => Cow::Owned(n.to_string()),
            Self::Float(f) => Cow::Owned(format_float(*f)),
            Self::Bool(b) => Cow::Borrowed(if *b { "1" } else { "0" }),
            Self::List(items) => Cow::Owned(list_to_string(items)),
            Self::Dict(pairs) => Cow::Owned(dict_to_string(pairs)),
        }
    }

    /// Try to interpret as an integer.
    pub fn as_int(&self) -> Result<i64, TclError> {
        match self {
            Self::Int(n) => Ok(*n),
            Self::Float(f) => Ok(*f as i64),
            Self::Bool(b) => Ok(i64::from(*b)),
            Self::Str(s) => s
                .trim()
                .parse::<i64>()
                .map_err(|_| TclError::new(format!("expected integer but got \"{}\"", s))),
            _ => Err(TclError::new(format!(
                "expected integer but got {}",
                self.type_name()
            ))),
        }
    }

    /// Try to interpret as a float.
    pub fn as_float(&self) -> Result<f64, TclError> {
        match self {
            Self::Float(f) => Ok(*f),
            Self::Int(n) => Ok(*n as f64),
            Self::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Self::Str(s) => s
                .trim()
                .parse::<f64>()
                .map_err(|_| TclError::new(format!("expected number but got \"{}\"", s))),
            _ => Err(TclError::new(format!(
                "expected number but got {}",
                self.type_name()
            ))),
        }
    }

    /// Try to interpret as a boolean.
    pub fn as_bool(&self) -> Result<bool, TclError> {
        match self {
            Self::Bool(b) => Ok(*b),
            Self::Int(n) => Ok(*n != 0),
            Self::Float(f) => Ok(*f != 0.0),
            Self::Str(s) => str_to_bool(s),
            _ => Err(TclError::new(format!(
                "expected boolean but got {}",
                self.type_name()
            ))),
        }
    }

    /// Try to interpret as a list slice.
    pub fn as_list(&self) -> Result<Vec<TclValue>, TclError> {
        match self {
            Self::List(items) => Ok(items.clone()),
            Self::Str(s) if s.is_empty() => Ok(Vec::new()),
            Self::Str(s) => Ok(parse_list(s)),
            _ => Ok(vec![self.clone()]),
        }
    }

    /// Return the type name as a string.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Str(_) => "string",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::List(_) => "list",
            Self::Dict(_) => "dict",
        }
    }

    /// Return true if this value is empty (empty string, empty list, empty dict).
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Str(s) => s.is_empty(),
            Self::List(l) => l.is_empty(),
            Self::Dict(d) => d.is_empty(),
            _ => false,
        }
    }
}

/// Format a float, ensuring it always has a decimal point.
fn format_float(f: f64) -> String {
    let s = f.to_string();
    if s.contains('.') {
        s
    } else {
        format!("{s}.0")
    }
}

/// Convert a list of values to a Tcl list string.
fn list_to_string(items: &[TclValue]) -> String {
    items
        .iter()
        .map(|v| quote_list_element(&v.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert dict pairs to a Tcl list string.
fn dict_to_string(pairs: &[(String, TclValue)]) -> String {
    pairs
        .iter()
        .flat_map(|(k, v)| vec![quote_list_element(k), quote_list_element(&v.as_str())])
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quote a string for inclusion in a Tcl list.
fn quote_list_element(s: &str) -> String {
    if s.is_empty() {
        return "{}".to_string();
    }
    let needs_quoting =
        s.contains(|c: char| c.is_whitespace() || c == '{' || c == '}' || c == '"' || c == '\\');
    if needs_quoting {
        format!("{{{s}}}")
    } else {
        s.to_string()
    }
}

/// Parse a Tcl list string into values.
fn parse_list(s: &str) -> Vec<TclValue> {
    let mut result = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Skip whitespace
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
fn str_to_bool(s: &str) -> Result<bool, TclError> {
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
        // Cross-type equality: compare string representations
        // but try numeric comparison first for numeric types
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_as_str() {
        let v = TclValue::Str("hello".into());
        assert_eq!(v.as_str(), "hello");
    }

    #[test]
    fn int_as_str() {
        let v = TclValue::Int(42);
        assert_eq!(v.as_str(), "42");
    }

    #[test]
    fn float_as_str() {
        let v = TclValue::Float(3.14);
        assert_eq!(v.as_str(), "3.14");
    }

    #[test]
    fn bool_as_str() {
        assert_eq!(TclValue::Bool(true).as_str(), "1");
        assert_eq!(TclValue::Bool(false).as_str(), "0");
    }

    #[test]
    fn list_as_str() {
        let v = TclValue::List(vec![TclValue::Str("a".into()), TclValue::Int(1)]);
        assert_eq!(v.as_str(), "a 1");
    }

    #[test]
    fn dict_as_str() {
        let v = TclValue::Dict(vec![
            ("name".into(), TclValue::Str("kairn".into())),
            ("ver".into(), TclValue::Int(1)),
        ]);
        assert_eq!(v.as_str(), "name kairn ver 1");
    }

    #[test]
    fn str_to_int() {
        let v = TclValue::Str("42".into());
        assert_eq!(v.as_int().unwrap(), 42);
    }

    #[test]
    fn str_to_int_fail() {
        let v = TclValue::Str("abc".into());
        assert!(v.as_int().is_err());
    }

    #[test]
    fn str_to_float() {
        let v = TclValue::Str("3.14".into());
        assert!((v.as_float().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn str_to_bool_variants() {
        for (s, expected) in &[
            ("true", true),
            ("yes", true),
            ("on", true),
            ("1", true),
            ("false", false),
            ("no", false),
            ("off", false),
            ("0", false),
        ] {
            let v = TclValue::Str((*s).into());
            assert_eq!(v.as_bool().unwrap(), *expected, "failed for {s}");
        }
    }

    #[test]
    fn str_to_bool_fail() {
        let v = TclValue::Str("maybe".into());
        assert!(v.as_bool().is_err());
    }

    #[test]
    fn cross_type_equality() {
        assert_eq!(TclValue::Int(42), TclValue::Str("42".into()));
        assert_eq!(TclValue::Str("42".into()), TclValue::Int(42));
    }

    #[test]
    fn int_float_equality() {
        assert_eq!(TclValue::Int(3), TclValue::Float(3.0));
    }

    #[test]
    fn from_impls() {
        let _: TclValue = "hello".into();
        let _: TclValue = String::from("hello").into();
        let _: TclValue = 42i64.into();
        let _: TclValue = 3.14f64.into();
        let _: TclValue = true.into();
    }

    #[test]
    fn empty_string_as_list() {
        let v = TclValue::Str(String::new());
        assert_eq!(v.as_list().unwrap().len(), 0);
    }

    #[test]
    fn string_as_list_parsing() {
        let v = TclValue::Str("a b c".into());
        let list = v.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_str(), "a");
    }

    #[test]
    fn type_name() {
        assert_eq!(TclValue::Str("x".into()).type_name(), "string");
        assert_eq!(TclValue::Int(1).type_name(), "int");
        assert_eq!(TclValue::Float(1.0).type_name(), "float");
        assert_eq!(TclValue::Bool(true).type_name(), "bool");
        assert_eq!(TclValue::List(vec![]).type_name(), "list");
        assert_eq!(TclValue::Dict(vec![]).type_name(), "dict");
    }

    #[test]
    fn display_matches_as_str() {
        let values = vec![
            TclValue::Str("hello".into()),
            TclValue::Int(42),
            TclValue::Float(3.14),
            TclValue::Bool(true),
        ];
        for v in values {
            assert_eq!(format!("{v}"), v.as_str());
        }
    }

    #[test]
    fn list_with_spaces_quoted() {
        let v = TclValue::List(vec![
            TclValue::Str("hello world".into()),
            TclValue::Str("foo".into()),
        ]);
        assert_eq!(v.as_str(), "{hello world} foo");
    }

    #[test]
    fn is_empty() {
        assert!(TclValue::Str(String::new()).is_empty());
        assert!(TclValue::List(vec![]).is_empty());
        assert!(TclValue::Dict(vec![]).is_empty());
        assert!(!TclValue::Int(0).is_empty());
    }
}
