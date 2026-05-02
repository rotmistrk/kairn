//! Dict commands: dict create/get/set/exists/keys/values/size/for.

use crate::error::{ErrorCode, TclError};
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register dict commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("dict", cmd_dict);
}

/// `dict subcommand args...`
fn cmd_dict(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"dict subcommand ...\"",
        ));
    }
    let subcmd = args[0].as_str().to_string();
    let rest = &args[1..];
    match subcmd.as_str() {
        "create" => dict_create(rest),
        "get" => dict_get(rest),
        "set" => dict_set(interp, rest),
        "exists" => dict_exists(rest),
        "keys" => dict_keys(rest),
        "values" => dict_values(rest),
        "size" => dict_size(rest),
        "for" => dict_for(interp, rest),
        _ => Err(TclError::new(format!(
            "unknown dict subcommand \"{subcmd}\""
        ))),
    }
}

/// `dict create key val ...`
fn dict_create(args: &[TclValue]) -> Result<TclValue, TclError> {
    let mut pairs = Vec::new();
    let mut i = 0;
    while i + 1 < args.len() {
        let key = args[i].as_str().to_string();
        let val = args[i + 1].clone();
        pairs.push((key, val));
        i += 2;
    }
    Ok(TclValue::Dict(pairs))
}

/// `dict get dict key ?key ...?`
fn dict_get(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"dict get dict key\"",
        ));
    }
    let mut current = args[0].clone();
    for key_arg in &args[1..] {
        let key = key_arg.as_str().to_string();
        current = match &current {
            TclValue::Dict(pairs) => pairs
                .iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| TclError::new(format!("key \"{key}\" not known in dictionary")))?,
            _ => {
                // Try to parse as flat list
                let list = current.as_list()?;
                let mut found = None;
                let mut i = 0;
                while i + 1 < list.len() {
                    if list[i].as_str() == key {
                        found = Some(list[i + 1].clone());
                        break;
                    }
                    i += 2;
                }
                found.ok_or_else(|| {
                    TclError::new(format!("key \"{key}\" not known in dictionary"))
                })?
            }
        };
    }
    Ok(current)
}

/// `dict set dictvar key value`
fn dict_set(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"dict set dictVar key value\"",
        ));
    }
    let var_name = args[0].as_str().to_string();
    let key = args[1].as_str().to_string();
    let value = args[2].clone();
    let mut pairs = match interp.get_var(&var_name) {
        Some(TclValue::Dict(d)) => d.clone(),
        Some(other) => {
            let list = other.as_list()?;
            let mut p = Vec::new();
            let mut i = 0;
            while i + 1 < list.len() {
                p.push((list[i].as_str().to_string(), list[i + 1].clone()));
                i += 2;
            }
            p
        }
        None => Vec::new(),
    };
    // Update or insert
    let mut found = false;
    for pair in &mut pairs {
        if pair.0 == key {
            pair.1 = value.clone();
            found = true;
            break;
        }
    }
    if !found {
        pairs.push((key, value));
    }
    let val = TclValue::Dict(pairs);
    interp.set_var(&var_name, val.clone());
    Ok(val)
}

/// `dict exists dict key`
fn dict_exists(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new(
            "wrong # args: should be \"dict exists dict key\"",
        ));
    }
    let key = args[1].as_str().to_string();
    let exists = match &args[0] {
        TclValue::Dict(pairs) => pairs.iter().any(|(k, _)| k == &key),
        _ => false,
    };
    Ok(TclValue::Bool(exists))
}

/// `dict keys dict`
fn dict_keys(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"dict keys dict\""));
    }
    match &args[0] {
        TclValue::Dict(pairs) => Ok(TclValue::List(
            pairs
                .iter()
                .map(|(k, _)| TclValue::Str(k.clone()))
                .collect(),
        )),
        _ => Err(TclError::new("expected dict")),
    }
}

/// `dict values dict`
fn dict_values(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"dict values dict\"",
        ));
    }
    match &args[0] {
        TclValue::Dict(pairs) => Ok(TclValue::List(
            pairs.iter().map(|(_, v)| v.clone()).collect(),
        )),
        _ => Err(TclError::new("expected dict")),
    }
}

/// `dict size dict`
fn dict_size(args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"dict size dict\""));
    }
    match &args[0] {
        TclValue::Dict(pairs) => Ok(TclValue::Int(pairs.len() as i64)),
        _ => Err(TclError::new("expected dict")),
    }
}

/// `dict for {k v} dict body`
fn dict_for(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"dict for {k v} dict body\"",
        ));
    }
    let var_spec = args[0].as_str().to_string();
    let vars: Vec<&str> = var_spec.split_whitespace().collect();
    if vars.len() != 2 {
        return Err(TclError::new("dict for: need exactly 2 variable names"));
    }
    let pairs = match &args[1] {
        TclValue::Dict(d) => d.clone(),
        other => {
            let list = other.as_list()?;
            let mut p = Vec::new();
            let mut i = 0;
            while i + 1 < list.len() {
                p.push((list[i].as_str().to_string(), list[i + 1].clone()));
                i += 2;
            }
            p
        }
    };
    let body = args[2].as_str().to_string();
    for (k, v) in &pairs {
        interp.set_var(vars[0], TclValue::Str(k.clone()));
        interp.set_var(vars[1], v.clone());
        match interp.eval(&body) {
            Ok(_) => {}
            Err(e) if e.code == ErrorCode::Break => break,
            Err(e) if e.code == ErrorCode::Continue => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(TclValue::Str(String::new()))
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn dict_create_and_get() {
        let mut interp = Interpreter::new();
        interp
            .eval("set d [dict create name alice age 30]")
            .unwrap();
        assert_eq!(interp.eval("dict get $d name").unwrap().as_str(), "alice");
        assert_eq!(interp.eval("dict get $d age").unwrap().as_str(), "30");
    }

    #[test]
    fn dict_set() {
        let mut interp = Interpreter::new();
        interp.eval("set d [dict create name alice]").unwrap();
        interp.eval("dict set d age 30").unwrap();
        assert_eq!(interp.eval("dict get $d age").unwrap().as_str(), "30");
    }

    #[test]
    fn dict_exists() {
        let mut interp = Interpreter::new();
        interp.eval("set d [dict create name alice]").unwrap();
        assert_eq!(interp.eval("dict exists $d name").unwrap().as_str(), "1");
        assert_eq!(interp.eval("dict exists $d age").unwrap().as_str(), "0");
    }

    #[test]
    fn dict_keys_values_size() {
        let mut interp = Interpreter::new();
        interp.eval("set d [dict create a 1 b 2]").unwrap();
        assert_eq!(interp.eval("dict size $d").unwrap().as_str(), "2");
        assert_eq!(interp.eval("dict keys $d").unwrap().as_str(), "a b");
        assert_eq!(interp.eval("dict values $d").unwrap().as_str(), "1 2");
    }

    #[test]
    fn dict_for() {
        let mut interp = Interpreter::new();
        interp.eval("set d [dict create a 1 b 2]").unwrap();
        interp.eval("set result {}").unwrap();
        interp
            .eval("dict for {k v} $d {append result \"$k=$v \"}")
            .unwrap();
        let result = interp.eval("set result").unwrap();
        assert!(result.as_str().contains("a=1"));
        assert!(result.as_str().contains("b=2"));
    }
}
