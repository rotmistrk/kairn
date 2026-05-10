//! Utility bridge commands: dialog, menu, fuzzy-select, bind, after, focus, notify, files.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use txv_widgets::{Menu, MenuItem};

use crate::desktop::TkDesktop;

use super::{opt_val, parse_opts, require_arg, tcl_to_string_list, SendShared};

pub fn register_dialog(interp: &mut Interpreter) {
    interp.register_fn("dialog", move |_interp, args| {
        let sub = require_arg(args, 0, "dialog")?;
        match sub.as_str() {
            "confirm" => Ok(TclValue::Bool(false)),
            "prompt" => {
                let default = args.get(2).map(|v| v.as_str().into_owned()).unwrap_or_default();
                Ok(TclValue::Str(default))
            }
            "info" | "error" => Ok(TclValue::Str(String::new())),
            _ => Err(TclError::new(format!("dialog: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_menu(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("menu", move |_interp, args| {
        let sub = require_arg(args, 0, "menu")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let items_val = args.get(1).ok_or_else(|| TclError::new("menu create: missing items"))?;
                let items_list = items_val.as_list()?;
                let items: Vec<MenuItem> = items_list
                    .iter()
                    .map(|v| MenuItem::new(v.as_str().into_owned(), 0))
                    .collect();
                let id = st.alloc_id();
                let menu = Menu::new(items);
                st.desktop.insert_widget(id.clone(), Box::new(menu));
                Ok(TclValue::Str(id))
            }
            "show" => Ok(TclValue::Str(String::new())),
            _ => Err(TclError::new(format!("menu: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_fuzzy_select(interp: &mut Interpreter) {
    interp.register_fn("fuzzy-select", move |_interp, args| {
        let items_val = args
            .first()
            .ok_or_else(|| TclError::new("fuzzy-select: missing items"))?;
        let items = tcl_to_string_list(items_val)?;
        Ok(TclValue::Str(items.into_iter().next().unwrap_or_default()))
    });
}

pub fn register_bind(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("bind", move |_interp, args| {
        let keyspec = require_arg(args, 0, "bind")?;
        let script = require_arg(args, 1, "bind")?;
        let mut st = s.lock()?;
        st.events.key_bindings.insert(keyspec, script);
        Ok(TclValue::Str(String::new()))
    });
}

pub fn register_after(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("after", move |_interp, args| {
        let ms = require_arg(args, 0, "after")?
            .parse::<u64>()
            .map_err(|_| TclError::new("after: invalid delay"))?;
        let mut script_idx = 1;
        let mut repeat = false;
        if let Some(flag) = args.get(1) {
            if flag.as_str() == "-repeat" {
                repeat = true;
                script_idx = 2;
            }
        }
        let script = require_arg(args, script_idx, "after")?;
        let mut st = s.lock()?;
        st.events.timers.push(crate::event_mgr::TimerDef {
            delay_ms: ms,
            repeat,
            script,
        });
        Ok(TclValue::Str(String::new()))
    });
}

pub fn register_focus(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("focus", move |_interp, args| {
        let id = require_arg(args, 0, "focus")?;
        let mut st = s.lock()?;
        st.desktop.focus(&id);
        Ok(TclValue::Str(String::new()))
    });
}

pub fn register_notify(interp: &mut Interpreter) {
    interp.register_fn("notify", move |_interp, _args| Ok(TclValue::Str(String::new())));
}

pub fn register_files(interp: &mut Interpreter) {
    interp.register_fn("files", move |_interp, args| {
        let path = require_arg(args, 0, "files")?;
        let opts = parse_opts(args, 1);
        let recursive = opts.iter().any(|(k, _)| k == "-recursive");
        let filter = opt_val(&opts, "-filter").map(|s| s.to_string());

        let walker = ignore::WalkBuilder::new(&path)
            .max_depth(if recursive {
                None
            } else {
                Some(1)
            })
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        let mut entries = Vec::new();
        for result in walker {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };
            let p = entry.path();
            if p == std::path::Path::new(&path) {
                continue;
            }
            let name = p.display().to_string();
            if let Some(ref pat) = filter {
                let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !glob_match(pat, fname) {
                    continue;
                }
            }
            entries.push(TclValue::Str(name));
        }
        Ok(TclValue::List(entries))
    });
}

fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        name == pattern
    }
}

/// Get a mutable reference to a widget by name, downcasting via `as_any_mut`.
pub fn get_widget<'a, T: 'static>(desktop: &'a mut TkDesktop, name: &str, cmd: &str) -> Result<&'a mut T, TclError> {
    let view = desktop
        .get_mut(name)
        .ok_or_else(|| TclError::new(format!("{cmd}: no widget {name}")))?;
    let any = view
        .as_any_mut()
        .ok_or_else(|| TclError::new(format!("{cmd}: widget {name} not downcastable")))?;
    any.downcast_mut::<T>()
        .ok_or_else(|| TclError::new(format!("{cmd}: widget {name} type mismatch")))
}
