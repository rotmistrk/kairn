//! System namespace — exec, env, clipboard, platform.

use std::env;
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
