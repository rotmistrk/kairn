//! Bridge between rusticle commands and txv-widgets.
//!
//! Registers all TK commands in the interpreter. Each command closure
//! captures a `Shared` handle for access to the desktop and event state.

mod commands;
mod editor_widget;
mod tabbar_table;
mod text_list;
mod tree_input;
mod window_app;

#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::desktop::TkDesktop;
use crate::event_mgr::EventState;

/// Shared mutable state for all bridge commands.
pub struct SharedState {
    /// Desktop holding widgets as Group children.
    pub desktop: TkDesktop,
    /// Event callbacks.
    pub events: EventState,
    /// Whether `app run` was called.
    pub run_requested: bool,
    /// Whether the event loop has already executed.
    pub has_run: bool,
    /// Next widget ID counter.
    next_id: u64,
}

impl SharedState {
    fn new() -> Self {
        Self {
            desktop: TkDesktop::new(),
            events: EventState::new(),
            run_requested: false,
            has_run: false,
            next_id: 1,
        }
    }

    fn alloc_id(&mut self) -> String {
        let id = format!("widget_{}", self.next_id);
        self.next_id += 1;
        id
    }
}

/// Convenience alias.
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

/// Parse a TclValue into a Vec<String>.
fn tcl_to_string_list(val: &TclValue) -> Result<Vec<String>, TclError> {
    let items = val.as_list()?;
    Ok(items.iter().map(|v| v.as_str().into_owned()).collect())
}

/// Parse a string as a simple space-separated list.
fn parse_string_list(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

// Re-export get_widget for submodules.
pub use commands::get_widget;

/// Register all TK bridge commands. Returns shared state handle.
pub fn register_all(interp: &mut Interpreter) -> Shared {
    #[allow(clippy::arc_with_non_send_sync)]
    let shared = Arc::new(Mutex::new(SharedState::new()));
    let ss = SendShared(Arc::clone(&shared));
    window_app::register_window(interp, &ss);
    window_app::register_app(interp, &ss);
    text_list::register_text(interp, &ss);
    text_list::register_list(interp, &ss);
    tree_input::register_tree(interp, &ss);
    tree_input::register_input(interp, &ss);
    tree_input::register_statusbar(interp, &ss);
    tabbar_table::register_tabbar(interp, &ss);
    tabbar_table::register_table(interp, &ss);
    tabbar_table::register_progress(interp, &ss);
    commands::register_dialog(interp);
    commands::register_menu(interp, &ss);
    commands::register_fuzzy_select(interp);
    commands::register_bind(interp, &ss);
    commands::register_after(interp, &ss);
    commands::register_focus(interp, &ss);
    commands::register_notify(interp);
    commands::register_files(interp);
    editor_widget::register_editor(interp, &ss);
    shared
}
