//! Info commands: info commands/vars/procs.

use crate::error::TclError;
use crate::interpreter::Interpreter;
use crate::value::TclValue;

/// Register info commands.
pub fn register(interp: &mut Interpreter) {
    interp.register_fn("info", cmd_info);
}

/// `info subcommand ?pattern?`
fn cmd_info(interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
    if args.is_empty() {
        return Err(TclError::new(
            "wrong # args: should be \"info subcommand ...\"",
        ));
    }
    let subcmd = args[0].as_str().to_string();
    let pattern = args.get(1).map(|a| a.as_str().to_string());
    match subcmd.as_str() {
        "commands" => info_commands(interp, pattern.as_deref()),
        "vars" => info_vars(interp, pattern.as_deref()),
        "procs" => info_procs(interp, pattern.as_deref()),
        _ => Err(TclError::new(format!(
            "unknown info subcommand \"{subcmd}\""
        ))),
    }
}

/// List registered commands.
fn info_commands(interp: &Interpreter, pattern: Option<&str>) -> Result<TclValue, TclError> {
    let mut names: Vec<String> = interp.commands.keys().cloned().collect();
    names.extend(interp.procs.keys().cloned());
    names.sort();
    if let Some(pat) = pattern {
        names.retain(|n| simple_glob(pat, n));
    }
    Ok(TclValue::List(
        names.into_iter().map(TclValue::Str).collect(),
    ))
}

/// List variables in current scope.
fn info_vars(interp: &Interpreter, pattern: Option<&str>) -> Result<TclValue, TclError> {
    let mut names: Vec<String> = if let Some(scope) = interp.scopes.last() {
        scope.vars.keys().cloned().collect()
    } else {
        Vec::new()
    };
    names.sort();
    if let Some(pat) = pattern {
        names.retain(|n| simple_glob(pat, n));
    }
    Ok(TclValue::List(
        names.into_iter().map(TclValue::Str).collect(),
    ))
}

/// List defined procedures.
fn info_procs(interp: &Interpreter, pattern: Option<&str>) -> Result<TclValue, TclError> {
    let mut names: Vec<String> = interp.procs.keys().cloned().collect();
    names.sort();
    if let Some(pat) = pattern {
        names.retain(|n| simple_glob(pat, n));
    }
    Ok(TclValue::List(
        names.into_iter().map(TclValue::Str).collect(),
    ))
}

/// Simple glob matching for info commands.
fn simple_glob(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }
    pattern == text
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn info_commands_lists() {
        let mut interp = Interpreter::new();
        let result = interp.eval("info commands").unwrap();
        assert!(!result.as_str().is_empty());
    }

    #[test]
    fn info_vars() {
        let mut interp = Interpreter::new();
        interp.eval("set x 1").unwrap();
        let result = interp.eval("info vars").unwrap();
        assert!(result.as_str().contains("x"));
    }

    #[test]
    fn info_procs() {
        let mut interp = Interpreter::new();
        interp.eval("proc foo {} {}").unwrap();
        let result = interp.eval("info procs").unwrap();
        assert!(result.as_str().contains("foo"));
    }
}
