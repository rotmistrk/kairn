//! Tree, input, and script-statusbar bridge commands.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use txv_widgets::{FileTreeData, InputLine, TextArea, TreeView};

use super::{get_widget, opt_val, parse_opts, require_arg, SendShared};

pub fn register_tree(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("tree", move |_interp, args| {
        let sub = require_arg(args, 0, "tree")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let path = opt_val(&opts, "-data").unwrap_or(".");
                let id = st.alloc_id();
                let data = FileTreeData::new(std::path::PathBuf::from(path));
                let tv = TreeView::new(data);
                st.desktop.insert_widget(id.clone(), Box::new(tv));
                Ok(TclValue::Str(id))
            }
            "selected" => {
                let id = require_arg(args, 1, "tree selected")?;
                let tv = get_widget::<TreeView<FileTreeData>>(&mut st.desktop, &id, "tree selected")?;
                let path = tv.data.path(tv.cursor).display().to_string();
                Ok(TclValue::Str(path))
            }
            "refresh" => {
                let id = require_arg(args, 1, "tree refresh")?;
                let tv = get_widget::<TreeView<FileTreeData>>(&mut st.desktop, &id, "tree refresh")?;
                tv.data.refresh();
                Ok(TclValue::Str(String::new()))
            }
            "on-select" | "on-activate" => {
                let id = require_arg(args, 1, &format!("tree {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("tree {sub}"))?;
                st.events.widget_handlers.insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("tree: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_input(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("input", move |_interp, args| {
        let sub = require_arg(args, 0, "input")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let id = st.alloc_id();
                let mut inp = InputLine::new();
                if let Some(default) = opt_val(&opts, "-default") {
                    inp.set_text(default);
                }
                st.desktop.insert_widget(id.clone(), Box::new(inp));
                Ok(TclValue::Str(id))
            }
            "get" => {
                let id = require_arg(args, 1, "input get")?;
                let inp = get_widget::<InputLine>(&mut st.desktop, &id, "input get")?;
                Ok(TclValue::Str(inp.text.clone()))
            }
            "set" => {
                let id = require_arg(args, 1, "input set")?;
                let text = require_arg(args, 2, "input set")?;
                let inp = get_widget::<InputLine>(&mut st.desktop, &id, "input set")?;
                inp.set_text(&text);
                Ok(TclValue::Str(String::new()))
            }
            "clear" => {
                let id = require_arg(args, 1, "input clear")?;
                let inp = get_widget::<InputLine>(&mut st.desktop, &id, "input clear")?;
                inp.clear();
                Ok(TclValue::Str(String::new()))
            }
            "focus" => {
                let id = require_arg(args, 1, "input focus")?;
                st.desktop.focus(&id);
                Ok(TclValue::Str(String::new()))
            }
            "on-change" | "on-submit" => {
                let id = require_arg(args, 1, &format!("input {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("input {sub}"))?;
                st.events.widget_handlers.insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("input: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_statusbar(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("statusbar", move |_interp, args| {
        let sub = require_arg(args, 0, "statusbar")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st.alloc_id();
                let mut ta = TextArea::new();
                ta.line_numbers = false;
                st.desktop.insert_widget(id.clone(), Box::new(ta));
                Ok(TclValue::Str(id))
            }
            "left" | "right" => {
                let id = require_arg(args, 1, &format!("statusbar {sub}"))?;
                let text = require_arg(args, 2, &format!("statusbar {sub}"))?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, &format!("statusbar {sub}"))?;
                ta.set_content(&text);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("statusbar: unknown subcommand {sub}"))),
        }
    });
}
