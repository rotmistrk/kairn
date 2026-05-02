//! Variable commands: set, unset, outer, append.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register variable commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("set", cmd_set);
    interp.register_fn("unset", cmd_unset);
    interp.register_fn("outer", cmd_outer);
    interp.register_fn("append", cmd_append);
    interp.register_fn("incr", cmd_incr);
}

/// `set var ?value?` or `set var = value` or `set a, b, c = list`
fn cmd_set(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"set varName ?newValue?\"",
        ));
    }

    // Find `=` to detect modern assignment form
    let eq_pos = args.iter().position(|a| a.as_str() == "=");

    if let Some(eq_idx) = eq_pos {
        return handle_equals_form(interp, args, eq_idx);
    }

    // Classic Tcl form: set var value
    let name = args[0].as_str().to_string();

    // Legacy brace destructuring: set {a, b, c} value
    if name.starts_with('{') && name.ends_with('}') && name.contains(',') {
        return destructure_dict(interp, &name, args);
    }

    if args.len() == 1 {
        interp
            .get_var(&name)
            .cloned()
            .ok_or_else(|| TclError::new(format!("can't read \"{name}\": no such variable")))
    } else {
        let value = args[1].clone();
        interp.try_set_var(&name, value.clone())?;
        Ok(value)
    }
}

/// Handle `set var = value` and `set a, b, c = list`
fn handle_equals_form(
    interp: &mut Interpreter,
    args: &[TclValue],
    eq_idx: usize,
) -> Result<TclValue, TclError> {
    if eq_idx == 0 || eq_idx + 1 >= args.len() {
        return Err(TclError::new(
            "wrong # args: should be \"set var = value\" or \"set a, b = list\"",
        ));
    }

    let value = args[eq_idx + 1].clone();

    // Collect variable names before `=`
    // "set a, b, c = list" → args = ["a,", "b,", "c", "=", list]
    // or "set x = 42" → args = ["x", "=", "42"]
    let mut names: Vec<String> = Vec::new();
    for arg in &args[..eq_idx] {
        let s = arg.as_str().to_string();
        // Strip trailing comma
        for part in s.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                names.push(trimmed.to_string());
            }
        }
    }

    if names.is_empty() {
        return Err(TclError::new("set: no variable names before '='"));
    }

    if names.len() == 1 {
        // Simple: set x = 42
        interp.try_set_var(&names[0], value.clone())?;
        return Ok(value);
    }

    // Destructuring: set a, b, c = [list 1 2 3]
    let list = value.as_list()?;
    for (i, name) in names.iter().enumerate() {
        let val = list.get(i).cloned().unwrap_or(TclValue::Str(String::new()));
        interp.set_var(name, val);
    }
    Ok(args[eq_idx + 1].clone())
}

/// Destructure a dict: `set {name, age} value` (legacy form).
fn destructure_dict(
    interp: &mut Interpreter,
    pattern: &str,
    args: &[TclValue],
) -> Result<TclValue, TclError> {
    if args.len() < 2 {
        return Err(TclError::new("wrong # args for destructuring"));
    }
    let inner = &pattern[1..pattern.len() - 1];
    let names: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    let dict = match &args[1] {
        TclValue::Dict(d) => d.clone(),
        other => {
            let list = other.as_list()?;
            let mut pairs = Vec::new();
            let mut i = 0;
            while i + 1 < list.len() {
                pairs.push((list[i].as_str().to_string(), list[i + 1].clone()));
                i += 2;
            }
            pairs
        }
    };
    for name in &names {
        let val = dict
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or(TclValue::Str(String::new()));
        interp.set_var(name, val);
    }
    Ok(args[1].clone())
}

/// `unset var` — remove a variable.
fn cmd_unset(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new("wrong # args: should be \"unset varName\""));
    }
    let name = args[0].as_str();
    interp.unset_var(&name);
    Ok(TclValue::Str(String::new()))
}

/// `outer set var value` — set variable in parent scope.
fn cmd_outer(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.len() < 3 {
        return Err(TclError::new(
            "wrong # args: should be \"outer set varName value\"",
        ));
    }
    let subcmd = args[0].as_str();
    if subcmd != "set" {
        return Err(TclError::new(format!(
            "outer: unknown subcommand \"{subcmd}\""
        )));
    }
    let name = args[1].as_str().to_string();
    let value = args[2].clone();
    interp.set_var_in_parent(&name, value.clone(), 1)?;
    Ok(value)
}

/// `append var string` — append to a variable.
fn cmd_append(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"append varName ?value ...?\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let mut current = interp
        .get_var(&name)
        .map(|v| v.as_str().to_string())
        .unwrap_or_default();
    for arg in &args[1..] {
        current.push_str(&arg.as_str());
    }
    let val = TclValue::Str(current);
    interp.set_var(&name, val.clone());
    Ok(val)
}

/// `incr var ?amount?` — increment an integer variable.
fn cmd_incr(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"incr varName ?increment?\"",
        ));
    }
    let name = args[0].as_str().to_string();
    let amount = if args.len() > 1 { args[1].as_int()? } else { 1 };
    let current = interp.get_var(&name).map(|v| v.as_int()).unwrap_or(Ok(0))?;
    let new_val = TclValue::Int(current + amount);
    interp.set_var(&name, new_val.clone());
    Ok(new_val)
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn set_and_get() {
        let mut interp = Interpreter::new();
        interp.eval("set x 42").unwrap();
        let result = interp.eval("set x").unwrap();
        assert_eq!(result.as_str(), "42");
    }

    #[test]
    fn set_with_equals() {
        let mut interp = Interpreter::new();
        interp.eval("set x = 42").unwrap();
        let result = interp.eval("set x").unwrap();
        assert_eq!(result.as_str(), "42");
    }

    #[test]
    fn destructure_with_equals() {
        let mut interp = Interpreter::new();
        interp.eval("set x [list 10 20 30]").unwrap();
        interp.eval("set a, b, c = $x").unwrap();
        assert_eq!(interp.eval("set a").unwrap().as_str(), "10");
        assert_eq!(interp.eval("set b").unwrap().as_str(), "20");
        assert_eq!(interp.eval("set c").unwrap().as_str(), "30");
    }

    #[test]
    fn unset_removes_var() {
        let mut interp = Interpreter::new();
        interp.eval("set x 42").unwrap();
        interp.eval("unset x").unwrap();
        assert!(interp.eval("set x").is_err());
    }

    #[test]
    fn append_to_var() {
        let mut interp = Interpreter::new();
        interp.eval("set x hello").unwrap();
        interp.eval("append x \" world\"").unwrap();
        let result = interp.eval("set x").unwrap();
        assert_eq!(result.as_str(), "hello world");
    }

    #[test]
    fn incr_default() {
        let mut interp = Interpreter::new();
        interp.eval("set x 10").unwrap();
        let result = interp.eval("incr x").unwrap();
        assert_eq!(result.as_str(), "11");
    }

    #[test]
    fn incr_by_amount() {
        let mut interp = Interpreter::new();
        interp.eval("set x 10").unwrap();
        let result = interp.eval("incr x 5").unwrap();
        assert_eq!(result.as_str(), "15");
    }

    #[test]
    fn variable_substitution() {
        let mut interp = Interpreter::new();
        interp.eval("set name world").unwrap();
        let result = interp.eval("set greeting \"hello $name\"").unwrap();
        assert_eq!(result.as_str(), "hello world");
    }
}
