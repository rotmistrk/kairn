//! Window and app bridge commands.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::layout_mgr::Side;

use super::{opt_val, parse_opts, require_arg, SendShared};

pub fn register_window(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("window", move |_interp, args| {
        let sub = require_arg(args, 0, "window")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let title = args.get(1).map(|v| v.as_str().into_owned()).unwrap_or_default();
                st.desktop.layout.set_title(&title);
                Ok(TclValue::Str("window_0".into()))
            }
            "add" => {
                let _win = require_arg(args, 1, "window add")?;
                let widget_id = require_arg(args, 2, "window add")?;
                let opts = parse_opts(args, 3);
                let side = opt_val(&opts, "-side")
                    .map(Side::parse)
                    .transpose()
                    .map_err(TclError::new)?
                    .unwrap_or(Side::Fill);
                let size = opt_val(&opts, "-width")
                    .or_else(|| opt_val(&opts, "-height"))
                    .and_then(|v| v.parse::<u16>().ok());
                st.desktop.layout.add(&widget_id, side, size);
                Ok(TclValue::Str(String::new()))
            }
            "title" => {
                let _win = require_arg(args, 1, "window title")?;
                let title = require_arg(args, 2, "window title")?;
                st.desktop.layout.set_title(&title);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("window: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_app(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("app", move |_interp, args| {
        let sub = require_arg(args, 0, "app")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "run" => {
                st.run_requested = true;
                Ok(TclValue::Str(String::new()))
            }
            "quit" => {
                st.events.quit_requested = true;
                Ok(TclValue::Str(String::new()))
            }
            "on-quit" => {
                let script = require_arg(args, 1, "app on-quit")?;
                st.events.quit_handler = Some(script);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("app: unknown subcommand {sub}"))),
        }
    });
}
