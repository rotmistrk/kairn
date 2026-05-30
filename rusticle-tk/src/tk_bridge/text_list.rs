//! Text and list widget bridge commands.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use txv_widgets::{ListView, TextArea};

use crate::widget_mgr::StringListData;

use super::{get_widget, opt_val, parse_opts, require_arg, tcl_to_string_list, SendShared};

pub fn register_text(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("text", move |_interp, args| {
        let sub = require_arg(args, 0, "text")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let id = st.alloc_id();
                let mut ta = TextArea::new();
                if let Some(path) = opt_val(&opts, "-file") {
                    let content =
                        std::fs::read_to_string(path).map_err(|e| TclError::new(format!("text create: {e}")))?;
                    ta.set_content(&content);
                }
                if let Some(content) = opt_val(&opts, "-content") {
                    ta.set_content(content);
                }
                if let Some(val) = opt_val(&opts, "-linenumbers") {
                    ta.line_numbers = val == "true" || val == "1";
                }
                st.desktop.insert_widget(id.clone(), Box::new(ta));
                Ok(TclValue::Str(id))
            }
            "set" => {
                let id = require_arg(args, 1, "text set")?;
                let content = require_arg(args, 2, "text set")?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, "text set")?;
                ta.set_content(&content);
                Ok(TclValue::Str(String::new()))
            }
            "get" => {
                let id = require_arg(args, 1, "text get")?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, "text get")?;
                Ok(TclValue::Str(ta.lines.join("\n")))
            }
            "clear" => {
                let id = require_arg(args, 1, "text clear")?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, "text clear")?;
                ta.set_content("");
                Ok(TclValue::Str(String::new()))
            }
            "append" => {
                let id = require_arg(args, 1, "text append")?;
                let content = require_arg(args, 2, "text append")?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, "text append")?;
                for line in content.lines() {
                    ta.lines.push(line.to_string());
                }
                Ok(TclValue::Str(String::new()))
            }
            "line-numbers" => {
                let id = require_arg(args, 1, "text line-numbers")?;
                let val = require_arg(args, 2, "text line-numbers")?;
                let ta = get_widget::<TextArea>(&mut st.desktop, &id, "text line-numbers")?;
                ta.line_numbers = val == "true" || val == "1";
                Ok(TclValue::Str(String::new()))
            }
            _ => Ok(TclValue::Str(String::new())),
        }
    });
}

pub fn register_list(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("list", move |_interp, args| {
        let sub = require_arg(args, 0, "list")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st.alloc_id();
                let lv = ListView::new(StringListData::new(Vec::new()));
                st.desktop.insert_widget(id.clone(), Box::new(lv));
                Ok(TclValue::Str(id))
            }
            "set-items" => {
                let id = require_arg(args, 1, "list set-items")?;
                let items_val = args
                    .get(2)
                    .ok_or_else(|| TclError::new("list set-items: missing items"))?;
                let items = tcl_to_string_list(items_val)?;
                let lv = get_widget::<ListView<StringListData>>(&mut st.desktop, &id, "list set-items")?;
                *lv.data_mut() = StringListData::new(items);
                Ok(TclValue::Str(String::new()))
            }
            "selected" => {
                let id = require_arg(args, 1, "list selected")?;
                let lv = get_widget::<ListView<StringListData>>(&mut st.desktop, &id, "list selected")?;
                let text = lv.data().selected_text(lv.cursor());
                Ok(TclValue::Str(text))
            }
            "index" => {
                let id = require_arg(args, 1, "list index")?;
                let lv = get_widget::<ListView<StringListData>>(&mut st.desktop, &id, "list index")?;
                Ok(TclValue::Int(lv.cursor() as i64))
            }
            "on-select" | "on-activate" => {
                let id = require_arg(args, 1, &format!("list {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("list {sub}"))?;
                st.events.widget_handlers.insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("list: unknown subcommand {sub}"))),
        }
    });
}
