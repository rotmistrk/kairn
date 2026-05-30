//! System namespace — exec, env, clipboard, platform.

use std::env;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::clipboard::{copy_to_clipboard, paste_from_clipboard};

use super::StateSnapshot;

pub fn register(interp: &mut Interpreter, snapshot: Arc<Mutex<StateSnapshot>>) {
    let snap = snapshot;
    interp.register_fn("system", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        handle_system_cmd(&snap, args, &sub)
    });
}

fn handle_system_cmd(snap: &Arc<Mutex<StateSnapshot>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "exec" => handle_exec(args),
        "env" => {
            let var = super::arg_str(args, 1)?;
            Ok(TclValue::Str(env::var(&var).unwrap_or_default()))
        }
        "set-env" => {
            let var = super::arg_str(args, 1)?;
            let val = super::arg_str(args, 2)?;
            env::set_var(&var, &val);
            Ok(TclValue::Str(String::new()))
        }
        "root-dir" => {
            let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
            Ok(TclValue::Str(s.root_dir.clone()))
        }
        "home-dir" => Ok(TclValue::Str(env::var("HOME").unwrap_or_default())),
        "platform" => Ok(TclValue::Str(platform_name().into())),
        "clipboard-get" => {
            let text = paste_from_clipboard().map_err(TclError::new)?;
            Ok(TclValue::Str(text))
        }
        "clipboard-set" => {
            let text = super::arg_str(args, 1)?;
            copy_to_clipboard(&text).map_err(TclError::new)?;
            Ok(TclValue::Str(String::new()))
        }
        _ => handle_title_cmds(snap, args, sub),
    }
}

fn handle_title_cmds(snap: &Arc<Mutex<StateSnapshot>>, args: &[TclValue], sub: &str) -> Result<TclValue, TclError> {
    match sub {
        "user" => Ok(TclValue::Str(env::var("USER").unwrap_or_default())),
        "hostname" => {
            let n = args
                .get(1)
                .and_then(|v| v.to_string().parse::<usize>().ok())
                .unwrap_or(0);
            Ok(TclValue::Str(short_hostname(n)))
        }
        "short-pwd" => {
            let max = args
                .get(1)
                .and_then(|v| v.to_string().parse::<usize>().ok())
                .unwrap_or(30);
            let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
            Ok(TclValue::Str(short_path(&s.root_dir, max)))
        }
        "busy" => {
            let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
            let indicator = if s.busy_count > 0 {
                "*"
            } else {
                ""
            };
            Ok(TclValue::Str(indicator.into()))
        }
        other => Err(TclError::new(format!("system: unknown subcommand '{other}'"))),
    }
}

fn handle_exec(args: &[TclValue]) -> Result<TclValue, TclError> {
    let command = super::arg_str(args, 1)?;
    let output = Command::new("sh")
        .args(["-c", &command])
        .output()
        .map_err(|e| TclError::new(format!("exec: {e}")))?;
    if output.status.success() {
        Ok(TclValue::Str(
            String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
        ))
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim_end().to_string();
        Err(TclError::new(format!("exec failed: {err}")))
    }
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

/// Get hostname, optionally truncated to first N domain components.
/// N=0 means full, N=1 means just the first part before '.'.
fn short_hostname(components: usize) -> String {
    let full = env::var("HOSTNAME")
        .or_else(|_| fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_default();
    if components == 0 {
        return full;
    }
    full.splitn(components + 1, '.')
        .take(components)
        .collect::<Vec<_>>()
        .join(".")
}

/// Smart-truncate a path: replace $HOME with ~, then shorten middle if over max.
fn short_path(path: &str, max: usize) -> String {
    let home = env::var("HOME").unwrap_or_default();
    let display = if !home.is_empty() && path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    };
    if display.len() <= max {
        return display;
    }
    // Keep first component and last part, join with /…/
    let parts: Vec<&str> = display.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 2 {
        return format!("…{}", &display[display.len().saturating_sub(max - 1)..]);
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    let prefix = if display.starts_with('~') {
        ""
    } else {
        "/"
    };
    let short = format!("{}{}/…/{}", prefix, first, last);
    if short.len() <= max {
        short
    } else {
        format!("…{}", &display[display.len().saturating_sub(max - 1)..])
    }
}
