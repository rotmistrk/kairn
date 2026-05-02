//! Type system: declarations, checking, inference, proc signatures.

use crate::error::TclError;
use crate::value::TclValue;

/// A type declaration.
#[derive(Clone, Debug, PartialEq)]
pub enum TypeDecl {
    /// Any string.
    String,
    /// Integer.
    Int,
    /// Float.
    Float,
    /// Boolean.
    Bool,
    /// Enum with allowed values.
    Enum(Vec<String>),
    /// Nullable wrapper.
    Nullable(Box<TypeDecl>),
    /// List of elements.
    List(Option<Box<TypeDecl>>),
    /// Dict (untyped keys/values).
    Dict,
    /// Record with typed fields.
    Record(Vec<(String, TypeDecl)>),
}

impl TypeDecl {
    /// Parse a type specification string.
    pub fn parse(spec: &str) -> Result<Self, TclError> {
        let s = spec.trim();
        if let Some(base) = s.strip_suffix('?') {
            let inner = Self::parse(base)?;
            return Ok(Self::Nullable(Box::new(inner)));
        }
        if s.starts_with("enum") {
            return parse_enum(s);
        }
        if let Some(elem_spec) = s.strip_prefix("list:") {
            let elem_type = Self::parse(elem_spec)?;
            return Ok(Self::List(Some(Box::new(elem_type))));
        }
        if s.starts_with("record") {
            return parse_record(s);
        }
        match s {
            "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "bool" => Ok(Self::Bool),
            "list" => Ok(Self::List(None)),
            "dict" => Ok(Self::Dict),
            _ => Err(TclError::new(format!("unknown type: {s}"))),
        }
    }

    /// Check if a value conforms to this type.
    pub fn check(&self, value: &TclValue) -> Result<(), TclError> {
        match self {
            Self::String => Ok(()),
            Self::Int => value.as_int().map(|_| ()),
            Self::Float => value.as_float().map(|_| ()),
            Self::Bool => value.as_bool().map(|_| ()),
            Self::Enum(variants) => {
                let s = value.as_str();
                if variants.iter().any(|v| v == s.as_ref()) {
                    Ok(())
                } else {
                    Err(TclError::new(format!(
                        "\"{}\" is not a valid enum value; expected: {}",
                        s,
                        variants.join(", ")
                    )))
                }
            }
            Self::Nullable(inner) => {
                if value.is_empty() {
                    Ok(())
                } else {
                    inner.check(value)
                }
            }
            Self::List(elem) => {
                let list = value.as_list()?;
                if let Some(elem_type) = elem {
                    for item in &list {
                        elem_type.check(item)?;
                    }
                }
                Ok(())
            }
            Self::Dict => match value {
                TclValue::Dict(_) => Ok(()),
                _ => Err(TclError::new("expected dict")),
            },
            Self::Record(fields) => check_record(value, fields),
        }
    }

    /// Return the display name of this type.
    pub fn name(&self) -> String {
        match self {
            Self::String => "string".into(),
            Self::Int => "int".into(),
            Self::Float => "float".into(),
            Self::Bool => "bool".into(),
            Self::Enum(v) => format!("enum {{{}}}", v.join(" ")),
            Self::Nullable(inner) => format!("{}?", inner.name()),
            Self::List(None) => "list".into(),
            Self::List(Some(elem)) => format!("list:{}", elem.name()),
            Self::Dict => "dict".into(),
            Self::Record(fields) => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, t)| format!("{k}:{}", t.name()))
                    .collect();
                format!("record {{{}}}", fs.join(" "))
            }
        }
    }
}

/// Inferred type from known builtins.
#[derive(Clone, Debug, PartialEq)]
pub enum InferredType {
    /// Known concrete type.
    Known(TypeDecl),
    /// Unknown type.
    Unknown,
}

/// Infer the return type of a known builtin command.
pub fn infer_return_type(cmd: &str, subcmd: Option<&str>) -> InferredType {
    match (cmd, subcmd) {
        ("expr", _) => InferredType::Known(TypeDecl::Int), // simplified
        ("string", Some("length")) => InferredType::Known(TypeDecl::Int),
        ("llength", _) => InferredType::Known(TypeDecl::Int),
        ("dict", Some("size")) => InferredType::Known(TypeDecl::Int),
        ("lindex", _) => InferredType::Known(TypeDecl::String),
        ("dict", Some("get")) => InferredType::Known(TypeDecl::String),
        ("dict", Some("keys")) => InferredType::Known(TypeDecl::List(None)),
        ("dict", Some("values")) => InferredType::Known(TypeDecl::List(None)),
        ("list", _) => InferredType::Known(TypeDecl::List(None)),
        ("dict", Some("create")) => InferredType::Known(TypeDecl::Dict),
        ("string", _) => InferredType::Known(TypeDecl::String),
        _ => InferredType::Unknown,
    }
}

/// Parse an enum type spec: `enum {a b c}`.
fn parse_enum(s: &str) -> Result<TypeDecl, TclError> {
    let inner = s
        .strip_prefix("enum")
        .unwrap_or("")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let variants: Vec<String> = inner.split_whitespace().map(|v| v.to_string()).collect();
    if variants.is_empty() {
        return Err(TclError::new("enum type requires at least one variant"));
    }
    Ok(TypeDecl::Enum(variants))
}

/// Parse a record type spec: `record {name:string age:int}`.
fn parse_record(s: &str) -> Result<TypeDecl, TclError> {
    let inner = s
        .strip_prefix("record")
        .unwrap_or("")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let mut fields = Vec::new();
    for field in inner.split_whitespace() {
        if let Some((name, type_str)) = field.split_once(':') {
            let td = TypeDecl::parse(type_str)?;
            fields.push((name.to_string(), td));
        } else {
            return Err(TclError::new(format!("invalid record field: {field}")));
        }
    }
    Ok(TypeDecl::Record(fields))
}

/// Check a value against a record type.
fn check_record(value: &TclValue, fields: &[(String, TypeDecl)]) -> Result<(), TclError> {
    let pairs = match value {
        TclValue::Dict(d) => d,
        _ => return Err(TclError::new("expected dict for record type")),
    };
    for (name, td) in fields {
        let val = pairs.iter().find(|(k, _)| k == name).map(|(_, v)| v);
        match val {
            Some(v) => td.check(v)?,
            None => {
                return Err(TclError::new(format!("missing field \"{name}\" in record")));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_types() {
        assert_eq!(TypeDecl::parse("string").unwrap(), TypeDecl::String);
        assert_eq!(TypeDecl::parse("int").unwrap(), TypeDecl::Int);
        assert_eq!(TypeDecl::parse("float").unwrap(), TypeDecl::Float);
        assert_eq!(TypeDecl::parse("bool").unwrap(), TypeDecl::Bool);
        assert_eq!(TypeDecl::parse("list").unwrap(), TypeDecl::List(None));
        assert_eq!(TypeDecl::parse("dict").unwrap(), TypeDecl::Dict);
    }

    #[test]
    fn parse_nullable() {
        let td = TypeDecl::parse("int?").unwrap();
        assert_eq!(td, TypeDecl::Nullable(Box::new(TypeDecl::Int)));
    }

    #[test]
    fn parse_enum() {
        let td = TypeDecl::parse("enum {a b c}").unwrap();
        assert_eq!(td, TypeDecl::Enum(vec!["a".into(), "b".into(), "c".into()]));
    }

    #[test]
    fn parse_typed_list() {
        let td = TypeDecl::parse("list:int").unwrap();
        assert_eq!(td, TypeDecl::List(Some(Box::new(TypeDecl::Int))));
    }

    #[test]
    fn parse_record() {
        let td = TypeDecl::parse("record {name:string age:int}").unwrap();
        assert_eq!(
            td,
            TypeDecl::Record(vec![
                ("name".into(), TypeDecl::String),
                ("age".into(), TypeDecl::Int),
            ])
        );
    }

    #[test]
    fn check_int() {
        assert!(TypeDecl::Int.check(&TclValue::Int(42)).is_ok());
        assert!(TypeDecl::Int.check(&TclValue::Str("abc".into())).is_err());
    }

    #[test]
    fn check_enum() {
        let td = TypeDecl::Enum(vec!["a".into(), "b".into()]);
        assert!(td.check(&TclValue::Str("a".into())).is_ok());
        assert!(td.check(&TclValue::Str("c".into())).is_err());
    }

    #[test]
    fn check_nullable() {
        let td = TypeDecl::Nullable(Box::new(TypeDecl::Int));
        assert!(td.check(&TclValue::Str(String::new())).is_ok());
        assert!(td.check(&TclValue::Int(42)).is_ok());
        assert!(td.check(&TclValue::Str("abc".into())).is_err());
    }

    #[test]
    fn check_typed_list() {
        let td = TypeDecl::List(Some(Box::new(TypeDecl::Int)));
        let good = TclValue::List(vec![TclValue::Int(1), TclValue::Int(2)]);
        assert!(td.check(&good).is_ok());
        let bad = TclValue::List(vec![TclValue::Str("abc".into())]);
        assert!(td.check(&bad).is_err());
    }

    #[test]
    fn check_record() {
        let td = TypeDecl::Record(vec![
            ("name".into(), TypeDecl::String),
            ("age".into(), TypeDecl::Int),
        ]);
        let good = TclValue::Dict(vec![
            ("name".into(), TclValue::Str("alice".into())),
            ("age".into(), TclValue::Int(30)),
        ]);
        assert!(td.check(&good).is_ok());
        let missing = TclValue::Dict(vec![("name".into(), TclValue::Str("alice".into()))]);
        assert!(td.check(&missing).is_err());
    }

    #[test]
    fn infer_known_types() {
        assert_eq!(
            infer_return_type("llength", None),
            InferredType::Known(TypeDecl::Int)
        );
        assert_eq!(
            infer_return_type("string", Some("length")),
            InferredType::Known(TypeDecl::Int)
        );
        assert_eq!(
            infer_return_type("unknown_cmd", None),
            InferredType::Unknown
        );
    }

    #[test]
    fn type_name_display() {
        assert_eq!(TypeDecl::String.name(), "string");
        assert_eq!(TypeDecl::Int.name(), "int");
        assert_eq!(TypeDecl::Nullable(Box::new(TypeDecl::Int)).name(), "int?");
        assert_eq!(
            TypeDecl::Enum(vec!["a".into(), "b".into()]).name(),
            "enum {a b}"
        );
    }
}
