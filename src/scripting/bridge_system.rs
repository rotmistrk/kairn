//! System namespace — exec, env, clipboard, platform.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::StateSnapshot;

pub fn register(interp: &mut Interpreter, snapshot: Arc<Mutex<StateSnapshot>>) {
    let snap = snapshot;
    interp.register_fn("system", move |_interp, args| {
        let sub = super::arg_str(args, 0)?;
        match sub.as_str() {
            "exec" => {
                let command = super::arg_str(args, 1)?;
                let output = std::process::Command::new("sh")
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
            "env" => {
                let var = super::arg_str(args, 1)?;
                let val = std::env::var(&var).unwrap_or_default();
                Ok(TclValue::Str(val))
            }
            "set-env" => {
                let var = super::arg_str(args, 1)?;
                let val = super::arg_str(args, 2)?;
                std::env::set_var(&var, &val);
                Ok(TclValue::Str(String::new()))
            }
            "root-dir" => {
                let s = snap.lock().map_err(|e| TclError::new(e.to_string()))?;
                Ok(TclValue::Str(s.root_dir.clone()))
            }
            "home-dir" => {
                let home = std::env::var("HOME").unwrap_or_default();
                Ok(TclValue::Str(home))
            }
            "platform" => {
                let p = if cfg!(target_os = "macos") {
                    "macos"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else {
                    "unknown"
                };
                Ok(TclValue::Str(p.into()))
            }
            "clipboard-get" => {
                let text = crate::clipboard::paste_from_clipboard().map_err(TclError::new)?;
                Ok(TclValue::Str(text))
            }
            "clipboard-set" => {
                let text = super::arg_str(args, 1)?;
                crate::clipboard::copy_to_clipboard(&text).map_err(TclError::new)?;
                Ok(TclValue::Str(String::new()))
            }
            other => Err(TclError::new(format!("system: unknown subcommand '{other}'"))),
        }
    });
}
