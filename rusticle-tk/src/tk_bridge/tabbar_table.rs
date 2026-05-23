//! TabBar, table, and progress bridge commands.

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use txv_widgets::{ProgressBar, TabBar, TabBarMode, Table};

use super::{get_widget, opt_val, parse_opts, parse_string_list, require_arg, tcl_to_string_list, SendShared};

pub fn register_tabbar(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("tabbar", move |_interp, args| {
        let sub = require_arg(args, 0, "tabbar")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st.alloc_id();
                let tb = TabBar::new(TabBarMode::Static);
                st.desktop.insert_widget(id.clone(), Box::new(tb));
                Ok(TclValue::Str(id))
            }
            "add" => {
                let id = require_arg(args, 1, "tabbar add")?;
                let title = require_arg(args, 2, "tabbar add")?;
                let tb = get_widget::<TabBar>(&mut st.desktop, &id, "tabbar add")?;
                tb.add_tab(title);
                Ok(TclValue::Str(String::new()))
            }
            "remove" => {
                let id = require_arg(args, 1, "tabbar remove")?;
                let index = require_arg(args, 2, "tabbar remove")?
                    .parse::<usize>()
                    .map_err(|_| TclError::new("tabbar remove: invalid index"))?;
                let tb = get_widget::<TabBar>(&mut st.desktop, &id, "tabbar remove")?;
                tb.remove_tab(index);
                Ok(TclValue::Str(String::new()))
            }
            "active" => {
                let id = require_arg(args, 1, "tabbar active")?;
                let tb = get_widget::<TabBar>(&mut st.desktop, &id, "tabbar active")?;
                Ok(TclValue::Int(tb.active_index() as i64))
            }
            "set-active" => {
                let id = require_arg(args, 1, "tabbar set-active")?;
                let index = require_arg(args, 2, "tabbar set-active")?
                    .parse::<usize>()
                    .map_err(|_| TclError::new("tabbar set-active: invalid index"))?;
                let tb = get_widget::<TabBar>(&mut st.desktop, &id, "tabbar set-active")?;
                tb.set_active(index);
                Ok(TclValue::Str(String::new()))
            }
            "on-change" => {
                let id = require_arg(args, 1, "tabbar on-change")?;
                let proc_name = require_arg(args, 2, "tabbar on-change")?;
                st.events.widget_handlers.insert((id, "on-change".into()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("tabbar: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_table(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("table", move |_interp, args| {
        let sub = require_arg(args, 0, "table")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let col_names = opt_val(&opts, "-columns").map(parse_string_list).unwrap_or_default();
                let columns: Vec<txv_widgets::table::Column> = col_names
                    .into_iter()
                    .map(|title| txv_widgets::table::Column { title, width: 20 })
                    .collect();
                let id = st.alloc_id();
                let tbl = Table::new(columns);
                st.desktop.insert_widget(id.clone(), Box::new(tbl));
                Ok(TclValue::Str(id))
            }
            "add-row" => {
                let id = require_arg(args, 1, "table add-row")?;
                let row_val = args
                    .get(2)
                    .ok_or_else(|| TclError::new("table add-row: missing row data"))?;
                let cells = tcl_to_string_list(row_val)?;
                let tbl = get_widget::<Table>(&mut st.desktop, &id, "table add-row")?;
                let mut rows = std::mem::take(&mut tbl.rows);
                rows.push(cells);
                tbl.set_rows(rows);
                Ok(TclValue::Str(String::new()))
            }
            "clear" => {
                let id = require_arg(args, 1, "table clear")?;
                let tbl = get_widget::<Table>(&mut st.desktop, &id, "table clear")?;
                tbl.set_rows(Vec::new());
                Ok(TclValue::Str(String::new()))
            }
            "selected" => {
                let id = require_arg(args, 1, "table selected")?;
                let tbl = get_widget::<Table>(&mut st.desktop, &id, "table selected")?;
                Ok(TclValue::Int(tbl.cursor as i64))
            }
            _ => Err(TclError::new(format!("table: unknown subcommand {sub}"))),
        }
    });
}

pub fn register_progress(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("progress", move |_interp, args| {
        let sub = require_arg(args, 0, "progress")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st.alloc_id();
                let pb = ProgressBar::new();
                st.desktop.insert_widget(id.clone(), Box::new(pb));
                Ok(TclValue::Str(id))
            }
            "set" => {
                let id = require_arg(args, 1, "progress set")?;
                let fraction = require_arg(args, 2, "progress set")?
                    .parse::<f32>()
                    .map_err(|_| TclError::new("progress set: invalid fraction"))?;
                let pb = get_widget::<ProgressBar>(&mut st.desktop, &id, "progress set")?;
                pb.set_progress(fraction);
                Ok(TclValue::Str(String::new()))
            }
            "done" => {
                let id = require_arg(args, 1, "progress done")?;
                let pb = get_widget::<ProgressBar>(&mut st.desktop, &id, "progress done")?;
                pb.set_progress(1.0);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("progress: unknown subcommand {sub}"))),
        }
    });
}
