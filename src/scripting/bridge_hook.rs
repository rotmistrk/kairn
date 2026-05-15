//! Hook namespace — register/fire event hooks.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

/// Hook storage: event name → list of script bodies.
static HOOKS: std::sync::LazyLock<std::sync::Mutex<Vec<(String, String)>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

pub fn register(interp: &mut Interpreter) {
    interp.register_fn("hook", |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "add" => {
                let event = super::arg_str(args, 1)?;
                let body = super::arg_str(args, 2)?;
                if let Ok(mut hooks) = HOOKS.lock() {
                    hooks.push((event, body));
                }
                Ok(TclValue::Str(String::new()))
            }
            "remove" => {
                let event = super::arg_str(args, 1)?;
                if let Ok(mut hooks) = HOOKS.lock() {
                    hooks.retain(|(e, _)| e != &event);
                }
                Ok(TclValue::Str(String::new()))
            }
            "list" => {
                let filter = super::arg_opt(args, 1);
                let hooks = HOOKS.lock().map_err(|e| TclError::new(e.to_string()))?;
                let items: Vec<TclValue> = hooks
                    .iter()
                    .filter(|(e, _)| filter.as_ref().is_none_or(|f| e == f))
                    .map(|(e, b)| TclValue::Str(format!("{e}: {b}")))
                    .collect();
                Ok(TclValue::List(items))
            }
            other => Err(TclError::new(format!("hook: unknown subcommand '{other}'"))),
        }
    });
}

/// Fire all hooks for a given event. Returns scripts to evaluate.
pub fn hooks_for_event(event: &str) -> Vec<String> {
    if let Ok(hooks) = HOOKS.lock() {
        hooks
            .iter()
            .filter(|(e, _)| e == event)
            .map(|(_, body)| body.clone())
            .collect()
    } else {
        Vec::new()
    }
}
