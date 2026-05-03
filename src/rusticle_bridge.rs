//! Bridge between kairn and the rusticle scripting interpreter.
//!
//! Registers all kairn-specific commands (buffer, editor, window,
//! terminal, kiro, hook) in the interpreter. Each command closure
//! captures shared application state behind `Arc<Mutex<T>>`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rusticle::error::TclError;
use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use crate::config::keybindings::{BindingSource, BindingTable, BoundAction, KeySpec};
use crate::config::themes::ThemeValues;

// ── Shared state ─────────────────────────

/// Lifecycle hooks that fire on editor events.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HookEvent {
    /// A buffer was opened.
    BufferOpen,
    /// A buffer was saved.
    BufferSave,
    /// A buffer was closed.
    BufferClose,
    /// Focus changed between panels.
    FocusChange,
    /// Editor mode changed.
    ModeChange,
    /// Application started.
    Startup,
    /// Application shutting down.
    Shutdown,
}

/// Hook registry for lifecycle events.
pub struct HookRegistry {
    hooks: HashMap<HookEvent, Vec<String>>,
}

impl HookRegistry {
    fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Add a hook script for an event.
    pub fn add(&mut self, event: HookEvent, script: String) {
        self.hooks.entry(event).or_default().push(script);
    }

    /// Remove all hooks for an event.
    pub fn remove(&mut self, event: &HookEvent) {
        self.hooks.remove(event);
    }

    /// Fire all hooks for an event.
    pub fn fire(&self, event: &HookEvent, interp: &mut Interpreter) -> Result<(), TclError> {
        if let Some(scripts) = self.hooks.get(event) {
            for script in scripts {
                interp.eval(script)?;
            }
        }
        Ok(())
    }

    fn list_events(&self) -> Vec<String> {
        self.hooks.keys().map(|e| format!("{e:?}")).collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared mutable state accessible to all bridge commands.
///
/// Fields are stubs until Phase B wires up the real application state.
pub struct BridgeState {
    /// Keybinding table.
    pub bindings: BindingTable,
    /// Theme values.
    pub theme: ThemeValues,
    /// Lifecycle hooks.
    pub hooks: HookRegistry,
    /// Whether quit was requested.
    pub quit_requested: bool,
}

impl BridgeState {
    /// Create a new bridge state with defaults.
    pub fn new() -> Self {
        Self {
            bindings: BindingTable::new(),
            theme: ThemeValues::new(),
            hooks: HookRegistry::new(),
            quit_requested: false,
        }
    }
}

impl Default for BridgeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe handle to bridge state.
pub type BridgeHandle = Arc<Mutex<BridgeState>>;

// ── Registry ─────────────────────────────

/// Registers all kairn bridge commands in the interpreter.
pub struct BridgeRegistry;

impl BridgeRegistry {
    /// Register all commands. Returns the shared state handle.
    pub fn register(interp: &mut Interpreter) -> BridgeHandle {
        let handle = Arc::new(Mutex::new(BridgeState::new()));
        register_buffer(interp, &handle);
        register_editor(interp, &handle);
        register_window(interp, &handle);
        register_bind(interp, &handle);
        register_theme(interp, &handle);
        register_terminal(interp, &handle);
        register_kiro(interp, &handle);
        register_hook(interp, &handle);
        handle
    }
}

// ── Helpers ──────────────────────────────

fn require_arg(args: &[TclValue], index: usize, cmd: &str) -> Result<String, TclError> {
    args.get(index)
        .map(|v| v.as_str().into_owned())
        .ok_or_else(|| TclError::new(format!("{cmd}: missing argument {index}")))
}

fn lock_state(handle: &BridgeHandle) -> Result<std::sync::MutexGuard<'_, BridgeState>, TclError> {
    handle
        .lock()
        .map_err(|_| TclError::new("internal: state lock poisoned"))
}

fn ok_empty() -> Result<TclValue, TclError> {
    Ok(TclValue::Str(String::new()))
}

fn stub(cmd: &str, sub: &str) -> Result<TclValue, TclError> {
    // Stubs return empty — will be wired in Phase B.
    let _ = (cmd, sub);
    ok_empty()
}

// ── Buffer commands ──────────────────────

fn register_buffer(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("buffer", move |_interp, args| {
        let sub = require_arg(args, 0, "buffer")?;
        let _st = lock_state(&h)?;
        match sub.as_str() {
            "save" | "save-all" | "close" | "reload" | "new" => stub("buffer", &sub),
            "list" => Ok(TclValue::List(Vec::new())),
            "modified" => Ok(TclValue::Bool(false)),
            _ => Err(TclError::new(format!(
                "buffer: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

// ── Editor commands ──────────────────────

fn register_editor(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("editor", move |_interp, args| {
        let sub = require_arg(args, 0, "editor")?;
        let mut st = lock_state(&h)?;
        match sub.as_str() {
            "quit" => {
                st.quit_requested = true;
                ok_empty()
            }
            "goto" | "insert" | "delete" | "undo" | "redo" | "find" | "replace"
            | "comment-toggle" | "diff" | "git-log" | "open-search" | "help" | "save-session"
            | "load-session" | "cycle-mode" => stub("editor", &sub),
            "selection" => Ok(TclValue::Str(String::new())),
            "cursor" => Ok(TclValue::Str("0 0".into())),
            _ => Err(TclError::new(format!(
                "editor: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

// ── Window commands ──────────────────────

fn register_window(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("window", move |_interp, args| {
        let sub = require_arg(args, 0, "window")?;
        let _st = lock_state(&h)?;
        match sub.as_str() {
            "toggle-tree" | "toggle-left" | "cycle-focus" | "focus" | "split" | "close"
            | "resize" | "refresh-tree" | "redraw" => stub("window", &sub),
            _ => Err(TclError::new(format!(
                "window: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

// ── Bind command ─────────────────────────

fn register_bind(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("bind", move |_interp, args| {
        let keyspec_str = require_arg(args, 0, "bind")?;
        let script = require_arg(args, 1, "bind")?;
        let ks = KeySpec::parse(&keyspec_str).map_err(|e| TclError::new(format!("bind: {e}")))?;
        let mut st = lock_state(&h)?;
        st.bindings.bind(
            ks,
            BoundAction {
                script,
                source: BindingSource::Runtime,
            },
        );
        ok_empty()
    });
}

// ── Theme command ────────────────────────

fn register_theme(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("theme", move |interp, args| {
        let sub = require_arg(args, 0, "theme")?;
        let mut st = lock_state(&h)?;
        match sub.as_str() {
            "load" => {
                let name = require_arg(args, 1, "theme load")?;
                let script = st
                    .theme
                    .find_theme_script(&name)
                    .ok_or_else(|| TclError::new(format!("theme: unknown theme \"{name}\"")))?;
                drop(st); // release lock before eval
                interp.eval(&script)?;
                let mut st2 = lock_state(&h)?;
                st2.theme.apply_from_context(interp);
                ok_empty()
            }
            "list" => {
                let names = st.theme.available_themes();
                Ok(TclValue::List(
                    names.into_iter().map(TclValue::Str).collect(),
                ))
            }
            "get" => {
                let prop = require_arg(args, 1, "theme get")?;
                let val = st
                    .theme
                    .get(&prop)
                    .ok_or_else(|| TclError::new(format!("theme: unknown property \"{prop}\"")))?;
                Ok(TclValue::Str(val.to_string()))
            }
            "set" => {
                let prop = require_arg(args, 1, "theme set")?;
                let val = require_arg(args, 2, "theme set")?;
                st.theme.set(&prop, &val);
                ok_empty()
            }
            _ => Err(TclError::new(format!(
                "theme: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

// ── Terminal commands ────────────────────

fn register_terminal(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("terminal", move |_interp, args| {
        let sub = require_arg(args, 0, "terminal")?;
        let _st = lock_state(&h)?;
        match sub.as_str() {
            "new-shell" | "new-kiro" | "close-tab" | "prev-tab" | "next-tab" | "send" => {
                stub("terminal", &sub)
            }
            "list" => Ok(TclValue::List(Vec::new())),
            _ => Err(TclError::new(format!(
                "terminal: unknown subcommand \"{sub}\""
            ))),
        }
    });
}

// ── Kiro commands ────────────────────────

fn register_kiro(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("kiro", move |_interp, args| {
        let sub = require_arg(args, 0, "kiro")?;
        let _st = lock_state(&h)?;
        match sub.as_str() {
            "send" | "cancel" => stub("kiro", &sub),
            "status" => Ok(TclValue::Str("idle".into())),
            _ => Err(TclError::new(format!("kiro: unknown subcommand \"{sub}\""))),
        }
    });
}

// ── Hook commands ────────────────────────

fn register_hook(interp: &mut Interpreter, handle: &BridgeHandle) {
    let h = Arc::clone(handle);
    interp.register_fn("hook", move |_interp, args| {
        let sub = require_arg(args, 0, "hook")?;
        let mut st = lock_state(&h)?;
        match sub.as_str() {
            "add" => {
                let event_str = require_arg(args, 1, "hook add")?;
                let script = require_arg(args, 2, "hook add")?;
                let event = parse_hook_event(&event_str)?;
                st.hooks.add(event, script);
                ok_empty()
            }
            "remove" => {
                let event_str = require_arg(args, 1, "hook remove")?;
                let event = parse_hook_event(&event_str)?;
                st.hooks.remove(&event);
                ok_empty()
            }
            "list" => {
                let events = st.hooks.list_events();
                Ok(TclValue::List(
                    events.into_iter().map(TclValue::Str).collect(),
                ))
            }
            _ => Err(TclError::new(format!("hook: unknown subcommand \"{sub}\""))),
        }
    });
}

fn parse_hook_event(s: &str) -> Result<HookEvent, TclError> {
    match s {
        "buffer-open" => Ok(HookEvent::BufferOpen),
        "buffer-save" => Ok(HookEvent::BufferSave),
        "buffer-close" => Ok(HookEvent::BufferClose),
        "focus-change" => Ok(HookEvent::FocusChange),
        "mode-change" => Ok(HookEvent::ModeChange),
        "startup" => Ok(HookEvent::Startup),
        "shutdown" => Ok(HookEvent::Shutdown),
        _ => Err(TclError::new(format!("hook: unknown event \"{s}\""))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_all_commands() {
        let mut interp = Interpreter::new();
        let _handle = BridgeRegistry::register(&mut interp);
        // All commands should be registered without error
    }

    #[test]
    fn buffer_list_returns_empty() {
        let mut interp = Interpreter::new();
        let _handle = BridgeRegistry::register(&mut interp);
        let result = interp.eval("buffer list").unwrap();
        assert_eq!(result.as_str().as_ref(), "");
    }

    #[test]
    fn editor_quit_sets_flag() {
        let mut interp = Interpreter::new();
        let handle = BridgeRegistry::register(&mut interp);
        interp.eval("editor quit").unwrap();
        let st = handle.lock().unwrap();
        assert!(st.quit_requested);
    }

    #[test]
    fn unknown_subcommand_errors() {
        let mut interp = Interpreter::new();
        let _handle = BridgeRegistry::register(&mut interp);
        assert!(interp.eval("buffer frobnicate").is_err());
        assert!(interp.eval("editor frobnicate").is_err());
        assert!(interp.eval("window frobnicate").is_err());
        assert!(interp.eval("theme frobnicate").is_err());
    }

    #[test]
    fn bind_via_script() {
        let mut interp = Interpreter::new();
        let handle = BridgeRegistry::register(&mut interp);
        interp.eval(r#"bind "ctrl+s" { buffer save }"#).unwrap();
        let st = handle.lock().unwrap();
        let ks = KeySpec::parse("ctrl+s").unwrap();
        let action = st.bindings.lookup_single(&ks.strokes[0]);
        assert!(action.is_some());
        assert_eq!(action.unwrap().script.trim(), "buffer save");
    }

    #[test]
    fn hook_add_and_list() {
        let mut interp = Interpreter::new();
        let _handle = BridgeRegistry::register(&mut interp);
        interp
            .eval(r#"hook add "startup" { puts "hello" }"#)
            .unwrap();
        let result = interp.eval("hook list").unwrap();
        assert!(!result.as_str().is_empty());
    }

    #[test]
    fn kiro_status_returns_idle() {
        let mut interp = Interpreter::new();
        let _handle = BridgeRegistry::register(&mut interp);
        let result = interp.eval("kiro status").unwrap();
        assert_eq!(result.as_str().as_ref(), "idle");
    }
}
