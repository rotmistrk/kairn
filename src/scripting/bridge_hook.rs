//! Hook namespace — register/fire event hooks with optional filters.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::hooks::{HookEvent, HookRegistry};

pub fn register(interp: &mut Interpreter, registry: Arc<Mutex<HookRegistry>>) {
    interp.register_fn("hook", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "add" => {
                let event_name = super::arg_str(args, 1)?;
                let event = HookEvent::parse_name(&event_name)
                    .ok_or_else(|| TclError::new(format!("unknown event: {event_name}")))?;
                // Parse optional -filter flag
                let (filter, body) = parse_add_args(args)?;
                let mut reg = registry.lock().map_err(|e| TclError::new(e.to_string()))?;
                reg.add(event, filter.as_deref(), body).map_err(TclError::new)?;
                Ok(TclValue::Str(String::new()))
            }
            "remove" => {
                let event_name = super::arg_str(args, 1)?;
                let event = HookEvent::parse_name(&event_name)
                    .ok_or_else(|| TclError::new(format!("unknown event: {event_name}")))?;
                let mut reg = registry.lock().map_err(|e| TclError::new(e.to_string()))?;
                reg.remove(&event);
                Ok(TclValue::Str(String::new()))
            }
            "list" => {
                let filter = super::arg_opt(args, 1).and_then(|s| HookEvent::parse_name(&s));
                let reg = registry.lock().map_err(|e| TclError::new(e.to_string()))?;
                let items: Vec<TclValue> = reg.list(filter.as_ref()).into_iter().map(TclValue::Str).collect();
                Ok(TclValue::List(items))
            }
            other => Err(TclError::new(format!("hook: unknown subcommand '{other}'"))),
        }
    });
}

/// Parse `hook add <event> ?-filter <pat>? <body>` arguments.
fn parse_add_args(args: &[TclValue]) -> Result<(Option<String>, String), TclError> {
    // args[0] = "add", args[1] = event, rest = [-filter pat] body
    let remaining = &args[2..];
    if remaining.is_empty() {
        return Err(TclError::new("hook add: missing body"));
    }
    if remaining.len() >= 3 && remaining[0].as_str() == "-filter" {
        let filter = remaining[1].as_str().into_owned();
        let body = remaining[2].as_str().into_owned();
        Ok((Some(filter), body))
    } else if remaining.len() >= 2 {
        // Could be: filter body (without -filter flag, positional)
        // Or just: body (single arg)
        // Check if first arg looks like a filter (has -filter prefix)
        let first = remaining[0].as_str();
        if first == "-filter" {
            return Err(TclError::new("hook add: -filter requires a pattern argument"));
        }
        // Two positional args: filter body
        let filter = remaining[0].as_str().into_owned();
        let body = remaining[1].as_str().into_owned();
        Ok((Some(filter), body))
    } else {
        // Single arg: body only
        let body = remaining[0].as_str().into_owned();
        Ok((None, body))
    }
}
