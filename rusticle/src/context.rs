//! Context blocks: named scopes with typed declarations.
//!
//! `context name { body }` creates a named scope. Variables inside are
//! accessed as `$ctx::var` and procs as `ctx::proc`.

use std::collections::HashMap;

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// A context: a named scope with optional type declarations.
#[derive(Clone, Debug)]
pub struct Context {
    /// Context name.
    pub name: String,
    /// Variables in this context.
    pub vars: HashMap<String, TclValue>,
    /// Type declarations for variables.
    pub declarations: HashMap<String, String>,
}

impl Context {
    /// Create a new empty context.
    pub fn new(name: String) -> Self {
        Self {
            name,
            vars: HashMap::new(),
            declarations: HashMap::new(),
        }
    }
}

/// Register context-related commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("context", cmd_context);
    interp.register_fn("declare", cmd_declare);
}

/// `context name { body }` — create a named scope.
fn cmd_context(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"context name body\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let body = args[1].as_str().to_string();

    // Create context and store it
    let ctx = Context::new(name.clone());
    interp.contexts.insert(name.clone(), ctx);

    // Evaluate body in a new scope
    interp.push_scope();
    interp.current_context = Some(name.clone());
    let result = interp.eval(&body);
    interp.current_context = None;

    // Capture variables from the scope into the context
    if let Some(scope) = interp.scopes.last() {
        let vars = scope.vars.clone();
        if let Some(ctx) = interp.contexts.get_mut(&name) {
            ctx.vars = vars;
        }
    }
    interp.pop_scope();

    // Register context variables as ctx::var in the global scope
    let vars_to_set: Vec<(String, TclValue)> = interp
        .contexts
        .get(&name)
        .map(|ctx| {
            ctx.vars
                .iter()
                .map(|(var, val)| (format!("{name}::{var}"), val.clone()))
                .collect()
        })
        .unwrap_or_default();
    for (qualified, val) in vars_to_set {
        interp.set_var(&qualified, val);
    }

    result
}

/// `declare var : type` — declare a typed variable in the current context.
fn cmd_declare(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"declare varName : type\"",
        ));
    }
    let var_name = args[0].as_str().to_string();
    // args[1] should be ":"
    // Concatenate remaining args for type spec (e.g., "enum {a b c}")
    let type_spec = args[2..]
        .iter()
        .map(|a| a.as_str().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    if let Some(ctx_name) = &interp.current_context.clone() {
        if let Some(ctx) = interp.contexts.get_mut(ctx_name) {
            ctx.declarations.insert(var_name, type_spec);
        }
    }
    Ok(TclValue::Str(String::new()))
}

/// Validate a value against a type declaration.
pub fn validate_type(type_spec: &str, value: &TclValue) -> Result<(), TclError> {
    let spec = type_spec.trim();
    // Nullable types
    if let Some(base) = spec.strip_suffix('?') {
        if value.is_empty() {
            return Ok(());
        }
        return validate_type(base, value);
    }
    match spec {
        "string" => Ok(()),
        "int" => value.as_int().map(|_| ()),
        "float" => value.as_float().map(|_| ()),
        "bool" => value.as_bool().map(|_| ()),
        "list" => value.as_list().map(|_| ()),
        "dict" => match value {
            TclValue::Dict(_) => Ok(()),
            _ => Err(TclError::new("expected dict")),
        },
        _ if spec.starts_with("enum") => validate_enum(spec, value),
        _ => Ok(()),
    }
}

/// Validate a value against an enum type.
fn validate_enum(spec: &str, value: &TclValue) -> Result<(), TclError> {
    // Parse "enum {a b c}"
    let inner = spec
        .strip_prefix("enum")
        .unwrap_or("")
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    let variants: Vec<&str> = inner.split_whitespace().collect();
    let val_str = value.as_str();
    if variants.iter().any(|v| *v == val_str.as_ref()) {
        Ok(())
    } else {
        Err(TclError::new(format!(
            "\"{}\" is not a valid value for {spec}; expected one of: {}",
            val_str,
            variants.join(", ")
        )))
    }
}

/// Check type on context variable assignment.
/// Called when setting `ctx::var` to validate against declarations.
pub fn check_context_assignment(
    interp: &Interpreter,
    qualified_name: &str,
    value: &TclValue,
) -> Result<(), TclError> {
    if let Some((ctx_name, var_name)) = qualified_name.split_once("::") {
        if let Some(ctx) = interp.contexts.get(ctx_name) {
            if let Some(type_spec) = ctx.declarations.get(var_name) {
                return validate_type(type_spec, value);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::value::TclValue;

    use super::validate_type;

    #[test]
    fn context_basic() {
        let mut interp = Interpreter::new();
        interp
            .eval(
                r#"context cfg {
            set mode normal
            set count 0
        }"#,
            )
            .unwrap();
        assert_eq!(interp.eval("set cfg::mode").unwrap().as_str(), "normal");
        assert_eq!(interp.eval("set cfg::count").unwrap().as_str(), "0");
    }

    #[test]
    fn context_with_proc() {
        let mut interp = Interpreter::new();
        interp
            .eval(
                r#"context math {
            set pi 3
            proc double {x} { return [expr {$x * 2}] }
        }"#,
            )
            .unwrap();
        assert_eq!(interp.eval("set math::pi").unwrap().as_str(), "3");
    }

    #[test]
    fn declare_and_validate() {
        let mut interp = Interpreter::new();
        interp
            .eval("context cfg { declare mode : enum {a b c} }")
            .unwrap();
        // Valid assignment
        interp.eval("set cfg::mode a").unwrap();
        // Invalid assignment should fail
        let result = interp.eval("set cfg::mode z");
        assert!(result.is_err());
    }

    #[test]
    fn validate_type_string() {
        assert!(validate_type("string", &TclValue::Str("hello".into())).is_ok());
    }

    #[test]
    fn validate_type_int() {
        assert!(validate_type("int", &TclValue::Int(42)).is_ok());
        assert!(validate_type("int", &TclValue::Str("abc".into())).is_err());
    }

    #[test]
    fn validate_type_bool() {
        assert!(validate_type("bool", &TclValue::Bool(true)).is_ok());
        assert!(validate_type("bool", &TclValue::Str("maybe".into())).is_err());
    }

    #[test]
    fn validate_type_nullable() {
        assert!(validate_type("int?", &TclValue::Str(String::new())).is_ok());
        assert!(validate_type("int?", &TclValue::Int(42)).is_ok());
    }

    #[test]
    fn validate_type_enum() {
        assert!(validate_type("enum {a b c}", &TclValue::Str("a".into())).is_ok());
        assert!(validate_type("enum {a b c}", &TclValue::Str("z".into())).is_err());
    }
}
