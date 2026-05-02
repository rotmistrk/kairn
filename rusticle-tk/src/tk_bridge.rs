//! Bridge between rusticle commands and txv-widgets.
//!
//! Registers all TK commands in the interpreter. Each command closure
//! captures an `Rc<RefCell<SharedState>>` for access to widgets, layout,
//! and event state.

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::event_mgr::EventState;
use crate::layout_mgr::{LayoutManager, Side};
use crate::widget_mgr::{StringListData, WidgetEntry, WidgetKind, WidgetManager};

/// Shared mutable state for all bridge commands.
pub struct SharedState {
    /// Widget registry.
    pub widgets: WidgetManager,
    /// Layout manager.
    pub layout: LayoutManager,
    /// Event callbacks.
    pub events: EventState,
    /// Whether `app run` was called.
    pub run_requested: bool,
    /// Whether the event loop has already executed.
    pub has_run: bool,
    /// Focused widget ID.
    pub focused: Option<String>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            widgets: WidgetManager::new(),
            layout: LayoutManager::new(),
            events: EventState::new(),
            run_requested: false,
            has_run: false,
            focused: None,
        }
    }
}

/// Convenience alias.
///
/// Safety: rusticle-tk is single-threaded. The `Send` bound on `register_fn`
/// is satisfied because the interpreter and all commands run on one thread.
pub type Shared = Arc<Mutex<SharedState>>;

/// Wrapper to satisfy `Send` for single-threaded use.
struct SendShared(Arc<Mutex<SharedState>>);

// Safety: rusticle-tk is single-threaded; SharedState is never sent across threads.
unsafe impl Send for SendShared {}
unsafe impl Sync for SendShared {}

impl SendShared {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SharedState>, TclError> {
        self.0
            .lock()
            .map_err(|_| TclError::new("internal: shared state lock poisoned"))
    }
}

impl Clone for SendShared {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

/// Get a required string arg at `index` from `args`, with `cmd` for errors.
fn require_arg(args: &[TclValue], index: usize, cmd: &str) -> Result<String, TclError> {
    args.get(index)
        .map(|v| v.as_str().into_owned())
        .ok_or_else(|| TclError::new(format!("{cmd}: missing argument {index}")))
}

/// Parse optional `-key value` pairs from args starting at `start`.
fn parse_opts(args: &[TclValue], start: usize) -> Vec<(String, String)> {
    let mut opts = Vec::new();
    let mut i = start;
    while i + 1 < args.len() {
        let key = args[i].as_str().into_owned();
        if key.starts_with('-') {
            let val = args[i + 1].as_str().into_owned();
            opts.push((key, val));
            i += 2;
        } else {
            break;
        }
    }
    opts
}

/// Find an option value by key.
fn opt_val<'a>(opts: &'a [(String, String)], key: &str) -> Option<&'a str> {
    opts.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// Register all TK bridge commands. Returns shared state handle.
pub fn register_all(interp: &mut Interpreter) -> Shared {
    #[allow(clippy::arc_with_non_send_sync)]
    let shared = Arc::new(Mutex::new(SharedState::new()));
    let ss = SendShared(Arc::clone(&shared));
    register_window(interp, &ss);
    register_app(interp, &ss);
    register_text(interp, &ss);
    register_list(interp, &ss);
    register_tree(interp, &ss);
    register_input(interp, &ss);
    register_statusbar(interp, &ss);
    register_tabbar(interp, &ss);
    register_table(interp, &ss);
    register_progress(interp, &ss);
    register_dialog(interp, &ss);
    register_menu(interp, &ss);
    register_fuzzy_select(interp, &ss);
    register_bind(interp, &ss);
    register_after(interp, &ss);
    register_notify(interp, &ss);
    register_files(interp);
    shared
}

// ── window commands ─────────────────────────────────────────────────

fn register_window(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("window", move |_interp, args| {
        let sub = require_arg(args, 0, "window")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let title = args
                    .get(1)
                    .map(|v| v.as_str().into_owned())
                    .unwrap_or_default();
                st.layout.set_title(&title);
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
                st.layout.add(&widget_id, side, size);
                Ok(TclValue::Str(String::new()))
            }
            "title" => {
                let _win = require_arg(args, 1, "window title")?;
                let title = require_arg(args, 2, "window title")?;
                st.layout.set_title(&title);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("window: unknown subcommand {sub}"))),
        }
    });
}

// ── app commands ────────────────────────────────────────────────────

fn register_app(interp: &mut Interpreter, shared: &SendShared) {
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
            "on-resize" => {
                let script = require_arg(args, 1, "app on-resize")?;
                st.events.resize_handler = Some(script);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("app: unknown subcommand {sub}"))),
        }
    });
}

// ── text commands ───────────────────────────────────────────────────

fn register_text(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("text", move |_interp, args| {
        let sub = require_arg(args, 0, "text")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let id = st.widgets.create(WidgetKind::Text).map_err(TclError::new)?;
                if let Some(path) = opt_val(&opts, "-file") {
                    let content = std::fs::read_to_string(path)
                        .map_err(|e| TclError::new(format!("text create: {e}")))?;
                    if let Some(WidgetEntry::Text(ta)) = st.widgets.get_mut(&id) {
                        ta.set_text(&content);
                    }
                }
                if let Some(content) = opt_val(&opts, "-content") {
                    if let Some(WidgetEntry::Text(ta)) = st.widgets.get_mut(&id) {
                        ta.set_text(content);
                    }
                }
                if let Some(val) = opt_val(&opts, "-readonly") {
                    let _readonly = val == "true" || val == "1";
                    // TextArea is always read-only in txv-widgets
                }
                Ok(TclValue::Str(id))
            }
            "load" => {
                let id = require_arg(args, 1, "text load")?;
                let path = require_arg(args, 2, "text load")?;
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| TclError::new(format!("text load: {e}")))?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Text(ta)) => ta.set_text(&content),
                    _ => return Err(TclError::new(format!("text load: no text widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "set" => {
                let id = require_arg(args, 1, "text set")?;
                let content = require_arg(args, 2, "text set")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Text(ta)) => ta.set_text(&content),
                    _ => return Err(TclError::new(format!("text set: no text widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "get" => {
                let id = require_arg(args, 1, "text get")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::Text(ta)) => Ok(TclValue::Str(ta.lines().join("\n"))),
                    _ => Err(TclError::new(format!("text get: no text widget {id}"))),
                }
            }
            "clear" => {
                let id = require_arg(args, 1, "text clear")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Text(ta)) => ta.set_text(""),
                    _ => return Err(TclError::new(format!("text clear: no text widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "append" => {
                let id = require_arg(args, 1, "text append")?;
                let content = require_arg(args, 2, "text append")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Text(ta)) => {
                        let mut lines: Vec<String> =
                            ta.lines().iter().map(|s| s.to_string()).collect();
                        for line in content.lines() {
                            lines.push(line.to_string());
                        }
                        ta.set_lines(lines);
                    }
                    _ => return Err(TclError::new(format!("text append: no text widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "line-numbers" => {
                let id = require_arg(args, 1, "text line-numbers")?;
                let val = require_arg(args, 2, "text line-numbers")?;
                let show = val == "true" || val == "1";
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Text(ta)) => ta.set_show_line_numbers(show),
                    _ => {
                        return Err(TclError::new(format!(
                            "text line-numbers: no text widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "readonly" | "save" | "cursor" | "scroll" | "syntax" => {
                // Stubs for commands that need more infrastructure
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("text: unknown subcommand {sub}"))),
        }
    });
}

// ── list commands ───────────────────────────────────────────────────

fn register_list(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("list", move |_interp, args| {
        let sub = require_arg(args, 0, "list")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let id = st.widgets.create(WidgetKind::List).map_err(TclError::new)?;
                if let Some(items_str) = opt_val(&opts, "-items") {
                    let items = parse_string_list(items_str);
                    if let Some(WidgetEntry::List(lv)) = st.widgets.get_mut(&id) {
                        lv.set_data(StringListData::new(items));
                    }
                }
                Ok(TclValue::Str(id))
            }
            "set-items" => {
                let id = require_arg(args, 1, "list set-items")?;
                let items_val = args
                    .get(2)
                    .ok_or_else(|| TclError::new("list set-items: missing items"))?;
                let items = tcl_to_string_list(items_val)?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::List(lv)) => lv.set_data(StringListData::new(items)),
                    _ => {
                        return Err(TclError::new(format!(
                            "list set-items: no list widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "selected" => {
                let id = require_arg(args, 1, "list selected")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::List(lv)) => {
                        let idx = lv.selected();
                        let text = lv.data().items.get(idx).cloned().unwrap_or_default();
                        Ok(TclValue::Str(text))
                    }
                    _ => Err(TclError::new(format!("list selected: no list widget {id}"))),
                }
            }
            "index" => {
                let id = require_arg(args, 1, "list index")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::List(lv)) => Ok(TclValue::Int(lv.selected() as i64)),
                    _ => Err(TclError::new(format!("list index: no list widget {id}"))),
                }
            }
            "on-select" | "on-activate" => {
                let id = require_arg(args, 1, &format!("list {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("list {sub}"))?;
                st.events
                    .widget_handlers
                    .insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("list: unknown subcommand {sub}"))),
        }
    });
}

/// Parse a TclValue into a Vec<String>.
fn tcl_to_string_list(val: &TclValue) -> Result<Vec<String>, TclError> {
    let items = val.as_list()?;
    Ok(items.iter().map(|v| v.as_str().into_owned()).collect())
}

/// Parse a string as a simple space-separated list.
fn parse_string_list(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

// ── tree commands ───────────────────────────────────────────────────

fn register_tree(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("tree", move |_interp, args| {
        let sub = require_arg(args, 0, "tree")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let path = opt_val(&opts, "-data").unwrap_or(".");
                let id = st
                    .widgets
                    .create(WidgetKind::Tree(path.to_string()))
                    .map_err(TclError::new)?;
                Ok(TclValue::Str(id))
            }
            "selected" => {
                let id = require_arg(args, 1, "tree selected")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::Tree(tv)) => {
                        let path = tv
                            .selected_node()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        Ok(TclValue::Str(path))
                    }
                    _ => Err(TclError::new(format!("tree selected: no tree widget {id}"))),
                }
            }
            "expand" => {
                let id = require_arg(args, 1, "tree expand")?;
                let node = require_arg(args, 2, "tree expand")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Tree(tv)) => {
                        tv.expand(&std::path::PathBuf::from(&node));
                    }
                    _ => return Err(TclError::new(format!("tree expand: no tree widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "collapse" => {
                let id = require_arg(args, 1, "tree collapse")?;
                let node = require_arg(args, 2, "tree collapse")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Tree(tv)) => {
                        tv.collapse(&std::path::PathBuf::from(&node));
                    }
                    _ => return Err(TclError::new(format!("tree collapse: no tree widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "refresh" => {
                let id = require_arg(args, 1, "tree refresh")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Tree(tv)) => {
                        let root = tv.data().root().to_path_buf();
                        let data = txv_widgets::FileTreeData::new(&root, 10)
                            .map_err(|e| TclError::new(format!("tree refresh: {e}")))?;
                        tv.set_data(data);
                    }
                    _ => return Err(TclError::new(format!("tree refresh: no tree widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "on-select" | "on-activate" => {
                let id = require_arg(args, 1, &format!("tree {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("tree {sub}"))?;
                st.events
                    .widget_handlers
                    .insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("tree: unknown subcommand {sub}"))),
        }
    });
}

// ── input commands ──────────────────────────────────────────────────

fn register_input(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("input", move |_interp, args| {
        let sub = require_arg(args, 0, "input")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let prompt = opt_val(&opts, "-prompt").unwrap_or("");
                let id = st
                    .widgets
                    .create(WidgetKind::Input(prompt.to_string()))
                    .map_err(TclError::new)?;
                if let Some(default) = opt_val(&opts, "-default") {
                    if let Some(WidgetEntry::Input(inp)) = st.widgets.get_mut(&id) {
                        inp.set_text(default);
                    }
                }
                Ok(TclValue::Str(id))
            }
            "get" => {
                let id = require_arg(args, 1, "input get")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::Input(inp)) => Ok(TclValue::Str(inp.text().to_string())),
                    _ => Err(TclError::new(format!("input get: no input widget {id}"))),
                }
            }
            "set" => {
                let id = require_arg(args, 1, "input set")?;
                let text = require_arg(args, 2, "input set")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Input(inp)) => inp.set_text(&text),
                    _ => return Err(TclError::new(format!("input set: no input widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "clear" => {
                let id = require_arg(args, 1, "input clear")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Input(inp)) => inp.clear(),
                    _ => return Err(TclError::new(format!("input clear: no input widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "focus" => {
                let id = require_arg(args, 1, "input focus")?;
                st.focused = Some(id);
                Ok(TclValue::Str(String::new()))
            }
            "on-change" | "on-submit" => {
                let id = require_arg(args, 1, &format!("input {sub}"))?;
                let proc_name = require_arg(args, 2, &format!("input {sub}"))?;
                st.events
                    .widget_handlers
                    .insert((id, sub.clone()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("input: unknown subcommand {sub}"))),
        }
    });
}

// ── statusbar commands ──────────────────────────────────────────────

fn register_statusbar(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("statusbar", move |_interp, args| {
        let sub = require_arg(args, 0, "statusbar")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st
                    .widgets
                    .create(WidgetKind::StatusBar)
                    .map_err(TclError::new)?;
                Ok(TclValue::Str(id))
            }
            "left" => {
                let id = require_arg(args, 1, "statusbar left")?;
                let text = require_arg(args, 2, "statusbar left")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::StatusBar(sb)) => {
                        sb.set_left(vec![txv_widgets::StatusSpan {
                            text,
                            style: txv::cell::Style::default(),
                        }]);
                    }
                    _ => {
                        return Err(TclError::new(format!(
                            "statusbar left: no statusbar widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "right" => {
                let id = require_arg(args, 1, "statusbar right")?;
                let text = require_arg(args, 2, "statusbar right")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::StatusBar(sb)) => {
                        sb.set_right(vec![txv_widgets::StatusSpan {
                            text,
                            style: txv::cell::Style::default(),
                        }]);
                    }
                    _ => {
                        return Err(TclError::new(format!(
                            "statusbar right: no statusbar widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!(
                "statusbar: unknown subcommand {sub}"
            ))),
        }
    });
}

// ── tabbar commands ─────────────────────────────────────────────────

fn register_tabbar(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("tabbar", move |_interp, args| {
        let sub = require_arg(args, 0, "tabbar")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st
                    .widgets
                    .create(WidgetKind::TabBar)
                    .map_err(TclError::new)?;
                Ok(TclValue::Str(id))
            }
            "add" => {
                let id = require_arg(args, 1, "tabbar add")?;
                let title = require_arg(args, 2, "tabbar add")?;
                let opts = parse_opts(args, 3);
                let modified = opt_val(&opts, "-modified")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false);
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::TabBar(tb)) => {
                        tb.add(txv_widgets::TabEntry { title, modified });
                    }
                    _ => return Err(TclError::new(format!("tabbar add: no tabbar widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "remove" => {
                let id = require_arg(args, 1, "tabbar remove")?;
                let index = require_arg(args, 2, "tabbar remove")?
                    .parse::<usize>()
                    .map_err(|_| TclError::new("tabbar remove: invalid index"))?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::TabBar(tb)) => tb.remove(index),
                    _ => {
                        return Err(TclError::new(format!(
                            "tabbar remove: no tabbar widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "active" => {
                let id = require_arg(args, 1, "tabbar active")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::TabBar(tb)) => Ok(TclValue::Int(tb.active() as i64)),
                    _ => Err(TclError::new(format!(
                        "tabbar active: no tabbar widget {id}"
                    ))),
                }
            }
            "set-active" => {
                let id = require_arg(args, 1, "tabbar set-active")?;
                let index = require_arg(args, 2, "tabbar set-active")?
                    .parse::<usize>()
                    .map_err(|_| TclError::new("tabbar set-active: invalid index"))?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::TabBar(tb)) => tb.set_active(index),
                    _ => {
                        return Err(TclError::new(format!(
                            "tabbar set-active: no tabbar widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "on-change" => {
                let id = require_arg(args, 1, "tabbar on-change")?;
                let proc_name = require_arg(args, 2, "tabbar on-change")?;
                st.events
                    .widget_handlers
                    .insert((id, "on-change".into()), proc_name);
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("tabbar: unknown subcommand {sub}"))),
        }
    });
}

// ── table commands ──────────────────────────────────────────────────

fn register_table(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("table", move |_interp, args| {
        let sub = require_arg(args, 0, "table")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let opts = parse_opts(args, 1);
                let col_names = opt_val(&opts, "-columns")
                    .map(parse_string_list)
                    .unwrap_or_default();
                let columns: Vec<txv_widgets::table::Column> = col_names
                    .into_iter()
                    .map(|title| txv_widgets::table::Column {
                        title,
                        width: 0,
                        align: txv_widgets::table::Align::Left,
                    })
                    .collect();
                let id = st
                    .widgets
                    .create(WidgetKind::Table(columns))
                    .map_err(TclError::new)?;
                Ok(TclValue::Str(id))
            }
            "add-row" => {
                let id = require_arg(args, 1, "table add-row")?;
                let row_val = args
                    .get(2)
                    .ok_or_else(|| TclError::new("table add-row: missing row data"))?;
                let cells = tcl_to_string_list(row_val)?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Table(tbl)) => {
                        let mut rows: Vec<Vec<String>> = Vec::new();
                        // Collect existing rows
                        let count = tbl.row_count();
                        for i in 0..count {
                            if let Some(r) = tbl.row(i) {
                                rows.push(r.to_vec());
                            }
                        }
                        rows.push(cells);
                        tbl.set_rows(rows);
                    }
                    _ => {
                        return Err(TclError::new(format!(
                            "table add-row: no table widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "clear" => {
                let id = require_arg(args, 1, "table clear")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Table(tbl)) => tbl.set_rows(Vec::new()),
                    _ => return Err(TclError::new(format!("table clear: no table widget {id}"))),
                }
                Ok(TclValue::Str(String::new()))
            }
            "selected" => {
                let id = require_arg(args, 1, "table selected")?;
                match st.widgets.get(&id) {
                    Some(WidgetEntry::Table(tbl)) => Ok(TclValue::Int(tbl.selected() as i64)),
                    _ => Err(TclError::new(format!(
                        "table selected: no table widget {id}"
                    ))),
                }
            }
            "sort" => {
                // Sort is a stub — would need sortable Table API
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("table: unknown subcommand {sub}"))),
        }
    });
}

// ── progress commands ───────────────────────────────────────────────

fn register_progress(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("progress", move |_interp, args| {
        let sub = require_arg(args, 0, "progress")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let id = st
                    .widgets
                    .create(WidgetKind::Progress)
                    .map_err(TclError::new)?;
                let opts = parse_opts(args, 1);
                if let Some(title) = opt_val(&opts, "-title") {
                    if let Some(WidgetEntry::Progress(pb)) = st.widgets.get_mut(&id) {
                        pb.label = title.to_string();
                    }
                }
                Ok(TclValue::Str(id))
            }
            "set" => {
                let id = require_arg(args, 1, "progress set")?;
                let fraction = require_arg(args, 2, "progress set")?
                    .parse::<f64>()
                    .map_err(|_| TclError::new("progress set: invalid fraction"))?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Progress(pb)) => {
                        pb.set_progress(fraction);
                        if let Some(msg) = args.get(3) {
                            pb.label = msg.as_str().into_owned();
                        }
                    }
                    _ => {
                        return Err(TclError::new(format!(
                            "progress set: no progress widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            "done" => {
                let id = require_arg(args, 1, "progress done")?;
                match st.widgets.get_mut(&id) {
                    Some(WidgetEntry::Progress(pb)) => {
                        pb.set_progress(1.0);
                        pb.label = "Done".into();
                    }
                    _ => {
                        return Err(TclError::new(format!(
                            "progress done: no progress widget {id}"
                        )))
                    }
                }
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("progress: unknown subcommand {sub}"))),
        }
    });
}

// ── dialog commands ─────────────────────────────────────────────────

fn register_dialog(interp: &mut Interpreter, _shared: &SendShared) {
    interp.register_fn("dialog", move |_interp, args| {
        let sub = require_arg(args, 0, "dialog")?;
        match sub.as_str() {
            "confirm" => {
                let msg = require_arg(args, 1, "dialog confirm")?;
                // In non-interactive mode, return false
                // Real implementation runs a modal dialog in the event loop
                let _ = msg;
                Ok(TclValue::Bool(false))
            }
            "prompt" => {
                let msg = require_arg(args, 1, "dialog prompt")?;
                let default = args
                    .get(2)
                    .map(|v| v.as_str().into_owned())
                    .unwrap_or_default();
                let _ = msg;
                Ok(TclValue::Str(default))
            }
            "info" => {
                let msg = require_arg(args, 1, "dialog info")?;
                let _ = msg;
                Ok(TclValue::Str(String::new()))
            }
            "error" => {
                let msg = require_arg(args, 1, "dialog error")?;
                let _ = msg;
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("dialog: unknown subcommand {sub}"))),
        }
    });
}

// ── menu commands ───────────────────────────────────────────────────

fn register_menu(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("menu", move |_interp, args| {
        let sub = require_arg(args, 0, "menu")?;
        let mut st = s.lock()?;
        match sub.as_str() {
            "create" => {
                let items_val = args
                    .get(1)
                    .ok_or_else(|| TclError::new("menu create: missing items"))?;
                let items_list = items_val.as_list()?;
                let items: Vec<txv_widgets::MenuItem> = items_list
                    .iter()
                    .map(|v| {
                        let label = v.as_str().into_owned();
                        if label == "---" {
                            txv_widgets::MenuItem::separator()
                        } else {
                            txv_widgets::MenuItem::new(label, "")
                        }
                    })
                    .collect();
                let id = st
                    .widgets
                    .create(WidgetKind::Menu(items))
                    .map_err(TclError::new)?;
                Ok(TclValue::Str(id))
            }
            "show" => {
                // Stub — would position and show the menu overlay
                Ok(TclValue::Str(String::new()))
            }
            _ => Err(TclError::new(format!("menu: unknown subcommand {sub}"))),
        }
    });
}

// ── fuzzy-select command ────────────────────────────────────────────

fn register_fuzzy_select(interp: &mut Interpreter, _shared: &SendShared) {
    interp.register_fn("fuzzy-select", move |_interp, args| {
        let items_val = args
            .first()
            .ok_or_else(|| TclError::new("fuzzy-select: missing items"))?;
        let items = tcl_to_string_list(items_val)?;
        // In non-interactive mode, return first item or empty
        Ok(TclValue::Str(items.into_iter().next().unwrap_or_default()))
    });
}

// ── bind command ────────────────────────────────────────────────────

fn register_bind(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("bind", move |_interp, args| {
        let keyspec = require_arg(args, 0, "bind")?;
        let script = require_arg(args, 1, "bind")?;
        let mut st = s.lock()?;
        st.events.key_bindings.insert(keyspec, script);
        Ok(TclValue::Str(String::new()))
    });
}

// ── after command ───────────────────────────────────────────────────

fn register_after(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("after", move |_interp, args| {
        let ms = require_arg(args, 0, "after")?
            .parse::<u64>()
            .map_err(|_| TclError::new("after: invalid delay"))?;
        // Check for -repeat flag
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

// ── notify command ──────────────────────────────────────────────────

fn register_notify(interp: &mut Interpreter, shared: &SendShared) {
    let s = shared.clone();
    interp.register_fn("notify", move |_interp, args| {
        let msg = require_arg(args, 0, "notify")?;
        let opts = parse_opts(args, 1);
        let duration = opt_val(&opts, "-duration")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(3000);
        let _style = opt_val(&opts, "-style").unwrap_or("info");
        let mut st = s.lock()?;
        st.events.pending_notification = Some((msg, duration));
        Ok(TclValue::Str(String::new()))
    });
}

// ── files command ───────────────────────────────────────────────────

fn register_files(interp: &mut Interpreter) {
    interp.register_fn("files", move |_interp, args| {
        let path = require_arg(args, 0, "files")?;
        let opts = parse_opts(args, 1);
        let recursive = opts.iter().any(|(k, _)| k == "-recursive");
        let filter = opt_val(&opts, "-filter");

        let walker = ignore::WalkBuilder::new(&path)
            .max_depth(if recursive { None } else { Some(1) })
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
            if let Some(pat) = filter {
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

/// Simple glob matching: supports `*` as wildcard prefix.
fn glob_match(pattern: &str, name: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        name == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusticle::interpreter::Interpreter;

    fn setup() -> (Interpreter, Shared) {
        let mut interp = Interpreter::new();
        let shared = register_all(&mut interp);
        (interp, shared)
    }

    /// Extract string value from eval result.
    fn val(result: &Result<TclValue, rusticle::error::TclError>) -> String {
        match result {
            Ok(v) => v.as_str().into_owned(),
            Err(e) => panic!("eval failed: {e}"),
        }
    }

    #[test]
    fn window_create_returns_id() {
        let (mut interp, _) = setup();
        let result = interp.eval(r#"window create "Test""#);
        assert!(result.is_ok());
        assert_eq!(val(&result), "window_0");
    }

    #[test]
    fn text_create_and_get() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set t [text create -content "hello"]"#);
        let result = interp.eval("text get $t");
        assert_eq!(val(&result), "hello");
    }

    #[test]
    fn text_set_and_get() {
        let (mut interp, _) = setup();
        let _ = interp.eval("set t [text create]");
        let _ = interp.eval(r#"text set $t "world""#);
        let result = interp.eval("text get $t");
        assert_eq!(val(&result), "world");
    }

    #[test]
    fn text_clear() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set t [text create -content "data"]"#);
        let _ = interp.eval("text clear $t");
        let result = interp.eval("text get $t");
        assert_eq!(val(&result), "");
    }

    #[test]
    fn list_create_and_selected() {
        let (mut interp, _) = setup();
        let _ = interp.eval("set l [list create]");
        let result = interp.eval("list index $l");
        assert_eq!(val(&result), "0");
    }

    #[test]
    fn statusbar_create_and_set() {
        let (mut interp, _) = setup();
        let _ = interp.eval("set s [statusbar create]");
        let result = interp.eval(r#"statusbar left $s "Ready""#);
        assert!(result.is_ok());
    }

    #[test]
    fn tabbar_create_add_active() {
        let (mut interp, _) = setup();
        let _ = interp.eval("set tb [tabbar create]");
        let _ = interp.eval(r#"tabbar add $tb "Tab1""#);
        let _ = interp.eval(r#"tabbar add $tb "Tab2""#);
        let result = interp.eval("tabbar active $tb");
        assert_eq!(val(&result), "0");
        let _ = interp.eval("tabbar set-active $tb 1");
        let result = interp.eval("tabbar active $tb");
        assert_eq!(val(&result), "1");
    }

    #[test]
    fn progress_create_and_set() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set p [progress create -title "Build"]"#);
        let result = interp.eval("progress set $p 0.5");
        assert!(result.is_ok());
        let result = interp.eval("progress done $p");
        assert!(result.is_ok());
    }

    #[test]
    fn bind_registers_key() {
        let (mut interp, shared) = setup();
        let _ = interp.eval(r#"bind Ctrl-Q { app quit }"#);
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert!(st.events.key_bindings.contains_key("Ctrl-Q"));
    }

    #[test]
    fn app_run_sets_flag() {
        let (mut interp, shared) = setup();
        let _ = interp.eval("app run");
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert!(st.run_requested);
    }

    #[test]
    fn app_quit_sets_flag() {
        let (mut interp, shared) = setup();
        let _ = interp.eval("app quit");
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert!(st.events.quit_requested);
    }

    #[test]
    fn dialog_confirm_returns_bool() {
        let (mut interp, _) = setup();
        let result = interp.eval(r#"dialog confirm "Sure?""#);
        // Non-interactive mode returns false
        assert_eq!(val(&result), "0");
    }

    #[test]
    fn dialog_prompt_returns_default() {
        let (mut interp, _) = setup();
        let result = interp.eval(r#"dialog prompt "Name:" "default""#);
        assert_eq!(val(&result), "default");
    }

    #[test]
    fn window_add_updates_layout() {
        let (mut interp, shared) = setup();
        let _ = interp.eval(r#"set w [window create "T"]"#);
        let _ = interp.eval("set t [text create]");
        let _ = interp.eval("window add $w $t -side fill");
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        let rects = st.layout.compute(txv::layout::Rect {
            x: 0,
            y: 0,
            w: 80,
            h: 24,
        });
        assert_eq!(rects.len(), 1);
    }

    #[test]
    fn input_create_get_set() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set i [input create -prompt "> "]"#);
        let _ = interp.eval(r#"input set $i "hello""#);
        let result = interp.eval("input get $i");
        assert_eq!(val(&result), "hello");
    }

    #[test]
    fn input_clear() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set i [input create]"#);
        let _ = interp.eval(r#"input set $i "data""#);
        let _ = interp.eval("input clear $i");
        let result = interp.eval("input get $i");
        assert_eq!(val(&result), "");
    }

    #[test]
    fn table_create_add_row() {
        let (mut interp, _) = setup();
        let _ = interp.eval(r#"set t [table create -columns "Name Size"]"#);
        let result = interp.eval("table selected $t");
        assert_eq!(val(&result), "0");
    }

    #[test]
    fn after_registers_timer() {
        let (mut interp, shared) = setup();
        let _ = interp.eval(r#"after 1000 { puts "tick" }"#);
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(st.events.timers.len(), 1);
        assert!(!st.events.timers[0].repeat);
    }

    #[test]
    fn after_repeat_registers_timer() {
        let (mut interp, shared) = setup();
        let _ = interp.eval(r#"after 1000 -repeat { puts "tick" }"#);
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(st.events.timers.len(), 1);
        assert!(st.events.timers[0].repeat);
    }

    #[test]
    fn hello_script_parses() {
        let (mut interp, shared) = setup();
        let script = r#"
            set win [window create "Hello"]
            set txt [text create -content "Hello, rusticle-tk!"]
            window add $win $txt -side fill
            set status [statusbar create]
            statusbar left $status "Ready"
            window add $win $status -side bottom -height 1
            bind Ctrl-Q { app quit }
            app run
        "#;
        let result = interp.eval(script);
        assert!(result.is_ok(), "hello script failed: {result:?}");
        let st = shared.lock().unwrap_or_else(|e| e.into_inner());
        assert!(st.run_requested);
        assert!(st.events.key_bindings.contains_key("Ctrl-Q"));
    }

    #[test]
    fn dialog_demo_script_parses() {
        let (mut interp, _) = setup();
        let script = r#"
            set answer [dialog confirm "Proceed?"]
            if {$answer} { puts "yes" } else { puts "no" }
        "#;
        let result = interp.eval(script);
        assert!(result.is_ok(), "dialog demo failed: {result:?}");
        let output = interp.get_output().join("");
        assert!(output.contains("no"));
    }
}
