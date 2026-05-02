//! Scope chain for lexical scoping.

use std::collections::HashMap;

use crate::error::TclError;
use crate::value::TclValue;

/// A single scope in the scope chain.
#[derive(Clone, Debug)]
pub struct Scope {
    /// Variables in this scope.
    pub vars: HashMap<String, TclValue>,
    /// Optional parent scope index for lexical scoping.
    /// `None` means walk to the previous scope in the chain.
    pub parent: Option<usize>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }

    /// Create a scope linked to a specific parent.
    pub fn with_parent(parent: usize) -> Self {
        Self {
            vars: HashMap::new(),
            parent: Some(parent),
        }
    }
}

/// Set a variable in the current (top) scope.
pub fn set_var(scopes: &mut [Scope], name: &str, value: TclValue) {
    if let Some(scope) = scopes.last_mut() {
        scope.vars.insert(name.to_string(), value);
    }
}

/// Get a variable, walking up the scope chain (lexical scoping).
pub fn get_var<'a>(scopes: &'a [Scope], name: &str) -> Option<&'a TclValue> {
    let mut idx = scopes.len().checked_sub(1)?;
    loop {
        if let Some(val) = scopes[idx].vars.get(name) {
            return Some(val);
        }
        if let Some(parent) = scopes[idx].parent {
            if parent < idx {
                idx = parent;
                continue;
            }
        }
        if idx == 0 {
            return None;
        }
        idx -= 1;
    }
}

/// Remove a variable from the current scope.
pub fn unset_var(scopes: &mut [Scope], name: &str) -> bool {
    if let Some(scope) = scopes.last_mut() {
        return scope.vars.remove(name).is_some();
    }
    false
}

/// Set a variable in a parent scope (for `outer`).
pub fn set_var_in_parent(
    scopes: &mut [Scope],
    name: &str,
    value: TclValue,
    levels: usize,
) -> Result<(), TclError> {
    let len = scopes.len();
    if levels >= len {
        return Err(TclError::new("outer: no parent scope"));
    }
    let target = len - 1 - levels;
    scopes[target].vars.insert(name.to_string(), value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut scopes = vec![Scope::new()];
        set_var(&mut scopes, "x", TclValue::Int(42));
        assert_eq!(
            get_var(&scopes, "x").map(|v| v.as_str().to_string()),
            Some("42".to_string())
        );
    }

    #[test]
    fn scope_chain_lookup() {
        let mut scopes = vec![Scope::new()];
        set_var(&mut scopes, "x", TclValue::Int(10));
        scopes.push(Scope::new());
        // Child scope can see parent's variable
        assert_eq!(
            get_var(&scopes, "x").map(|v| v.as_str().to_string()),
            Some("10".to_string())
        );
    }

    #[test]
    fn child_shadows_parent() {
        let mut scopes = vec![Scope::new()];
        set_var(&mut scopes, "x", TclValue::Int(10));
        scopes.push(Scope::new());
        set_var(&mut scopes, "x", TclValue::Int(20));
        assert_eq!(
            get_var(&scopes, "x").map(|v| v.as_str().to_string()),
            Some("20".to_string())
        );
    }

    #[test]
    fn unset_removes() {
        let mut scopes = vec![Scope::new()];
        set_var(&mut scopes, "x", TclValue::Int(42));
        assert!(unset_var(&mut scopes, "x"));
        assert!(get_var(&scopes, "x").is_none());
    }

    #[test]
    fn set_in_parent() {
        let mut scopes = vec![Scope::new(), Scope::new()];
        set_var_in_parent(&mut scopes, "x", TclValue::Int(99), 1).unwrap();
        assert_eq!(
            scopes[0].vars.get("x").map(|v| v.as_str().to_string()),
            Some("99".to_string())
        );
    }

    #[test]
    fn linked_parent_scope() {
        let mut scopes = vec![Scope::new(), Scope::new(), Scope::new()];
        // Set x in scope 0
        scopes[0].vars.insert("x".to_string(), TclValue::Int(100));
        // Scope 2 links to scope 0 (skipping scope 1)
        scopes[2].parent = Some(0);
        assert_eq!(
            get_var(&scopes, "x").map(|v| v.as_str().to_string()),
            Some("100".to_string())
        );
    }
}
